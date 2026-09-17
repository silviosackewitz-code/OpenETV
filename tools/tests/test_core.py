"""The calculation: fixed values of today's behaviour, the properties it
must have, and the defects the review found (docs/plan.md) as expected
failures — each turns into a pass, and then has to lose its mark, when the
defect is fixed."""

import numpy as np
import pytest

from etvlib import core
from etvlib.tables import Table, make_table

known_defect = pytest.mark.xfail(strict=True)


def test_lookup_interpolates_between_breakpoints_and_holds_the_edges(sample_request):
    assert core.lookup(sample_request, 6000, 55) == pytest.approx(32.75)
    assert core.lookup(sample_request, 5000, 50) == 25.7              # a breakpoint itself
    assert core.lookup(sample_request, 100, -5) == 0.0                # below both axes
    assert core.lookup(sample_request, 99999, 200) == 48.0            # above both axes


def test_the_sample_tables_give_the_known_map(sample_engine, sample_request):
    result = core.calculate(sample_engine, sample_request, sample_request.rpm, sample_request.axis, 7000.0, 0.3)
    assert result.table.values[0].tolist() == [4.1, 5.0, 6.7, 9.0, 12.5, 17.0, 22.7, 30.1, 41.2, 58.7, 100.0]
    assert result.table.values[-1].tolist() == [10.2, 11.1, 13.0, 15.6, 18.8, 23.2, 28.7, 36.3, 46.9, 63.5, 100.0]
    assert (result.count(core.OK), result.count(core.SATURATED)) == (70, 7)
    assert result.outside == ()


ROW = np.array([-5.0, 10.0, 30.0, 50.0, 60.0])
THROTTLE = np.array([0.0, 25.0, 50.0, 75.0, 100.0])


def test_a_request_between_two_breakpoints_is_interpolated():
    assert core.invert_row(ROW, THROTTLE, 20.0, 0.0, False) == (37.5, core.OK)


def test_zero_torque_needs_an_open_throttle_when_closed_throttle_brakes():
    tps, status = core.invert_row(ROW, THROTTLE, 0.0, 0.0, False)
    assert tps == pytest.approx(25.0 * 5 / 15) and status == core.OK


def test_a_request_below_closed_throttle_closes_the_throttle():
    assert core.invert_row(ROW, THROTTLE, -8.0, 0.0, False) == (0.0, core.BELOW_MIN)


def test_saturation_takes_the_first_or_the_last_breakpoint_within_the_tolerance():
    # The manual's example: the maximum at 90 %, slightly less at 100 %.
    torque = np.array([0.0, 40.0, 55.9, 55.4])
    throttle = np.array([0.0, 50.0, 90.0, 100.0])
    assert core.invert_row(torque, throttle, 70.0, 0.0, True) == (90.0, core.SATURATED)
    assert core.invert_row(torque, throttle, 70.0, 0.5, True) == (100.0, core.SATURATED)
    assert core.invert_row(torque, throttle, 70.0, 0.5, False) == (90.0, core.SATURATED)


def test_the_threshold_decides_per_engine_row():
    engine = make_table([4000, 8000], [0, 50, 100], [[0, 50, 50], [0, 60, 60]])
    request = make_table([4000, 8000], [0, 100], [[0, 80], [0, 80]])
    result = core.calculate(engine, request, engine.rpm, np.array([100.0]), 4000.0, 0.0)
    assert result.table.values.ravel().tolist() == [50.0, 100.0]      # up to the threshold: first; above: last


def test_the_engines_own_torque_gives_throttle_equal_to_grip(sample_engine):
    """The identity: request what the engine delivers at throttle = grip."""
    request = Table(sample_engine.rpm, sample_engine.axis, sample_engine.values)
    result = core.calculate(sample_engine, request, request.rpm, request.axis, 0.0, 0.0)
    assert np.allclose(result.table.values, np.tile(request.axis, (len(request.rpm), 1)), atol=0.05)


