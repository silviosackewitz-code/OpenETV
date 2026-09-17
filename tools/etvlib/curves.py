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
                     max_percent: float, per_rpm: bool = False) -> Table:
    """A torque request in Nm, rounded to 0.01.

    By default in absolute torque: full grip asks for `max_percent` of the
    engine's one maximum, the shape spreads that over the grip, and the same
    grip means the same torque at every RPM — capped where the engine has
    less. That is what a torque request is for: with a cable the torque at a
    held grip follows the engine's torque curve while the revs rise, and the
    rider has to correct for it.

    `per_rpm=True` scales to the engine's maximum at each RPM instead. Every
    RPM then uses the whole grip travel, and the torque at a held grip follows
    the engine's torque curve again.

    The engine's maximum at an RPM is interpolated between its RPM rows; rows
    without any torque (a dummy row at 0 rpm) do not count."""
    real = engine.values.any(axis=1) if engine.values.any() else np.ones(len(engine.rpm), dtype=bool)
    max_at_rpm = np.interp(rpm, engine.rpm[real], engine.values[real].max(axis=1))
    if per_rpm:
        values = np.outer(max_at_rpm * (max_percent / 100.0), shape(grip / 100.0))
    else:
        full_grip = engine.values.max() * (max_percent / 100.0)
        values = np.minimum(np.outer(np.full(len(rpm), full_grip), shape(grip / 100.0)), max_at_rpm[:, None])
    return Table(np.asarray(rpm, dtype=float), np.asarray(grip, dtype=float), np.round(values, 2))
