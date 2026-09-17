"""
OpenETV - Throttle Position Map Generator: the Streamlit window.

What is calculated, and how, is in `tools/etvlib` (`core.py`); this script
only shows tables, collects settings and offers downloads. It is replaced by
a window of its own in the rebuild (docs/plan.md).
"""
import os
import sys

import altair as alt
import numpy as np
import pandas as pd
import streamlit as st

# The package lies in tools/, next to this file — also inside the built app.
sys.path.insert(0, os.path.join(os.path.dirname(os.path.abspath(__file__)), "tools"))

from etvlib import core, curves, dss, tables  # noqa: E402 — needs the path above

st.set_page_config(page_title="OpenETV – Throttle Position Map", layout="wide")

# Resolve sample data relative to this file, not the current working
# directory (which is unpredictable when launched from a packaged app).
APP_DIR = os.path.dirname(os.path.abspath(__file__))
SAMPLE_ENGINE_PATH = os.path.join(APP_DIR, "sample_data", "engine_torque_map.csv")
SAMPLE_DEMAND_PATH = os.path.join(APP_DIR, "sample_data", "demand_map.csv")


def to_frame(table, corner=None):
    df = pd.DataFrame(table.values, index=table.rpm, columns=table.axis)
    df.index.name = corner
    return df


def to_table(df):
    return tables.make_table(df.index, df.columns, df.values)


def load_table(uploaded_file, default_path, key_prefix):
    """Loads a RPM x <axis> table from CSV/Excel/.dss upload, or a default CSV.
    Returns (df, meta) where meta is the `dss.DssTable` if loaded from a .dss
    file (for re-use as DSS export defaults), else None.
    """
    try:
        if uploaded_file is None:
            with open(default_path, encoding="utf-8") as f:
                return to_frame(tables.read_csv(f.read())), None
        if uploaded_file.name.endswith(".dss"):
            found = dss.parse_dss(uploaded_file.getvalue().decode("utf-8"))
            if not found:
                st.error("No table_3d table found in this .dss file.")
                st.stop()
            table_path = st.selectbox(
                "Select table from .dss", options=list(found.keys()), key=f"{key_prefix}_dss_select"
            )
            info = found[table_path]
            st.caption(
                f"Loaded: `{table_path}` [{info.unit}] – RPM axis `{info.rpm_path}` "
                f"[{info.rpm_unit}], other axis `{info.other_path}` [{info.other_unit}]"
            )
            return to_frame(info.table), info
        if uploaded_file.name.endswith((".xlsx", ".xls")):
            return to_frame(tables.read_xlsx(uploaded_file.getvalue())), None
        return to_frame(tables.read_csv(uploaded_file.getvalue().decode("utf-8"))), None
    except tables.TableError as error:
        st.error(str(error))
        st.stop()


def df_to_download_buttons(df, base_name, key_prefix):
    table = to_table(df)
    st.download_button(
        "Download CSV", tables.to_csv(table, df.index.name or "").encode("utf-8"),
        file_name=f"{base_name}.csv", mime="text/csv", key=f"{key_prefix}_csv",
    )
    st.download_button(
        "Download Excel",
        tables.to_xlsx(table, base_name, df.index.name or ""),
        file_name=f"{base_name}.xlsx",
        mime="application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
        key=f"{key_prefix}_xlsx",
    )


def heatmap(df, value_name, color_scheme="viridis"):
    long = df.reset_index().melt(id_vars=df.index.name or "index", var_name="col", value_name=value_name)
    long.columns = ["row", "col", value_name]
    long["row"] = long["row"].astype(str)
    long["col"] = long["col"].astype(str)
    chart = (
        alt.Chart(long)
        .mark_rect()
        .encode(
            x=alt.X("col:O", title=df.columns.name or "Column", sort=list(df.columns.astype(str))),
            y=alt.Y("row:O", title=df.index.name or "Row", sort=list(df.index.astype(str))),
            color=alt.Color(f"{value_name}:Q", scale=alt.Scale(scheme=color_scheme)),
            tooltip=["row", "col", alt.Tooltip(f"{value_name}:Q", format=".1f")],
        )
        .properties(height=350)
    )
    st.altair_chart(chart, use_container_width=True)


st.title("OpenETV – Throttle Position Map Generator")
st.caption(
    "Modeled after the ETV Builder workflow of the EGEA Bike Torque Tool: from an "
    "engine torque table (RPM × Throttle, TORQUE DYNO) and a rider demand table "
    "(RPM × Pedal, TORQUE TARGET), the throttle position (ETV MAP / TPS Target) is "
    "computed."
)

