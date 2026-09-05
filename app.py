"""
OpenETV - Throttle Position Map Generator

Modeled after the Ride-by-Wire torque model and the ETV Builder workflow of
the original "EGEA Bike Torque Tool" (Stephane Egea):

  TORQUE DYNO      : Engine torque table   RPM x Throttle[%] -> Torque[Nm]
                      (may include negative drag torque at closed throttle)
  TORQUE TARGET    : Rider demand table    RPM x Pedal[%]    -> Target torque[Nm]
  ETV MAP (invert) : Output                RPM x Pedal[%]    -> Throttle[%]
                      (the actual Electronic Throttle Valve target)

Inversion logic per RPM breakpoint (no interpolation across RPM, as in the
original tool):
  - If the target torque is below what is available at TPS=0 (usually
    negative), TPS is set to 0 (closing the throttle further isn't possible).
  - If it is above the max torque (minus tolerance), the cell counts as
    saturated: TPS is set to the first (RPM <= threshold) or last
    (RPM > threshold) breakpoint that reaches the maximum ("RPM Calc Method").
  - Otherwise, linear interpolation is used between the two throttle
    breakpoints that bracket the target torque.

Post-processing (as in the original tool):
  - Zero-gas fix: Pedal=0% is forced to TPS=0%.
  - Flat-spot fix: TPS is made monotonically non-decreasing over increasing
    pedal for each RPM row (more pedal must never mean less throttle).
"""
import io
import os

import altair as alt
import numpy as np
import pandas as pd
import streamlit as st

import dss

st.set_page_config(page_title="OpenETV – Throttle Position Map", layout="wide")

# Resolve sample data relative to this file, not the current working
# directory (which is unpredictable when launched from a packaged app).
APP_DIR = os.path.dirname(os.path.abspath(__file__))
SAMPLE_ENGINE_PATH = os.path.join(APP_DIR, "sample_data", "engine_torque_map.csv")
SAMPLE_DEMAND_PATH = os.path.join(APP_DIR, "sample_data", "demand_map.csv")


def load_table(uploaded_file, default_path, key_prefix):
    """Loads a RPM x <axis> table from CSV/Excel/.dss upload, or a default CSV.
    Returns (df, meta) where meta is the .dss axis/unit info if loaded from a
    .dss file (for re-use as DSS export defaults), else None.
    """
    if uploaded_file is not None:
        if uploaded_file.name.endswith(".dss"):
            text = uploaded_file.getvalue().decode("utf-8")
            tables = dss.parse_dss(text)
            if not tables:
                st.error("No table_3d table found in this .dss file.")
                st.stop()
            table_path = st.selectbox(
                "Select table from .dss", options=list(tables.keys()), key=f"{key_prefix}_dss_select"
            )
            info = tables[table_path]
            st.caption(
                f"Loaded: `{table_path}` [{info['unit']}] – RPM axis `{info['rpm_path']}` "
                f"[{info['rpm_unit']}], other axis `{info['other_path']}` [{info['other_unit']}]"
            )
            return info["df"], {**info, "table_path": table_path}
        if uploaded_file.name.endswith((".xlsx", ".xls")):
            df = pd.read_excel(uploaded_file, index_col=0)
        else:
            df = pd.read_csv(uploaded_file, index_col=0)
    else:
        df = pd.read_csv(default_path, index_col=0)
    df.index = df.index.astype(float)
    df.columns = df.columns.astype(float)
    return df.sort_index().sort_index(axis=1), None


