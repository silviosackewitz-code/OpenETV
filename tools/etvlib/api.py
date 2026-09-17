"""What the window asks the server: the routes under `/api/`.

The server keeps nothing between requests. The page holds the tables and
sends what a calculation needs; a route turns JSON into tables, calls the
package and turns the answer back. A table travels as
`{"rpm": […], "axis": […], "values": [[…], …]}`, rows being RPM.

Every route takes the decoded request (`body` of a POST, the query of a GET)
and returns a dict. What the user got wrong is raised as `ValueError` (or
`TableError`, `UnitError`, which are) with a sentence for the user;
`server.py` answers it as `{"ok": false, "error": …}`.
"""

from __future__ import annotations

import base64
import binascii
import math
import re
import xml.etree.ElementTree as ET
from pathlib import Path

import numpy as np

from . import __version__, core, curves, dss, tables
from .tables import TableError

#: The tables the window starts with. In the built app they lie next to the
#: package (packaging/openetv.spec), in a checkout at the top of the repository.
_HERE = Path(__file__).resolve().parent
SAMPLE_DIRS = (_HERE / "sample_data", _HERE.parents[1] / "sample_data")

#: What offered tables are needed as, and how a message names them.
ROLES = {"engine": "engine torque table", "request": "torque request"}

#: Defaults of a .dss export, as the ECU software names an ETV map.
DSS_DEFAULTS = {"table_path": "ETV.Target.TPS", "table_unit": "%", "range_min": 0, "range_max": 100,
                "rpm_path": "BreakPt.RPM", "rpm_unit": "1/min", "other_path": "BreakPt.GAS", "other_unit": "%"}

CORNER = "RPM\\Pedal[%]"


# ── reading the request ─────────────────────────────────────

def _number(body: dict, key: str, default: float | None = None, minimum: float | None = None) -> float:
    value = body.get(key, default)
    if isinstance(value, bool) or not isinstance(value, (int, float)) or not math.isfinite(value):
        raise ValueError(f"`{key}` has to be a number.")
    if minimum is not None and value < minimum:
        raise ValueError(f"`{key}` cannot be below {tables.format_breakpoint(minimum)}.")
    return float(value)


def _breakpoints(body: dict, key: str, what: str) -> np.ndarray:
    values = body.get(key)
    if not isinstance(values, list) or not values:
        raise ValueError(f"The {what} are missing.")
    if any(isinstance(v, bool) or not isinstance(v, (int, float)) or not math.isfinite(v) for v in values):
        raise ValueError(f"The {what} have to be numbers.")
    return np.unique(np.asarray(values, dtype=float))


def _shape(spec) -> curves.Shape:
    """`{"kind": "cable" | "linear" | "corner_exit"}`, `{"kind": "power", "n": …}`
    or `{"kind": "s_curve", "center": % of the grip, "steepness": …}`."""
    if not isinstance(spec, dict):
        raise ValueError("The shape of the request is missing.")
    kind = spec.get("kind")
    if kind in curves.POWER_PRESETS:
        return curves.power_shape(curves.POWER_PRESETS[kind])
    if kind == "power":
        return curves.power_shape(_number(spec, "n", minimum=0.05))
    if kind == "s_curve":
        center = _number(spec, "center", 60.0, minimum=1.0)
        if center > 99.0:
            raise ValueError("`center` cannot be above 99.")
        return curves.s_curve_shape(center / 100.0, _number(spec, "steepness", 8.0, minimum=0.5))
    raise ValueError(f"Unknown shape {kind!r}.")


# ── GET ─────────────────────────────────────────────────────

def about(query: dict) -> dict:
    return {"ok": True, "version": __version__, "dss_defaults": DSS_DEFAULTS,
            "presets": curves.POWER_PRESETS}


def sample(query: dict) -> dict:
    """The tables the window starts with, so that it is never empty."""
    folder = next((d for d in SAMPLE_DIRS if d.is_dir()), None)
    if folder is None:
        raise ValueError("The sample tables are missing from this installation.")
    read = {name: tables.read_csv((folder / f"{name}.csv").read_text(encoding="utf-8"))
            for name in ("engine_torque_map", "demand_map")}
    return {"ok": True, "engine": tables.to_json(read["engine_torque_map"]),
            "request": tables.to_json(read["demand_map"])}


# ── POST ────────────────────────────────────────────────────

def parse(body: dict) -> dict:
    """A file the user chose: `{"name": …, "data": base64, "role": "engine" | "request"}`.

    Answers every table of the file. A .dss can hold several, and some of them
    cannot serve as what they were offered for — those come with `refused`, the
    sentence that says why, so the window can list them and not offer them."""
    name = body.get("name")
    role = ROLES.get(body.get("role"))
    if not isinstance(name, str) or not isinstance(body.get("data"), str):
        raise ValueError("No file was sent.")
    try:
        data = base64.b64decode(body["data"], validate=True)
    except (binascii.Error, ValueError):
        raise ValueError(f"{name} did not arrive in one piece.") from None

    suffix = Path(name).suffix.lower()
    if suffix == ".dss":
        try:
            found = dss.parse_dss(data.decode("utf-8-sig"))
        except (ET.ParseError, UnicodeDecodeError, ValueError) as error:
            raise ValueError(f"{name} is not a DataSubset file: {error}") from None
        if not found:
            raise ValueError(f"{name} holds no table (no table_3d).")
        return {"ok": True, "kind": "dss", "tables": [_dss_json(t, role) for t in found.values()]}
    if suffix == ".xls":
        raise ValueError(f"{name} is in the old Excel format. Save it as .xlsx or as CSV.")
    if suffix in (".xlsx", ".xlsm"):
        try:
            table = tables.read_xlsx(data)
        except TableError:
            raise
        except Exception as error:              # noqa: BLE001 — whatever openpyxl makes of a broken file
            raise ValueError(f"{name} cannot be read as an Excel file: {error}") from None
        return {"ok": True, "kind": "xlsx", "tables": [{"path": name, "table": tables.to_json(table)}]}
    try:
        text = data.decode("utf-8-sig")
    except UnicodeDecodeError:
        text = data.decode("cp1252")            # a spreadsheet's export on Windows
    return {"ok": True, "kind": "csv", "tables": [{"path": name, "table": tables.to_json(tables.read_csv(text))}]}


