import numpy as np
import pytest

from etvlib import tables
from etvlib.tables import TableError


def test_a_csv_is_read_with_rpm_as_rows_and_sorted():
    table = tables.read_csv("RPM\\TPS,100,0,50\n8000,80,-8,60\n4000,40,-4,30\n")
    assert table.rpm.tolist() == [4000.0, 8000.0]
    assert table.axis.tolist() == [0.0, 50.0, 100.0]
    assert table.values.tolist() == [[-4.0, 30.0, 40.0], [-8.0, 60.0, 80.0]]


def test_a_german_spreadsheet_export_is_read_too():
    table = tables.read_csv("﻿RPM;0;12,5\n4000;-4,5;30\n\n")
    assert table.axis.tolist() == [0.0, 12.5]
    assert table.values.tolist() == [[-4.5, 30.0]]


@pytest.mark.parametrize("text, says", [
    ("RPM,0,50\n", "at least one row"),
    ("RPM,0,50\n4000,1\n", "Row 2 has 1 values"),
    ("RPM,0,50\n4000,1,abc\n", "Row 2, column 2 is not a number"),
    ("RPM,0,50\n4000,1,\n", "Row 2, column 2 is empty"),
])
def test_what_is_not_a_table_is_refused_with_a_sentence(text, says):
    with pytest.raises(TableError, match=says):
        tables.read_csv(text)


def test_csv_and_excel_come_back_as_they_went(sample_engine):
    for again in (tables.read_csv(tables.to_csv(sample_engine, "RPM\\TPS")),
                  tables.read_xlsx(tables.to_xlsx(sample_engine, "engine", "RPM\\TPS"))):
        assert np.array_equal(again.rpm, sample_engine.rpm)
        assert np.array_equal(again.axis, sample_engine.axis)
        assert np.array_equal(again.values, sample_engine.values)


def test_breakpoints_are_typed_as_a_list():
    assert tables.parse_breakpoints("50, 0,12.5,, 100 ").tolist() == [0.0, 12.5, 50.0, 100.0]
    assert [tables.format_breakpoint(v) for v in (4000.0, 12.5)] == ["4000", "12.5"]
    with pytest.raises(ValueError):
        tables.parse_breakpoints("0, ten")