def df_to_download_buttons(df, base_name, key_prefix):
    csv = df.to_csv().encode("utf-8")
    st.download_button(
        "Download CSV", csv, file_name=f"{base_name}.csv", mime="text/csv",
        key=f"{key_prefix}_csv",
    )
    buf = io.BytesIO()
    with pd.ExcelWriter(buf, engine="openpyxl") as writer:
        df.to_excel(writer, sheet_name=base_name)
    st.download_button(
        "Download Excel",
        buf.getvalue(),
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


def bilinear_lookup(df, rpm, pedal):
    """Bilinear interpolation of a RPM x Pedal table at an arbitrary point, clamped to range."""
    rpm_bp = df.index.values
    col_bp = df.columns.values
    r = np.clip(rpm, rpm_bp.min(), rpm_bp.max())
    c = np.clip(pedal, col_bp.min(), col_bp.max())
    i1 = np.searchsorted(rpm_bp, r, side="right")
    i1 = min(max(i1, 1), len(rpm_bp) - 1)
    i0 = i1 - 1
    j1 = np.searchsorted(col_bp, c, side="right")
    j1 = min(max(j1, 1), len(col_bp) - 1)
    j0 = j1 - 1

    r0, r1 = rpm_bp[i0], rpm_bp[i1]
    c0, c1 = col_bp[j0], col_bp[j1]
    fr = 0.0 if r1 == r0 else (r - r0) / (r1 - r0)
    fc = 0.0 if c1 == c0 else (c - c0) / (c1 - c0)

    v00 = df.iloc[i0, j0]
    v01 = df.iloc[i0, j1]
    v10 = df.iloc[i1, j0]
    v11 = df.iloc[i1, j1]
    return (
        v00 * (1 - fr) * (1 - fc)
        + v01 * (1 - fr) * fc
        + v10 * fr * (1 - fc)
        + v11 * fr * fc
    )


def fmt_num(v):
    v = float(v)
    return str(int(v)) if v == int(v) else str(v)


def invert_row(torque_curve, throttle_bp, target, tolerance, use_last_at_max):
    """Given one RPM row of the engine map (torque over throttle), find the TPS
    that yields `target` Nm. Returns (tps, status) where status in
    {"ok", "saturated", "below_min", "non_monotonic"}.
    """
    max_t = torque_curve.max()
    min_t = torque_curve[0]

    if target >= max_t - tolerance:
        near_max_idx = np.where(torque_curve >= max_t - tolerance)[0]
        idx = near_max_idx[-1] if use_last_at_max else near_max_idx[0]
        return float(throttle_bp[idx]), "saturated"

    if target <= min_t:
        return float(throttle_bp[0]), "below_min"

    j = int(np.searchsorted(torque_curve, target, side="right"))
    j = min(max(j, 1), len(torque_curve) - 1)
    t0, t1 = torque_curve[j - 1], torque_curve[j]
    x0, x1 = throttle_bp[j - 1], throttle_bp[j]
    status = "ok" if np.all(np.diff(torque_curve) >= -1e-9) else "non_monotonic"
    if t1 == t0:
        return float(x0), status
    tps = x0 + (target - t0) / (t1 - t0) * (x1 - x0)
    return float(tps), status


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

        def power_shape(x, n):
            return x ** n

        def s_curve_shape(x, center, steepness):
            raw = 1.0 / (1.0 + np.exp(-steepness * (x - center)))
            lo, hi = 1.0 / (1.0 + np.exp(steepness * center)), 1.0 / (1.0 + np.exp(-steepness * (1 - center)))
            return (raw - lo) / (hi - lo)

        if preset == "Cable-like feel (concave, fine near closed throttle)":
            shape_n = 0.6
            shape_fn = lambda x: power_shape(x, shape_n)
            st.caption(f"n = {shape_n} (fixed for this preset)")
        elif preset == "Linear (1:1 gain throughout)":
            shape_n = 1.0
            shape_fn = lambda x: power_shape(x, shape_n)
            st.caption(f"n = {shape_n} (fixed for this preset)")
        elif preset == "Corner-exit precision (convex, fine through low/mid pedal)":
            shape_n = 1.8
            shape_fn = lambda x: power_shape(x, shape_n)
            st.caption(f"n = {shape_n} (fixed for this preset) – flat/precise through low-mid pedal, steep near full gas")
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
            shape_fn = lambda x: s_curve_shape(x, center / 100.0, steepness)

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
        gen_rpm_default = ",".join(fmt_num(v) for v in engine_df.index)
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
                gas_bp = np.array(sorted(float(x) for x in gas_bp_input.split(",") if x.strip()))
            except ValueError:
                st.error("Could not parse gas breakpoints as numbers.")
                st.stop()
            try:
                rpm_bp = np.array(sorted(float(x) for x in gen_rpm_input.split(",") if x.strip()))
            except ValueError:
                st.error("Could not parse RPM breakpoints as numbers.")
                st.stop()
            engine_rpm_for_gen = engine_df.index.values.astype(float)
            engine_max_torque = engine_df.max(axis=1).values.astype(float)
            max_torque = np.interp(rpm_bp, engine_rpm_for_gen, engine_max_torque)
            shape = shape_fn(gas_bp / 100.0)
            generated = np.outer(max_torque * (max_fraction / 100.0), shape)
            generated_df = pd.DataFrame(np.round(generated, 2), index=rpm_bp, columns=gas_bp)
            generated_df.index.name = "RPM\\Pedal[%]"
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
engine_throttle_bp = engine_df.columns.values.astype(float)

default_rpm = ",".join(fmt_num(v) for v in demand_df.index)
default_pedal = ",".join(fmt_num(v) for v in demand_df.columns)
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
    "Default = RPM breakpoints of the demand table. For the engine map row, each "
    "output RPM is internally rounded to the nearest existing breakpoint of the "
    "engine torque table (no RPM interpolation in the engine map, as in the "
    "original tool) – the demand value itself is bilinearly interpolated at the "
    "exact RPM."
)

