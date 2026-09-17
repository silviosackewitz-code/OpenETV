"""Reading and writing Mectronik "DataSubset" files (.dss).

The XML format in which the ECU calibration software (Mecal) exports and
imports calibration tables: `table_3d` (two breakpoint axes and a matrix of
values) and `vector` (the breakpoints, referenced by path from `AxeX` and
`AxeY` of a table). One file can hold several tables, for example one per
gear.
"""

from __future__ import annotations

import xml.etree.ElementTree as ET
from dataclasses import dataclass

import numpy as np

from .tables import Table, make_table


@dataclass(frozen=True)
class DssTable:
    """A table of a .dss file, rows turned to RPM whichever axis held it."""

    path: str
    table: Table
    unit: str | None
    range_min: str | None
    range_max: str | None
    rpm_path: str | None
    rpm_unit: str
    other_path: str | None
    other_unit: str


def _is_rpm(unit, path) -> bool:
    return (unit or "").strip().lower() == "1/min" or "RPM" in (path or "").upper()


def parse_dss(xml_text: str) -> dict[str, DssTable]:
    """The tables of a .dss file by their path, in the order of the file."""
    root = ET.fromstring(xml_text)

    vectors = {}
    for v in root.findall("vector"):
        vectors[v.findtext("path")] = {"unit": v.findtext("unit"),
                                       "values": [float(x.text) for x in v.findall("value")]}

    tables = {}
    for t in root.findall("table_3d"):
        path = t.findtext("path")
        x_path, y_path = t.findtext("AxeX"), t.findtext("AxeY")
        # One <row> per AxeY breakpoint, one <value> per AxeX breakpoint.
        matrix = np.array([[float(x.text) for x in row.findall("value")] for row in t.findall("row")])

        x_info, y_info = vectors.get(x_path, {}), vectors.get(y_path, {})
        x_vals = x_info.get("values", list(range(matrix.shape[1])))
        y_vals = y_info.get("values", list(range(matrix.shape[0])))
        x_unit, y_unit = x_info.get("unit", ""), y_info.get("unit", "")

        # RPM is AxeX unless only AxeY looks like it.
        if _is_rpm(y_unit, y_path) and not _is_rpm(x_unit, x_path):
            table = make_table(y_vals, x_vals, matrix)
            rpm_path, rpm_unit, other_path, other_unit = y_path, y_unit, x_path, x_unit
        else:
            table = make_table(x_vals, y_vals, matrix.T)
            rpm_path, rpm_unit, other_path, other_unit = x_path, x_unit, y_path, y_unit

        tables[path] = DssTable(path, table, t.findtext("unit"), t.findtext("rangeMin"),
                                t.findtext("rangeMax"), rpm_path, rpm_unit, other_path, other_unit)
    return tables


def build_dss_xml(table: Table, table_path: str, table_unit: str, range_min, range_max,
                  rpm_path: str, rpm_unit: str, other_path: str, other_unit: str,
                  dataset_name: str | None = None) -> str:
    """A .dss file with one `table_3d` (RPM as AxeX) and its two vectors."""
    root = ET.Element("dataset", notes="", name=dataset_name or table_path)

    t = ET.SubElement(root, "table_3d")
    ET.SubElement(t, "path").text = table_path
    ET.SubElement(t, "unit").text = table_unit
    ET.SubElement(t, "rangeMin").text = str(range_min)
    ET.SubElement(t, "rangeMax").text = str(range_max)
    ET.SubElement(t, "AxeX").text = rpm_path
    ET.SubElement(t, "AxeY").text = other_path
    for row_vals in table.values.T:
        row = ET.SubElement(t, "row")
        for val in row_vals:
            ET.SubElement(row, "value").text = f"{val:.6f}"

    for path, unit, breakpoints in ((rpm_path, rpm_unit, table.rpm), (other_path, other_unit, table.axis)):
        v = ET.SubElement(root, "vector")
        ET.SubElement(v, "path").text = path
        ET.SubElement(v, "unit").text = unit
        ET.SubElement(v, "rangeMin").text = "0"
        ET.SubElement(v, "rangeMax").text = f"{breakpoints.max():.0f}"
        ET.SubElement(v, "sizeMax").text = str(len(breakpoints))
        for value in breakpoints:
            ET.SubElement(v, "value").text = f"{value:.6f}"

    ET.indent(root, space="  ")
    return ('<?xml version="1.0" encoding="utf-8" standalone="yes"?>\n'
            + ET.tostring(root, encoding="utf-8", xml_declaration=False).decode("utf-8"))
