from pathlib import Path

import pytest

from etvlib import dss, tables

ROOT = Path(__file__).resolve().parents[2]
#: The manual of the original tool and real ECU exports: local only, not in git.
REFERENCE = ROOT / ".reference"


@pytest.fixture
def sample_engine():
    return tables.read_csv((ROOT / "sample_data" / "engine_torque_map.csv").read_text(encoding="utf-8"))


@pytest.fixture
def sample_request():
    return tables.read_csv((ROOT / "sample_data" / "demand_map.csv").read_text(encoding="utf-8"))


def _real(name: str) -> dict[str, dss.DssTable]:
    path = REFERENCE / name
    if not path.exists():
        pytest.skip("needs the real ECU exports in .reference/")
    return dss.parse_dss(path.read_text(encoding="utf-8"))


@pytest.fixture
def real_engine():
    """The engine torque table of a real ECU: no negative torque, a dummy row
    at 0 rpm, 8 of its 18 rows not rising all the way."""
    return _real("Enginetorque .dss")["Torque.Tables.ACTIVE_TORQUE"].table


@pytest.fixture
def real_maps():
    """The grip-to-throttle maps of the same ECU, one per gear. In %: these
    are ETV maps, not torque requests, whatever the file is called."""
    return _real("torque demand.dss")
