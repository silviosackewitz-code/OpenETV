"""The routes of `/api/`, called as the server calls them: a dict in, a dict out."""

import base64

import numpy as np
import pytest

from etvlib import api, core, dss, tables
from etvlib.tables import make_table

ENGINE = make_table([4000, 8000], [0, 50, 100], [[0, 50, 60], [0, 70, 100]])
REQUEST = make_table([4000, 8000], [0, 50, 100], [[0, 25, 50], [0, 25, 50]])


def _b64(data) -> str:
    return base64.b64encode(data if isinstance(data, bytes) else data.encode("utf-8")).decode("ascii")


def test_the_window_starts_with_the_sample_tables(sample_engine, sample_request):
    answer = api.sample({})
    assert np.array_equal(tables.from_json(answer["engine"], "engine").values, sample_engine.values)
    assert np.array_equal(tables.from_json(answer["request"], "request").values, sample_request.values)
    assert api.about({})["version"]


def test_calculate_answers_what_the_package_calculates(sample_engine, sample_request):
    answer = api.calculate({"engine": tables.to_json(sample_engine), "request": tables.to_json(sample_request),
                            "rpm_threshold": 7000, "tolerance": 0.3})
    expected = core.calculate(sample_engine, sample_request, sample_request.rpm, sample_request.axis, 7000.0, 0.3)
    assert answer["table"]["values"] == expected.table.values.tolist()
    assert answer["status"] == expected.status.tolist()
    assert answer["counts"] == {"saturated": 7, "below_min": 0, "non_monotonic": 0}
    assert answer["outside"] == []


def test_calculate_at_breakpoints_of_its_own_names_rpm_outside_the_engine_table():
    answer = api.calculate({"engine": tables.to_json(ENGINE), "request": tables.to_json(REQUEST),
                            "rpm": [9000, 6000, 6000], "grip": [50]})
    assert answer["table"]["rpm"] == [6000.0, 9000.0]                 # sorted, once each
    assert answer["outside"] == [9000.0]


@pytest.mark.parametrize("change, says", [
    ({"engine": None}, "The engine torque table is missing."),
    ({"request": {"rpm": [1], "axis": [0, 1], "values": [[0, None]]}}, "torque request has an empty cell"),
    ({"request": {"rpm": [1], "axis": [0, 1], "values": [[0, "x"]]}}, "torque request has a cell that is not a number"),
    ({"request": {"rpm": [1, 1], "axis": [0, 1], "values": [[0, 1], [0, 1]]}}, "RPM breakpoint 1 twice"),
    ({"request": {"rpm": [1], "axis": [0, 1], "values": [[0]]}}, "torque request: 1 RPM and 2 column"),
    ({"tolerance": -1}, "`tolerance` cannot be below 0"),
    ({"tolerance": "0.3"}, "`tolerance` has to be a number"),
    ({"rpm": []}, "RPM breakpoints are missing"),
    ({"grip": [0, "a"]}, "pedal breakpoints have to be numbers"),
])
def test_what_the_page_got_wrong_comes_back_as_a_sentence(change, says):
    body = {"engine": tables.to_json(ENGINE), "request": tables.to_json(REQUEST), **change}
    with pytest.raises(ValueError, match=says):
        api.calculate(body)


def test_generate_and_its_preview():
    body = {"engine": tables.to_json(ENGINE), "rpm": [4000, 8000], "grip": [0, 50, 100],
            "shape": {"kind": "linear"}, "max_percent": 100}
    assert api.generate(body)["table"]["values"] == [[0.0, 50.0, 60.0], [0.0, 50.0, 100.0]]
    assert api.generate({**body, "per_rpm": True})["table"]["values"] == [[0.0, 30.0, 60.0], [0.0, 50.0, 100.0]]

    curve = api.shape({"shape": {"kind": "s_curve", "center": 60, "steepness": 8}, "max_percent": 80})
    assert len(curve["grip"]) == len(curve["percent"]) == 101
    assert curve["percent"][0] == 0.0 and curve["percent"][-1] == 80.0
    assert api.shape({"shape": {"kind": "power", "n": 2}})["percent"][50] == 25.0

    for spec, says in (({"kind": "wave"}, "Unknown shape"), ({"kind": "power"}, "`n` has to be a number"),
                       ({"kind": "s_curve", "center": 100}, "`center` cannot be above 99"), (None, "is missing")):
        with pytest.raises(ValueError, match=says):
            api.shape({"shape": spec})