col1, col2 = st.columns(2)
with col1:
    st.subheader("1) Engine Torque Table (TORQUE DYNO)")
    st.caption(
        "Rows = RPM, columns = Throttle [%], values = Torque [Nm]. "
        "Values at TPS=0 may be negative (drag torque/friction)."
    )
    engine_upload = st.file_uploader(
        "Load your own table (CSV/Excel/.dss)", type=["csv", "xlsx", "dss"], key="engine_upload"
    )
    engine_df, engine_meta = load_table(engine_upload, SAMPLE_ENGINE_PATH, "engine")
    engine_df = st.data_editor(engine_df, num_rows="dynamic", key="engine_editor")
    engine_df.index = engine_df.index.astype(float)
    engine_df.columns = engine_df.columns.astype(float)
    engine_df = engine_df.sort_index().sort_index(axis=1)

with col2:
    st.subheader("2) Demand Table (TORQUE TARGET)")
    st.caption("Rows = RPM, columns = Pedal [%], values = Target torque [Nm]")

    with st.expander("Generate demand curve"):
        st.caption(
            "A linearly rising target-torque curve tends to feel 'soft at the start, "
            "harsh at the end' to riders used to a 1:1 gas/throttle cable, since a cable "
            "naturally delivers a lot of torque gain in the first 50–60% of travel and "
            "little afterwards (chapter 3.1, 'A Practical Guide to Race Motorbike "
            "Electronics'). But the whole point of ride-by-wire is that you are **not** "
            "bound to replicate that cable feel: you can instead put the finest control "
            "exactly where precise dosing matters most, e.g. through the low-to-mid "
            "pedal range used for corner-exit throttle application, and let torque ramp "
            "up quickly only once you commit to full power. Pick a preset below or shape "
            "your own curve; the preview updates live as you move the sliders."
        )

        preset = st.selectbox(
            "Example curve",
            [
                "Cable-like feel (concave, fine near closed throttle)",
                "Linear (1:1 gain throughout)",
                "Corner-exit precision (convex, fine through low/mid pedal)",
                "S-curve (custom fine-control zone)",
            ],
        )

        if preset == "Cable-like feel (concave, fine near closed throttle)":
            shape_n = curves.POWER_PRESETS["cable"]
            shape_fn = curves.power_shape(shape_n)
            st.caption(f"n = {shape_n} (fixed for this preset)")
        elif preset == "Linear (1:1 gain throughout)":
            shape_n = curves.POWER_PRESETS["linear"]
            shape_fn = curves.power_shape(shape_n)
            st.caption(f"n = {shape_n} (fixed for this preset)")
        elif preset == "Corner-exit precision (convex, fine through low/mid pedal)":
            shape_n = curves.POWER_PRESETS["corner_exit"]
            shape_fn = curves.power_shape(shape_n)
            st.caption(
                f"n = {shape_n} (fixed for this preset) – flat/precise through low-mid pedal, steep near full gas"
            )
        else:
            sc1, sc2 = st.columns(2)
            with sc1:
                center = st.slider(
                    "Fine-control zone boundary [% pedal]",
                    min_value=5, max_value=95, value=60, step=5,
                    help="Below this pedal position the curve stays flat (fine dosing, "
                         "e.g. for corner-exit modulation); above it torque ramps up "
                         "quickly.",
                )
            with sc2:
                steepness = st.slider(
                    "Transition sharpness",
                    min_value=2.0, max_value=20.0, value=8.0, step=1.0,
                    help="Higher = narrower, more sudden transition from fine control "
                         "to full power.",
                )
            shape_fn = curves.s_curve_shape(center / 100.0, steepness)

        gc1, gc2 = st.columns(2)
        with gc1:
            max_fraction = st.number_input(
                "Target torque at 100% gas [% of engine max]",
                min_value=10.0, max_value=100.0, value=100.0, step=5.0,
            )
        with gc2:
            gas_bp_input = st.text_input(
                "Gas breakpoints [%] (comma-separated, fine near 0% – as in real ECU exports)",
                value="0,2,3,4,5,6,7,8,9,10,12.5,15,17.5,20,22.5,25,30,40,50,60,70,80,90,100",
            )
        gen_rpm_default = ",".join(tables.format_breakpoint(v) for v in engine_df.index)
        gen_rpm_input = st.text_input(
            "RPM breakpoints for the generated curve (comma-separated)",
            value=gen_rpm_default,
            help="Default = RPM breakpoints of the engine torque table. Max torque at "
                 "each requested RPM is linearly interpolated from the engine table if "
                 "it doesn't fall exactly on one of its breakpoints.",
        )

        preview_x = np.linspace(0, 100, 101)
        preview_y = shape_fn(preview_x / 100.0) * max_fraction
        preview_df = pd.DataFrame({"pedal": preview_x, "target_pct": preview_y})
        preview_chart = (
            alt.Chart(preview_df)
            .mark_line()
            .encode(
                x=alt.X("pedal:Q", title="Pedal [%]", scale=alt.Scale(domain=[0, 100])),
                y=alt.Y("target_pct:Q", title="Target torque [% of engine max]", scale=alt.Scale(domain=[0, 100])),
            )
            .properties(height=200)
        )
        st.altair_chart(preview_chart, use_container_width=True)

        if st.button("Generate demand curve"):
            try:
                gas_bp = tables.parse_breakpoints(gas_bp_input)
            except ValueError:
                st.error("Could not parse gas breakpoints as numbers.")
                st.stop()
            try:
                rpm_bp = tables.parse_breakpoints(gen_rpm_input)
            except ValueError:
                st.error("Could not parse RPM breakpoints as numbers.")
                st.stop()
            generated_df = to_frame(
                curves.generate_request(to_table(engine_df), rpm_bp, gas_bp, shape_fn, max_fraction),
                "RPM\\Pedal[%]",
            )
            st.session_state["demand_base_df"] = generated_df
            st.session_state["demand_version"] = st.session_state.get("demand_version", 0) + 1
            st.rerun()

    demand_upload = st.file_uploader(
        "Load your own table (CSV/Excel/.dss)", type=["csv", "xlsx", "dss"], key="demand_upload"
    )
    demand_meta = None
    if demand_upload is not None:
        demand_df, demand_meta = load_table(demand_upload, SAMPLE_DEMAND_PATH, "demand")
    elif "demand_base_df" in st.session_state:
        demand_df = st.session_state["demand_base_df"]
    else:
        demand_df, demand_meta = load_table(None, SAMPLE_DEMAND_PATH, "demand")
    demand_key = f"demand_editor_{st.session_state.get('demand_version', 0)}"
    demand_df = st.data_editor(demand_df, num_rows="dynamic", key=demand_key)
    demand_df.index = demand_df.index.astype(float)
    demand_df.columns = demand_df.columns.astype(float)
    demand_df = demand_df.sort_index().sort_index(axis=1)