try:
    out_rpm = np.array(sorted(float(x) for x in rpm_input.split(",") if x.strip()))
except ValueError:
    st.error("Could not parse RPM breakpoints as numbers.")
    st.stop()
try:
    out_pedal = np.array(sorted(float(x) for x in pedal_input.split(",") if x.strip()))
except ValueError:
    st.error("Could not parse pedal breakpoints as numbers.")
    st.stop()

if st.button("Calculate ETV MAP", type="primary"):
    result = np.zeros((len(out_rpm), len(out_pedal)))
    status = np.empty_like(result, dtype=object)
    snapped_count = 0

    for i, rpm in enumerate(out_rpm):
        nearest_idx = int(np.argmin(np.abs(engine_rpm_bp - rpm)))
        snapped_rpm = engine_rpm_bp[nearest_idx]
        if snapped_rpm != rpm:
            snapped_count += 1
        torque_curve = engine_df.loc[snapped_rpm].values.astype(float)
        use_last = snapped_rpm > rpm_calc_method
        for j, pedal in enumerate(out_pedal):
            target = float(bilinear_lookup(demand_df, rpm, pedal))
            tps, st_ij = invert_row(torque_curve, engine_throttle_bp, target, tolerance, use_last)
            result[i, j] = tps
            status[i, j] = st_ij

    result_df = pd.DataFrame(np.round(result, 1), index=out_rpm, columns=out_pedal)
    result_df.index.name = "RPM\\Pedal[%]"
    st.session_state["result_df"] = result_df
    st.session_state["status"] = status
    st.session_state["out_pedal"] = out_pedal
    st.session_state["demand_meta"] = demand_meta
    st.session_state["engine_meta"] = engine_meta
    st.session_state["snapped_count"] = snapped_count

if "result_df" in st.session_state:
    result_df = st.session_state["result_df"]
    status = st.session_state["status"]

    st.subheader("4) Result: ETV MAP (Throttle TPS [%])")

    snapped_count = st.session_state.get("snapped_count", 0)
    if snapped_count:
        st.info(
            f"{snapped_count} RPM breakpoint(s) did not fall exactly on the engine "
            "torque table's grid and were rounded to the nearest breakpoint for the "
            "engine map row (the result's row label still shows the originally "
            "requested RPM)."
        )

    n_sat = int((status == "saturated").sum())
    n_below = int((status == "below_min").sum())
    n_nonmono = int((status == "non_monotonic").sum())
    if n_sat:
        st.warning(
            f"{n_sat} cell(s) are saturated: target torque reaches/exceeds the "
            "available max torque at this RPM (tolerance band taken into account)."
        )
    if n_below:
        st.info(
            f"{n_below} cell(s) are below the torque available at TPS=0% "
            "(e.g. a 0 Nm target with negative drag torque) – TPS was set to 0%; "
            "'Zero-gas fix' below additionally forces TPS=0% exactly at Pedal=0%."
        )
    if n_nonmono:
        st.warning(
            f"{n_nonmono} cell(s) fall in an engine map row that does not increase "
            "monotonically over throttle – check the result there with 'Flat-spot fix' "
            "if needed."
        )

    st.dataframe(result_df, use_container_width=True)
    heatmap(result_df, "Throttle", color_scheme="turbo")

    st.subheader("5) Post-processing")
    pc1, pc2, pc3 = st.columns(3)
    with pc1:
        if st.button("Apply zero-gas fix (Pedal=0% → TPS=0%)"):
            fixed = st.session_state["result_df"].copy()
            if 0.0 in fixed.columns:
                fixed[0.0] = 0.0
            st.session_state["result_df"] = fixed
            st.rerun()
    with pc2:
        if st.button("Apply flat-spot fix (monotonic over pedal)"):
            fixed = st.session_state["result_df"].copy()
            fixed.loc[:, :] = np.maximum.accumulate(fixed.values, axis=1)
            st.session_state["result_df"] = fixed
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
            "RPM axis path", value=(demand_meta["rpm_path"] if demand_meta else "BreakPt.RPM")
        )
        rpm_unit = st.text_input(
            "RPM axis unit", value=(demand_meta["rpm_unit"] if demand_meta else "1/min")
        )
    with d3:
        other_path = st.text_input(
            "Pedal axis path", value=(demand_meta["other_path"] if demand_meta else "BreakPt.GAS")
        )
        other_unit = st.text_input(
            "Pedal axis unit", value=(demand_meta["other_unit"] if demand_meta else "%")
        )

    xml_str = dss.build_dss_xml(
        result_df, table_path, table_unit, 0, 100, rpm_path, rpm_unit, other_path, other_unit
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