def test_each_fix_is_applied_and_says_how_much_it_changed():
    etv = {"rpm": [4000], "axis": [0, 10, 20, 50, 100], "values": [[9, 10, 12, 45, 45]]}
    assert api.postprocess({"table": etv, "fix": "zero_gas"}) == {
        "ok": True, "changed": 2, "table": {"rpm": [4000.0], "axis": [0.0, 10.0, 20.0, 50.0, 100.0],
                                           "values": [[0.0, 6.0, 12.0, 45.0, 45.0]]}}
    assert api.postprocess({"table": etv, "fix": "zero_gas", "ramp_to": 0})["changed"] == 1
    # The plateau 45, 45 becomes a line from (20 %, 12) to (100 %, 45): 12 + 33 × 30/80 at 50 % grip.
    assert api.postprocess({"table": etv, "fix": "flat_spots"})["table"]["values"] == [[9.0, 10.0, 12.0, 24.4, 45.0]]
    assert api.postprocess({"table": etv, "fix": "dips"})["changed"] == 0
    with pytest.raises(ValueError, match="Unknown fix 'smooth'"):
        api.postprocess({"table": etv, "fix": "smooth"})


def test_a_csv_and_an_excel_file_are_parsed():
    csv = api.parse({"name": "engine.CSV", "data": _b64("RPM;0;100\n4000;0;40,5\n"), "role": "engine"})
    assert csv["kind"] == "csv" and csv["tables"][0]["table"]["values"] == [[0.0, 40.5]]
    xlsx = api.parse({"name": "engine.xlsx", "data": _b64(tables.to_xlsx(ENGINE, "engine")), "role": "engine"})
    assert xlsx["tables"][0]["table"] == tables.to_json(ENGINE)


def test_a_dss_file_lists_its_tables_and_says_which_cannot_be_used():
    xml = dss.build_dss_xml(REQUEST, "Demand.Dry.Gear1", "%", 0, 100, "BreakPt.RPM", "1/min", "BreakPt.GAS", "%")
    [found] = api.parse({"name": "Demand.DSS", "data": _b64(xml), "role": "request"})["tables"]
    assert found["path"] == "Demand.Dry.Gear1" and found["rpm_path"] == "BreakPt.RPM"
    assert "throttle map" in found["refused"]
    assert found["table"] == tables.to_json(REQUEST)
    # The same file without a role — for looking at it — refuses nothing.
    assert api.parse({"name": "Demand.dss", "data": _b64(xml)})["tables"][0]["refused"] is None


@pytest.mark.parametrize("name, data, says", [
    ("old.xls", _b64(b"\xd0\xcf"), "old Excel format"),
    ("broken.dss", _b64("<dataset>"), "not a DataSubset file"),
    ("empty.dss", _b64("<dataset/>"), "holds no table"),
    ("broken.xlsx", _b64(b"not a zip"), "cannot be read as an Excel file"),
    ("short.csv", _b64("RPM,0\n"), "at least one row"),
    ("cut.csv", "%%%", "did not arrive in one piece"),
])
def test_a_file_that_cannot_be_read_says_why(name, data, says):
    with pytest.raises(ValueError, match=says):
        api.parse({"name": name, "data": data, "role": "engine"})


def test_every_export_format_comes_back_as_the_same_table():
    etv = tables.to_json(REQUEST)
    csv = api.export({"table": etv, "format": "csv", "name": "my map/1"})
    assert csv["name"] == "my_map_1.csv"
    assert tables.to_json(tables.read_csv(base64.b64decode(csv["data"]).decode())) == etv
    xlsx = api.export({"table": etv, "format": "xlsx"})
    assert xlsx["name"] == "etv_map_throttle.xlsx"
    assert tables.to_json(tables.read_xlsx(base64.b64decode(xlsx["data"]))) == etv

    file = api.export({"table": etv, "format": "dss", "dss": {"table_path": "ETV.Map A", "rpm_path": ""}})
    [read] = dss.parse_dss(base64.b64decode(file["data"]).decode()).values()
    assert file["name"] == "ETV.Map_A.dss"
    assert (read.path, read.unit, read.rpm_path) == ("ETV.Map A", "%", "BreakPt.RPM")   # empty: the default
    assert tables.to_json(read.table) == etv
    with pytest.raises(ValueError, match="Unknown format 'pdf'"):
        api.export({"table": etv, "format": "pdf"})