c1, c2 = st.columns(2)
with c1:
    st.markdown("**Engine map**")
    heatmap(engine_df, "Torque")
with c2:
    st.markdown("**Demand map**")
    heatmap(demand_df, "Target torque")

st.divider()
st.subheader("3) Calculation Settings (ETV MAP)")

engine_rpm_bp = engine_df.index.values.astype(float)

default_rpm = ",".join(tables.format_breakpoint(v) for v in demand_df.index)
default_pedal = ",".join(tables.format_breakpoint(v) for v in demand_df.columns)
rc0, rc1 = st.columns(2)
with rc0:
    rpm_input = st.text_input("RPM breakpoints (comma-separated)", value=default_rpm)
with rc1:
    pedal_input = st.text_input("Pedal breakpoints [%] (comma-separated)", value=default_pedal)
rc2, rc3 = st.columns(2)
with rc2:
    rpm_calc_method = st.selectbox(
        "RPM Calc Method (threshold for saturation case)",
        options=list(engine_rpm_bp),
        index=len(engine_rpm_bp) // 2,
        help=(
            "Below this RPM, saturation picks the smallest TPS breakpoint that "
            "reaches the max torque; above it, the largest."
        ),
    )
with rc3:
    tolerance = st.number_input("Max Torque Tolerance [Nm]", min_value=0.0, value=0.3, step=0.1)

st.caption(
    "Default = RPM breakpoints of the demand table. Both tables are read at the exact "
    "output RPM: between two RPM rows of the engine torque table its torque is "
    "interpolated linearly, as the ECU does; rows without any torque (a dummy row at "
    "0 rpm) are left out."
)

try:
    out_rpm = tables.parse_breakpoints(rpm_input)
except ValueError:
    st.error("Could not parse RPM breakpoints as numbers.")
    st.stop()
try:
    out_pedal = tables.parse_breakpoints(pedal_input)
except ValueError:
    st.error("Could not parse pedal breakpoints as numbers.")
    st.stop()

if st.button("Calculate ETV MAP", type="primary"):
    result = core.calculate(to_table(engine_df), to_table(demand_df), out_rpm, out_pedal, rpm_calc_method, tolerance)
    st.session_state["result_df"] = to_frame(result.table, "RPM\\Pedal[%]")
    st.session_state["status"] = result.status
    st.session_state["outside_rpm"] = result.outside
    st.session_state["out_pedal"] = out_pedal
    st.session_state["demand_meta"] = demand_meta
    st.session_state["engine_meta"] = engine_meta

