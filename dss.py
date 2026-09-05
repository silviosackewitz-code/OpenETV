"""
Reader/writer for the Mectronik "DataSubset" (.dss) XML format used by the
ECU calibration software (Mecal) to export/import 2D calibration tables
(table_3d: two breakpoint axes + a value matrix) and 1D breakpoint vectors.

A .dss file can contain several <table_3d> blocks (e.g. one per gear).
Axis breakpoints are stored separately as <vector> elements and referenced
by path from <AxeX>/<AxeY> inside each table_3d.
"""
import xml.etree.ElementTree as ET

import numpy as np
import pandas as pd


def _is_rpm(unit, path):
    unit = (unit or "").strip().lower()
    path = (path or "").upper()
    return unit == "1/min" or "RPM" in path


def parse_dss(xml_text):
    """Parse a .dss XML string. Returns an ordered dict:
    {table_path: {"df": DataFrame(index=RPM, columns=other axis),
                  "unit", "range_min", "range_max",
                  "rpm_path", "rpm_unit", "other_path", "other_unit"}}
    DataFrame is always oriented with RPM as the row index, regardless of
    whether RPM was AxeX or AxeY in the source file.
    """
    root = ET.fromstring(xml_text)

    vectors = {}
    for v in root.findall("vector"):
        path = v.findtext("path")
        unit = v.findtext("unit")
        values = [float(x.text) for x in v.findall("value")]
        vectors[path] = {"unit": unit, "values": values}

    tables = {}
    for t in root.findall("table_3d"):
        path = t.findtext("path")
        unit = t.findtext("unit")
        range_min = t.findtext("rangeMin")
        range_max = t.findtext("rangeMax")
        axeX_path = t.findtext("AxeX")
        axeY_path = t.findtext("AxeY")

        rows = [[float(x.text) for x in row.findall("value")] for row in t.findall("row")]
        matrix = np.array(rows)  # shape (len(Y breakpoints), len(X breakpoints))

        x_info = vectors.get(axeX_path, {})
        y_info = vectors.get(axeY_path, {})
        x_vals = np.array(x_info.get("values", list(range(matrix.shape[1]))), dtype=float)
        y_vals = np.array(y_info.get("values", list(range(matrix.shape[0]))), dtype=float)
        x_unit = x_info.get("unit", "")
        y_unit = y_info.get("unit", "")

        if _is_rpm(x_unit, axeX_path):
            rpm_vals, other_vals, df_matrix = x_vals, y_vals, matrix.T
            rpm_path, rpm_unit, other_path, other_unit = axeX_path, x_unit, axeY_path, y_unit
        elif _is_rpm(y_unit, axeY_path):
            rpm_vals, other_vals, df_matrix = y_vals, x_vals, matrix
            rpm_path, rpm_unit, other_path, other_unit = axeY_path, y_unit, axeX_path, x_unit
        else:
            # Fallback: assume AxeX is RPM if nothing matches.
            rpm_vals, other_vals, df_matrix = x_vals, y_vals, matrix.T
            rpm_path, rpm_unit, other_path, other_unit = axeX_path, x_unit, axeY_path, y_unit

        df = pd.DataFrame(df_matrix, index=rpm_vals, columns=other_vals)
        df = df.sort_index().sort_index(axis=1)

        tables[path] = {
            "df": df,
            "unit": unit,
            "range_min": range_min,
            "range_max": range_max,
            "rpm_path": rpm_path,
            "rpm_unit": rpm_unit,
            "other_path": other_path,
            "other_unit": other_unit,
        }
    return tables


def build_dss_xml(
    df,
    table_path,
    table_unit,
    range_min,
    range_max,
    rpm_path,
    rpm_unit,
    other_path,
    other_unit,
    dataset_name=None,
):
    """Serialize a DataFrame (index=RPM, columns=other axis, e.g. Pedal/Gas)
    into a Mectronik .dss XML string with one table_3d and its two vectors.
    """
    dataset_name = dataset_name or table_path
    root = ET.Element("dataset", notes="", name=dataset_name)

    t = ET.SubElement(root, "table_3d")
    ET.SubElement(t, "path").text = table_path
    ET.SubElement(t, "unit").text = table_unit
    ET.SubElement(t, "rangeMin").text = str(range_min)
    ET.SubElement(t, "rangeMax").text = str(range_max)
    ET.SubElement(t, "AxeX").text = rpm_path
    ET.SubElement(t, "AxeY").text = other_path

    # Rows follow the "other" (AxeY) axis order; each row lists RPM (AxeX) values.
    matrix = df.values.T  # shape (n_other, n_rpm)
    for row_vals in matrix:
        row_el = ET.SubElement(t, "row")
        for val in row_vals:
            ET.SubElement(row_el, "value").text = f"{val:.6f}"

    vx = ET.SubElement(root, "vector")
    ET.SubElement(vx, "path").text = rpm_path
    ET.SubElement(vx, "unit").text = rpm_unit
    ET.SubElement(vx, "rangeMin").text = "0"
    ET.SubElement(vx, "rangeMax").text = f"{df.index.max():.0f}"
    ET.SubElement(vx, "sizeMax").text = str(len(df.index))
    for v in df.index:
        ET.SubElement(vx, "value").text = f"{v:.6f}"

    vy = ET.SubElement(root, "vector")
    ET.SubElement(vy, "path").text = other_path
    ET.SubElement(vy, "unit").text = other_unit
    ET.SubElement(vy, "rangeMin").text = "0"
    ET.SubElement(vy, "rangeMax").text = f"{df.columns.max():.0f}"
    ET.SubElement(vy, "sizeMax").text = str(len(df.columns))
    for v in df.columns:
        ET.SubElement(vy, "value").text = f"{v:.6f}"

    ET.indent(root, space="  ")
    xml_bytes = ET.tostring(root, encoding="utf-8", xml_declaration=False)
    header = '<?xml version="1.0" encoding="utf-8" standalone="yes"?>\n'
    return header + xml_bytes.decode("utf-8")
