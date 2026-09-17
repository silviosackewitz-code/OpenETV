"""From an engine torque table and a torque request to the ETV map.

  engine torque   RPM × throttle % -> torque the engine delivers, Nm
  torque request  RPM × grip %     -> torque the rider asks for, Nm
  ETV map         RPM × grip %     -> throttle % the ECU drives to

Per RPM the engine's torque curve over throttle is read backwards: which
throttle gives the requested torque? Modelled on the ETV Builder of the "EGEA
Bike Torque Tool" (Stéphane Egea):

- A request below what the engine gives at closed throttle gets throttle 0 —
  it cannot close further.
- A request at or above the maximum (less a tolerance) is saturated: the
  first breakpoint that reaches the maximum up to a chosen RPM, the last one
  above it ("RPM Calc Method").
- In between, linear interpolation between the two throttle breakpoints
  around the request — the first two coming from closed throttle, since an
  engine's torque often sags after its peak and is then reached twice.

Known defects of this calculation, with measurements: docs/plan.md, "Review of
the calculation". They are kept as they were while the code moved here, and
`tools/tests/test_core.py` holds each as an expected failure.
"""

from __future__ import annotations

from dataclasses import dataclass

import numpy as np

from .tables import Table

#: What happened in a cell of the result.
OK = "ok"
SATURATED = "saturated"
BELOW_MIN = "below_min"
NON_MONOTONIC = "non_monotonic"


def lookup(table: Table, rpm: float, x: float) -> float:
    """The table's value at any point, bilinear between breakpoints, held at the edges."""
    rpm_bp, col_bp = table.rpm, table.axis
    r = np.clip(rpm, rpm_bp.min(), rpm_bp.max())
    c = np.clip(x, col_bp.min(), col_bp.max())
    i1 = min(max(int(np.searchsorted(rpm_bp, r, side="right")), 1), len(rpm_bp) - 1)
    j1 = min(max(int(np.searchsorted(col_bp, c, side="right")), 1), len(col_bp) - 1)
    i0, j0 = i1 - 1, j1 - 1

    r0, r1 = rpm_bp[i0], rpm_bp[i1]
    c0, c1 = col_bp[j0], col_bp[j1]
    fr = 0.0 if r1 == r0 else (r - r0) / (r1 - r0)
    fc = 0.0 if c1 == c0 else (c - c0) / (c1 - c0)

    v = table.values
    return float(
        v[i0, j0] * (1 - fr) * (1 - fc)
        + v[i0, j1] * (1 - fr) * fc
        + v[i1, j0] * fr * (1 - fc)
        + v[i1, j1] * fr * fc
    )


def invert_row(torque: np.ndarray, throttle: np.ndarray, target: float,
               tolerance: float, use_last_at_max: bool) -> tuple[float, str]:
    """The throttle that gives `target` Nm in one RPM row of the engine table,
    and what happened (`OK`, `SATURATED`, `BELOW_MIN`, `NON_MONOTONIC`)."""
    max_t = torque.max()
    min_t = torque[0]

    if target >= max_t - tolerance:
        near_max = np.where(torque >= max_t - tolerance)[0]
        idx = near_max[-1] if use_last_at_max else near_max[0]
        return float(throttle[idx]), SATURATED

    if target <= min_t:
        return float(throttle[0]), BELOW_MIN

    # The first crossing coming from closed throttle: the smallest opening that
    # delivers the torque. Real engine rows sag after their peak, so the same
    # torque can be found again further up — never search such a row as if it
    # were sorted. `j >= 1`, because the row starts below the target.
    j = int(np.argmax(torque >= target))
    t0, t1 = torque[j - 1], torque[j]
    x0, x1 = throttle[j - 1], throttle[j]
    # Worth a look: further open, this engine gives less than was asked for.
    status = NON_MONOTONIC if (torque[j:] < target).any() else OK
    return float(x0 + (target - t0) / (t1 - t0) * (x1 - x0)), status