def test_post_processing(sample_engine, sample_request):
    etv = core.calculate(sample_engine, sample_request, sample_request.rpm, sample_request.axis, 7000.0, 0.3).table
    assert (etv.values[:, 0] > 0).all()                               # 0 Nm is not 0 % throttle …
    fixed = core.zero_gas_fix(etv)
    assert (fixed.values[:, 0] == 0).all()                            # … until the fix says so
    assert (np.diff(fixed.values, axis=1) >= 0).all()                 # and no step back on the way up
    assert (fixed.values[:, etv.axis >= 20] == etv.values[:, etv.axis >= 20]).all()

    dipped = Table(np.array([5000.0]), np.array([0.0, 50.0, 100.0]), np.array([[10.0, 8.0, 30.0]]))
    assert core.monotonic_fix(dipped).values.tolist() == [[10.0, 10.0, 30.0]]


#: Like a real engine at low RPM: the torque peaks early and sags after it.
SAGGING = make_table([4500], range(0, 100, 10), [[0.0, 50.0, 86.0, 84.0, 83.0, 83.5, 84.0, 84.5, 85.0, 85.5]])


def test_a_row_that_sags_after_its_peak_is_inverted_on_its_rising_part():
    # 85.2 Nm is reached between 10 and 20 % throttle, and again at 85 %. A
    # binary search, which this once was, answered 84 %.
    tps, status = core.invert_row(SAGGING.values[0], SAGGING.axis, 85.2, 0.0, False)
    assert tps == pytest.approx(10 + 10 * 35.2 / 36)
    assert status == core.NON_MONOTONIC                               # further open the engine gives less

    # Below the sag nothing is doubtful: every larger throttle gives more than 40 Nm.
    assert core.invert_row(SAGGING.values[0], SAGGING.axis, 40.0, 0.0, False) == (8.0, core.OK)


def test_a_plateau_is_answered_with_its_smallest_throttle():
    row, throttle = np.array([0.0, 10.0, 10.0, 20.0]), np.array([0.0, 10.0, 20.0, 30.0])
    assert core.invert_row(row, throttle, 10.0, 0.0, False) == (10.0, core.OK)
    # A real table is 0 Nm up to some throttle: the first torque starts where the zeros end.
    row = np.array([0.0, 0.0, 0.0, 9.0])
    assert core.invert_row(row, throttle, 0.9, 0.0, False) == (21.0, core.OK)


# --- What the review found (docs/plan.md, "What is wrong or missing") ---------------------------

def test_an_rpm_between_two_engine_rows_uses_both():
    engine = make_table([4000, 8000], [0, 100], [[0, 40], [0, 80]])
    request = make_table([4000, 8000], [0, 100], [[0, 30], [0, 30]])
    result = core.calculate(engine, request, np.array([6000.0]), np.array([100.0]), 1e9, 0.0)
    # At 6000 rpm the engine gives 60 Nm wide open; 30 Nm is half of that.
    assert result.table.values[0, 0] == pytest.approx(50.0, abs=0.1)
    assert result.outside == ()


def test_a_row_without_torque_does_not_count_and_rpm_outside_the_table_are_named():
    # As a real export: a dummy row at 0 rpm, the engine itself from 4000.
    engine = make_table([0, 4000, 8000], [0, 100], [[0, 0], [0, 40], [0, 80]])
    request = make_table([0, 8000], [0, 100], [[0, 20], [0, 20]])
    result = core.calculate(engine, request, np.array([2000.0, 4000.0, 9000.0]), np.array([100.0]), 1e9, 0.0)
    # 2000 rpm once took the dummy row — a whole row of 0 % throttle. It takes the 4000 row now.
    assert result.table.values.ravel().tolist() == [50.0, 50.0, 25.0]
    assert result.outside == (2000.0, 9000.0)


def test_the_threshold_is_compared_with_the_output_rpm():
    engine = make_table([4000, 8000], [0, 50, 100], [[0, 50, 50], [0, 50, 50]])
    request = make_table([4000, 8000], [0, 100], [[0, 80], [0, 80]])
    result = core.calculate(engine, request, np.array([5000.0, 7000.0]), np.array([100.0]), 6000.0, 0.0)
    assert result.table.values.ravel().tolist() == [50.0, 100.0]


