import numpy as np
import pytest

from etvlib import curves
from etvlib.tables import make_table


@pytest.mark.parametrize("shape", [curves.power_shape(n) for n in curves.POWER_PRESETS.values()]
                         + [curves.s_curve_shape(0.6, 8.0), curves.s_curve_shape(0.05, 20.0)])
def test_every_shape_runs_from_nothing_to_everything_and_never_falls(shape):
    y = shape(np.linspace(0.0, 1.0, 101))
    assert y[0] == pytest.approx(0.0) and y[-1] == pytest.approx(1.0)
    assert (np.diff(y) >= 0).all()


def test_cable_is_concave_and_corner_exit_convex():
    half = np.array([0.5])
    assert curves.power_shape(curves.POWER_PRESETS["cable"])(half)[0] > 0.5
    assert curves.power_shape(curves.POWER_PRESETS["linear"])(half)[0] == 0.5
    assert curves.power_shape(curves.POWER_PRESETS["corner_exit"])(half)[0] < 0.5


def test_the_s_curve_is_flat_below_its_centre():
    assert curves.s_curve_shape(0.6, 8.0)(np.array([0.0, 0.3, 0.6, 1.0])).round(4).tolist() == \
        [0.0, 0.0787, 0.5163, 1.0]


ENGINE = make_table([0, 4000, 8000], [0, 50, 100], [[0, 0, 0], [0, 50, 60], [0, 70, 100]])


def test_the_same_grip_asks_for_the_same_torque_at_every_rpm():
    request = curves.generate_request(ENGINE, np.array([4000.0, 6000.0, 8000.0]),
                                      np.array([0.0, 25.0, 50.0, 100.0]), curves.power_shape(1.0), 100.0)
    # 1 % grip is 1 Nm with this engine of 100 Nm — until an RPM has no more to give.
    assert request.values.tolist() == [[0.0, 25.0, 50.0, 60.0], [0.0, 25.0, 50.0, 80.0], [0.0, 25.0, 50.0, 100.0]]


def test_full_grip_can_ask_for_less_than_the_engine_has():
    request = curves.generate_request(ENGINE, np.array([4000.0, 8000.0]), np.array([50.0, 100.0]),
                                      curves.power_shape(1.0), 50.0)
    assert request.values.tolist() == [[25.0, 50.0], [25.0, 50.0]]


def test_a_row_without_torque_does_not_pull_the_maximum_down():
    # 2000 rpm lies below the engine's first real row: that row's maximum is held, not halved.
    request = curves.generate_request(ENGINE, np.array([2000.0]), np.array([100.0]), curves.power_shape(1.0), 100.0)
    assert request.values.tolist() == [[60.0]]


def test_scaled_to_the_maximum_at_each_rpm_on_request(sample_engine):
    # 4000 rpm lies between two rows of the engine table: its maximum is interpolated.
    request = curves.generate_request(sample_engine, np.array([3000.0, 4000.0, 9000.0]),
                                      np.array([0.0, 25.0, 50.0, 100.0]), curves.power_shape(1.8), 90.0, per_rpm=True)
    assert request.values.tolist() == [[0.0, 4.08, 14.22, 49.5], [0.0, 4.94, 17.19, 59.85], [0.0, 7.27, 25.33, 88.2]]