def engine_row(engine: Table, rpm: float) -> tuple[np.ndarray, bool]:
    """The engine's torque over throttle at any RPM, and whether that RPM lies
    outside the table.

    Between two rows of the table: a straight line between them, as the ECU
    reads the table itself. Rows without any torque do not count — a real
    export carries one at 0 rpm, and it must not pull its neighbours down.
    Outside the rows that count, the nearest of them is held."""
    real = np.flatnonzero(engine.values.any(axis=1))
    if not len(real):
        return engine.values[0], False
    rpms = engine.rpm[real]
    if rpm <= rpms[0] or rpm >= rpms[-1]:
        edge = real[0] if rpm <= rpms[0] else real[-1]
        return engine.values[edge], bool(rpm != engine.rpm[edge])
    hi = int(np.searchsorted(rpms, rpm, side="right"))
    lo = hi - 1
    share = (rpm - rpms[lo]) / (rpms[hi] - rpms[lo])
    return engine.values[real[lo]] * (1 - share) + engine.values[real[hi]] * share, False


@dataclass(frozen=True)
class EtvResult:
    #: Throttle % over RPM × grip %, rounded to 0.1.
    table: Table
    #: Per cell one of `OK`, `SATURATED`, `BELOW_MIN`, `NON_MONOTONIC`.
    status: np.ndarray
    #: The output RPM outside the engine table, which took its first or last row.
    outside: tuple[float, ...]

    def count(self, status: str) -> int:
        return int((self.status == status).sum())


def calculate(engine: Table, request: Table, out_rpm: np.ndarray, out_grip: np.ndarray,
              rpm_threshold: float, tolerance: float) -> EtvResult:
    """The ETV map at the given breakpoints.

    Request and engine are both read at the exact output RPM (`engine_row`).
    Above `rpm_threshold` a saturated cell takes the last breakpoint that
    reaches the maximum, up to it the first."""
    result = np.zeros((len(out_rpm), len(out_grip)))
    status = np.empty_like(result, dtype=object)
    outside = []

    for i, rpm in enumerate(out_rpm):
        torque, is_outside = engine_row(engine, rpm)
        if is_outside:
            outside.append(float(rpm))
        use_last = rpm > rpm_threshold
        for j, grip in enumerate(out_grip):
            result[i, j], status[i, j] = invert_row(
                torque, engine.axis, lookup(request, rpm, grip), tolerance, use_last)

    table = Table(np.asarray(out_rpm, dtype=float), np.asarray(out_grip, dtype=float), np.round(result, 1))
    return EtvResult(table, status, tuple(outside))


def zero_gas_fix(etv: Table, ramp_to: float = 20.0) -> Table:
    """A closed grip closes the throttle, and the throttle rises from there in a
    straight line up to the grip breakpoint nearest to `ramp_to` %.

    Zero torque is not zero throttle, so the calculated map starts above 0 % —
    with a closed grip the engine would keep pulling. Setting only grip 0 % to
    zero leaves a step to the next breakpoint (6–16 % throttle with a real
    table), hence the ramp, as in the original tool. How far it reaches is a
    judgement: along the ramp the map no longer follows the torque request, so
    not further than needed; too short, and the first degrees of grip give a
    step in torque. `ramp_to=0` sets grip 0 % only."""
    values = etv.values.copy()
    values[:, etv.axis == 0.0] = 0.0
    above = etv.axis[etv.axis > 0.0]
    if ramp_to > 0.0 and len(above):
        end = above[int(np.argmin(np.abs(above - ramp_to)))]
        ramp = (etv.axis > 0.0) & (etv.axis < end)
        values[:, ramp] = np.round(np.outer(values[:, etv.axis == end].ravel(), etv.axis[ramp] / end), 1)
    return Table(etv.rpm, etv.axis, values)


def monotonic_fix(etv: Table) -> Table:
    """More grip never means less throttle: the running maximum along each RPM row."""
    return Table(etv.rpm, etv.axis, np.maximum.accumulate(etv.values, axis=1))
