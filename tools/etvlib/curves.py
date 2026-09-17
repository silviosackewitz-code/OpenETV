"""Generating a torque request: a shape over the grip, scaled to the engine.

A shape maps grip 0…1 to a share 0…1 of the torque at full grip. With a cable
the grip is the throttle, and most engines then give most of their torque in
the first half of the travel — a concave shape. Ride-by-wire is free to put
the fine control where it is needed instead, for example through the low and
middle grip used out of a corner — a convex shape.
"""

from __future__ import annotations

from collections.abc import Callable

import numpy as np

from .tables import Table

Shape = Callable[[np.ndarray], np.ndarray]

#: Exponents of the power shapes `x ** n`: below 1 concave, above 1 convex.
POWER_PRESETS = {"cable": 0.6, "linear": 1.0, "corner_exit": 1.8}


def power_shape(n: float) -> Shape:
    return lambda x: x ** n


def s_curve_shape(center: float, steepness: float) -> Shape:
    """Flat below `center` (0…1 of the grip), steep above; a logistic curve
    stretched so that it runs from 0 at grip 0 to 1 at full grip."""
    lo = 1.0 / (1.0 + np.exp(steepness * center))
    hi = 1.0 / (1.0 + np.exp(-steepness * (1 - center)))

    def shape(x):
        return (1.0 / (1.0 + np.exp(-steepness * (x - center))) - lo) / (hi - lo)
    return shape


def generate_request(engine: Table, rpm: np.ndarray, grip: np.ndarray, shape: Shape,
                     max_percent: float) -> Table:
    """A torque request in Nm, rounded to 0.01: at every RPM the shape, scaled
    to `max_percent` of the engine's maximum torque at that RPM (interpolated
    between the engine table's RPM breakpoints)."""
    max_torque = np.interp(rpm, engine.rpm, engine.values.max(axis=1))
    values = np.outer(max_torque * (max_percent / 100.0), shape(grip / 100.0))
    return Table(np.asarray(rpm, dtype=float), np.asarray(grip, dtype=float), np.round(values, 2))