if "result_df" in st.session_state:
    result_df = st.session_state["result_df"]
    status = st.session_state["status"]

    st.subheader("4) Result: ETV MAP (Throttle TPS [%])")

    outside_rpm = st.session_state.get("outside_rpm", ())
    if outside_rpm:
        st.warning(
            f"{', '.join(tables.format_breakpoint(r) for r in outside_rpm)} rpm lie outside the "
            "engine torque table. Its first or last RPM row was used there – what the engine "
            "really gives at these RPM is not known."
        )

    n_sat = int((status == core.SATURATED).sum())
    n_below = int((status == core.BELOW_MIN).sum())
    n_nonmono = int((status == core.NON_MONOTONIC).sum())
    if n_sat:
        st.warning(
            f"{n_sat} cell(s) are saturated: target torque reaches/exceeds the "
            "available max torque at this RPM (tolerance band taken into account)."
        )
    if n_below:
        st.info(
            f"{n_below} cell(s) are below the torque available at TPS=0% "
            "(e.g. a 0 Nm target with negative drag torque) – TPS was set to 0%; "
            "'Zero-gas fix' below forces TPS=0% at Pedal=0% and ramps up from there."
        )
    if n_nonmono:
        st.warning(
            f"{n_nonmono} cell(s): the engine's torque sags after its peak here and falls "
            "below the request again at a larger throttle. The smallest throttle that "
            "delivers the torque was used – check these cells."
        )

    st.dataframe(result_df, use_container_width=True)
    heatmap(result_df, "Throttle", color_scheme="turbo")

    st.subheader("5) Post-processing")
    pc1, pc2, pc3 = st.columns(3)
    with pc1:
        ramp_to = st.number_input(
            "Zero-gas ramp up to pedal [%]", min_value=0.0, max_value=50.0, value=20.0, step=2.5,
            help="TPS rises in a straight line from 0% at closed pedal to the calculated value "
                 "at the pedal breakpoint nearest to this. Along the ramp the map does not "
                 "follow the torque request, so no further than needed; too short, and the "
                 "first degrees of pedal give a step in torque. 0 = set Pedal=0% only.",
        )
        if st.button("Apply zero-gas fix (Pedal=0% → TPS=0%, then a ramp)"):
            fixed = core.zero_gas_fix(to_table(st.session_state["result_df"]), ramp_to)
            st.session_state["result_df"] = to_frame(fixed, "RPM\\Pedal[%]")
            st.rerun()
    with pc2:
        if st.button("Apply flat-spot fix (monotonic over pedal)"):
            fixed = core.monotonic_fix(to_table(st.session_state["result_df"]))
            st.session_state["result_df"] = to_frame(fixed, "RPM\\Pedal[%]")
            st.rerun()
    with pc3:
        if st.button("Reset (recalculate)"):
            del st.session_state["result_df"]
            st.rerun()

    st.subheader("6) Export")
    df_to_download_buttons(result_df, "etv_map_throttle", key_prefix="result")

    st.markdown("**DSS export (for re-import into the ECU software)**")
    demand_meta = st.session_state.get("demand_meta")
    d1, d2, d3 = st.columns(3)
    with d1:
        table_path = st.text_input(
            "Table path", value="ETV.Target.TPS", help="Name/path under which the ECU software expects the table."
        )
        table_unit = st.text_input("Table unit", value="%")
    with d2:
        rpm_path = st.text_input(
            "RPM axis path", value=(demand_meta.rpm_path if demand_meta else "BreakPt.RPM")
        )
        rpm_unit = st.text_input(
            "RPM axis unit", value=(demand_meta.rpm_unit if demand_meta else "1/min")
        )
    with d3:
        other_path = st.text_input(
            "Pedal axis path", value=(demand_meta.other_path if demand_meta else "BreakPt.GAS")
        )
        other_unit = st.text_input(
            "Pedal axis unit", value=(demand_meta.other_unit if demand_meta else "%")
        )

    xml_str = dss.build_dss_xml(
        to_table(result_df), table_path, table_unit, 0, 100, rpm_path, rpm_unit, other_path, other_unit
    )
    st.download_button(
        "Download .dss",
        xml_str.encode("utf-8"),
        file_name=f"{table_path}.dss",
        mime="application/xml",
        key="result_dss",
    )
    with st.expander("DSS preview"):
        st.code(xml_str[:2000] + ("\n..." if len(xml_str) > 2000 else ""), language="xml")