def test_the_zero_gas_fix_ramps_up_to_20_percent_grip():
    # As the original's fix: 0 at grip 0, then a straight line up to the 20 % breakpoint.
    etv = Table(np.array([8000.0]), np.array([0.0, 5.0, 10.0, 20.0, 50.0]), np.array([[9.0, 10.0, 11.0, 12.0, 30.0]]))
    assert core.zero_gas_fix(etv).values.tolist() == [[0.0, 3.0, 6.0, 12.0, 30.0]]
    # How far is the user's choice; the nearest breakpoint is taken.
    assert core.zero_gas_fix(etv, ramp_to=9.0).values.tolist() == [[0.0, 5.5, 11.0, 12.0, 30.0]]
    assert core.zero_gas_fix(etv, ramp_to=0.0).values.tolist() == [[0.0, 10.0, 11.0, 12.0, 30.0]]


def test_the_ramp_is_a_straight_line_in_grip_not_in_cells():
    etv = Table(np.array([8000.0]), np.array([0.0, 2.0, 10.0, 20.0]), np.array([[9.0, 9.5, 11.0, 12.0]]))
    assert core.zero_gas_fix(etv).values.tolist() == [[0.0, 1.2, 6.0, 12.0]]


def test_a_map_without_a_closed_grip_breakpoint_still_gets_its_ramp():
    etv = Table(np.array([8000.0]), np.array([5.0, 20.0, 50.0]), np.array([[10.0, 12.0, 30.0]]))
    assert core.zero_gas_fix(etv).values.tolist() == [[3.0, 12.0, 30.0]]


@known_defect
def test_review_4_a_flat_spot_becomes_a_ramp():
    # The original's fix (manual p. 40) turns a plateau into a linear ramp up to the last cell.
    etv = Table(np.array([4000.0]), np.array([0.0, 50.0, 60.0, 70.0, 80.0, 90.0, 100.0]),
                np.array([[0.0, 32.5, 45.0, 45.0, 45.0, 45.0, 45.0]]))
    assert hasattr(core, "flat_spot_fix")
    assert core.flat_spot_fix(etv).values.tolist() == [[0.0, 32.5, 35.0, 37.5, 40.0, 42.5, 45.0]]


# --- Against a real ECU (skipped without .reference/) --------------------------------------------

def test_the_real_map_means_one_newton_metre_per_percent_grip(real_engine, real_maps):
    """Why the real 'Demand' tables are ETV maps: sent through the engine
    table as throttle, they give torque = grip, at every RPM."""
    etv = real_maps["Demand.Dry.Gear456"].table
    for rpm in (5000.0, 8000.0, 11000.0):
        for grip in (30.0, 50.0, 70.0):
            throttle = core.lookup(etv, rpm, grip)
            assert core.lookup(real_engine, rpm, throttle) == pytest.approx(grip, abs=3.0)


def test_the_inversion_rebuilds_the_real_map_in_the_middle_of_the_grip(real_engine, real_maps):
    etv = real_maps["Demand.Dry.Gear456"].table
    rpm = etv.rpm[(etv.rpm >= 4000) & (etv.rpm <= 11500)]             # every RPM both tables have
    grip = etv.axis[(etv.axis >= 20) & (etv.axis <= 80)]
    one_nm_per_percent = Table(etv.rpm, etv.axis, np.tile(etv.axis, (len(etv.rpm), 1)))
    ours = core.calculate(real_engine, one_nm_per_percent, rpm, grip, 8000.0, 0.3).table.values
    theirs = np.array([[core.lookup(etv, r, g) for g in grip] for r in rpm])
    assert np.abs(ours - theirs).mean() < 1.5                         # measured: 1.2 % throttle
    assert np.abs(ours - theirs).max() < 6.0                          # measured: 5.3, at 11500 rpm


def test_most_real_engine_rows_do_not_rise_all_the_way_and_are_inverted_all_the_same(real_engine):
    sagging = 0
    for row in real_engine.values[real_engine.values.any(axis=1)]:
        sagging += bool((np.diff(row) < 0).any())
        # The identity on the rising part: every new high is found at its own throttle.
        for j in range(1, int(np.argmax(row))):
            if row[j] > row[:j].max():
                assert core.invert_row(row, real_engine.axis, row[j], 0.0, False)[0] == real_engine.axis[j]
    assert sagging == 8                                               # not an exception: review point 1
