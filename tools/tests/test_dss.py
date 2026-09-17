import numpy as np

from etvlib import dss
from etvlib.tables import make_table

ETV = make_table([4000, 8000], [0, 50, 100], [[0, 30.5, 100], [0, 42.25, 100]])


def _xml(table=ETV):
    return dss.build_dss_xml(table, "ETV.Target.TPS", "%", 0, 100, "BreakPt.RPM", "1/min", "BreakPt.GAS", "%")


def test_a_written_file_is_read_back_the_same():
    [read] = dss.parse_dss(_xml()).values()
    assert read.path == "ETV.Target.TPS" and read.unit == "%"
    assert (read.rpm_path, read.rpm_unit, read.other_path, read.other_unit) == \
        ("BreakPt.RPM", "1/min", "BreakPt.GAS", "%")
    assert np.array_equal(read.table.rpm, ETV.rpm) and np.array_equal(read.table.axis, ETV.axis)
    assert np.array_equal(read.table.values, ETV.values)


def test_rpm_is_written_as_the_x_axis_one_row_per_grip_breakpoint():
    xml = _xml()
    assert "<AxeX>BreakPt.RPM</AxeX>" in xml and "<AxeY>BreakPt.GAS</AxeY>" in xml
    assert xml.count("<row>") == 3
    assert xml.startswith('<?xml version="1.0" encoding="utf-8" standalone="yes"?>\n<dataset notes="" name=')


def test_rpm_becomes_the_rows_whichever_axis_holds_it():
    # The same table written the other way round: grip as AxeX, one <row> per RPM.
    rows = "".join("<row>" + "".join(f"<value>{v}</value>" for v in row) + "</row>" for row in ETV.values)
    vectors = "".join(f"<vector><path>{path}</path><unit>{unit}</unit>"
                      + "".join(f"<value>{v}</value>" for v in values) + "</vector>"
                      for path, unit, values in (("BreakPt.GAS", "%", ETV.axis), ("BreakPt.RPM", "1/min", ETV.rpm)))
    xml = (f"<dataset><table_3d><path>T</path><unit>%</unit><AxeX>BreakPt.GAS</AxeX><AxeY>BreakPt.RPM</AxeY>"
           f"{rows}</table_3d>{vectors}</dataset>")
    [read] = dss.parse_dss(xml).values()
    assert (read.rpm_path, read.other_path) == ("BreakPt.RPM", "BreakPt.GAS")
    assert np.array_equal(read.table.rpm, ETV.rpm) and np.array_equal(read.table.values, ETV.values)


def test_the_real_files_come_back_the_same(real_engine, real_maps):
    assert real_engine.values.shape == (19, 22)
    assert list(real_maps) == ["Demand.Dry.Gear1", "Demand.Dry.Gear2", "Demand.Dry.Gear3", "Demand.Dry.Gear456"]
    for read in real_maps.values():
        assert read.unit == "%"                                       # throttle, not torque
        xml = dss.build_dss_xml(read.table, read.path, read.unit, read.range_min, read.range_max,
                                read.rpm_path, read.rpm_unit, read.other_path, read.other_unit)
        [again] = dss.parse_dss(xml).values()
        assert np.array_equal(again.table.values, read.table.values)