def _dss_json(found: dss.DssTable, role: str | None) -> dict:
    refused = None
    if role:
        try:
            dss.require_torque(found, role)
        except dss.UnitError as error:
            refused = str(error)
    return {"path": found.path, "unit": found.unit, "range_min": found.range_min, "range_max": found.range_max,
            "rpm_path": found.rpm_path, "rpm_unit": found.rpm_unit,
            "other_path": found.other_path, "other_unit": found.other_unit,
            "refused": refused, "table": tables.to_json(found.table)}


def shape(body: dict) -> dict:
    """The curve of a shape over the grip, for the preview: share of the torque
    at full grip, in %, at every whole % of grip."""
    grip = np.linspace(0.0, 100.0, 101)
    share = _shape(body.get("shape"))(grip / 100.0) * _number(body, "max_percent", 100.0, minimum=0.0)
    return {"ok": True, "grip": grip.tolist(), "percent": np.round(share, 3).tolist()}


def generate(body: dict) -> dict:
    engine = tables.from_json(body.get("engine"), ROLES["engine"])
    request = curves.generate_request(
        engine, _breakpoints(body, "rpm", "RPM breakpoints"), _breakpoints(body, "grip", "pedal breakpoints"),
        _shape(body.get("shape")), _number(body, "max_percent", 100.0, minimum=0.0),
        per_rpm=body.get("per_rpm") is True)
    return {"ok": True, "table": tables.to_json(request)}


def calculate(body: dict) -> dict:
    engine = tables.from_json(body.get("engine"), ROLES["engine"])
    request = tables.from_json(body.get("request"), ROLES["request"])
    rpm = _breakpoints(body, "rpm", "RPM breakpoints") if "rpm" in body else request.rpm
    grip = _breakpoints(body, "grip", "pedal breakpoints") if "grip" in body else request.axis
    threshold = _number(body, "rpm_threshold", float(engine.rpm[len(engine.rpm) // 2]))
    result = core.calculate(engine, request, rpm, grip, threshold, _number(body, "tolerance", 0.3, minimum=0.0))
    return {"ok": True, "table": tables.to_json(result.table), "status": result.status.tolist(),
            "counts": {s: result.count(s) for s in (core.SATURATED, core.BELOW_MIN, core.NON_MONOTONIC)},
            "outside": list(result.outside)}


def postprocess(body: dict) -> dict:
    """One fix applied to an ETV map: `zero_gas` (with `ramp_to`), `flat_spots`, `dips`."""
    etv = tables.from_json(body.get("table"), "ETV map")
    fix = body.get("fix")
    if fix == "zero_gas":
        fixed = core.zero_gas_fix(etv, _number(body, "ramp_to", 20.0, minimum=0.0))
    elif fix == "flat_spots":
        fixed = core.flat_spot_fix(etv)
    elif fix == "dips":
        fixed = core.monotonic_fix(etv)
    else:
        raise ValueError(f"Unknown fix {fix!r}.")
    changed = int((fixed.values != etv.values).sum())
    return {"ok": True, "table": tables.to_json(fixed), "changed": changed}


def export(body: dict) -> dict:
    """A table as a file: `{"table": …, "format": "csv" | "xlsx" | "dss", "name": …, "dss": {…}}`.

    Answers the file's name and its content in base64; saving it is the
    window's part."""
    table = tables.from_json(body.get("table"), "table to export")
    name = _file_stem(body.get("name") or "etv_map_throttle")
    kind = body.get("format")
    if kind == "csv":
        return _file(f"{name}.csv", "text/csv", tables.to_csv(table, CORNER).encode("utf-8"))
    if kind == "xlsx":
        return _file(f"{name}.xlsx", "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
                     tables.to_xlsx(table, name, CORNER))
    if kind == "dss":
        given = body.get("dss") if isinstance(body.get("dss"), dict) else {}
        meta = {key: given.get(key) if given.get(key) not in (None, "") else default
                for key, default in DSS_DEFAULTS.items()}
        xml = dss.build_dss_xml(table, str(meta["table_path"]), str(meta["table_unit"]),
                                meta["range_min"], meta["range_max"], str(meta["rpm_path"]),
                                str(meta["rpm_unit"]), str(meta["other_path"]), str(meta["other_unit"]))
        return _file(f"{_file_stem(str(meta['table_path']))}.dss", "application/xml", xml.encode("utf-8"))
    raise ValueError(f"Unknown format {kind!r}.")


def _file_stem(name: str) -> str:
    """A name that is safe as a file name on every system."""
    return re.sub(r"[^\w.\-]+", "_", str(name)).strip("._") or "table"


def _file(name: str, mime: str, content: bytes) -> dict:
    return {"ok": True, "name": name, "mime": mime, "data": base64.b64encode(content).decode("ascii")}


GET = {"about": about, "sample": sample}
POST = {"parse": parse, "shape": shape, "generate": generate, "calculate": calculate,
        "postprocess": postprocess, "export": export}
