"""Generates plausible sample tables for a race-bike RBW torque model:
engine torque map (RPM x Throttle -> Torque, TORQUE DYNO style with negative
closed-throttle/friction torque) and rider demand map (RPM x Pedal -> Torque).
Run once to (re)create the CSVs in this folder.
"""
import numpy as np
import pandas as pd

rpm_bp = np.array([3000, 5000, 7000, 9000, 11000, 13000, 15000])
throttle_bp = np.array([0, 10, 20, 30, 40, 50, 60, 70, 80, 90, 100])
pedal_bp = np.array([0, 10, 20, 30, 40, 50, 60, 70, 80, 90, 100])

# Peak torque per RPM follows a typical bell-shaped curve (Nm), full throttle.
peak_torque_by_rpm = np.array([55, 78, 92, 98, 90, 72, 48])
# Closed-throttle (engine drag / friction) torque, more negative at higher RPM.
friction_by_rpm = -(4 + rpm_bp / 1000.0 * 0.9)

# Engine torque map: saturating curve in throttle from friction up to peak torque.
throttle_shape = 1 - np.exp(-throttle_bp / 35.0)
throttle_shape = throttle_shape / throttle_shape[-1]
engine_map = friction_by_rpm[:, None] + np.outer(
    peak_torque_by_rpm - friction_by_rpm, throttle_shape
)
engine_df = pd.DataFrame(
    np.round(engine_map, 1), index=rpm_bp, columns=throttle_bp
)
engine_df.index.name = "RPM\\Throttle[%]"
engine_df.to_csv("sample_data/engine_torque_map.csv")

# Rider demand map: progressive (soft) low-pedal response, race-typical.
# At pedal=0 the driver wants 0 Nm, even though the engine's closed-throttle
# torque is negative (friction) -- this is the classic "0% gas isn't 0% TPS" case.
pedal_shape = (pedal_bp / 100.0) ** 1.6
demand_map = np.outer(peak_torque_by_rpm, pedal_shape)
demand_df = pd.DataFrame(
    np.round(demand_map, 1), index=rpm_bp, columns=pedal_bp
)
demand_df.index.name = "RPM\\Pedal[%]"
demand_df.to_csv("sample_data/demand_map.csv")

print("Wrote sample_data/engine_torque_map.csv and sample_data/demand_map.csv")
