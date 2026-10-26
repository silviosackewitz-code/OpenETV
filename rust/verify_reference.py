#!/usr/bin/env python3
"""Compare native --batch JSON with the existing Python numerical implementation.
Usage: python verify_reference.py --reference /path/to/OpenETV --binary target/debug/openetv-native
Reads reference code/sample CSV only at runtime; never copies private reference assets.
"""
import argparse
import importlib
import json
import random
import subprocess
import sys
from pathlib import Path


def main():
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument('--reference', type=Path, required=True)
    p.add_argument('--binary', type=Path, required=True)
    args = p.parse_args()
    sys.path.insert(0, str(args.reference / 'tools'))
    api = importlib.import_module('etvlib.api')
    core = importlib.import_module('etvlib.core')
    tables = importlib.import_module('etvlib.tables')
    dss = importlib.import_module('etvlib.dss')
    rng = random.Random(20261026)
    cases = []
    for _ in range(50):
        engine = {'rpm': [0, 1000, 2500, 6000], 'axis': [0, 5, 20, 60, 100],
                  'values': [[0]*5] + [[rng.uniform(-30, 5)] + [rng.uniform(10, 150) for _ in range(4)] for _ in range(3)]}
        rpm, grip = [0, 500, 1000, 1800, 5000, 7000], [0, 2.5, 10, 33, 75, 100]
        request = {'rpm': [500, 3000, 7000], 'axis': [0, 25, 100],
                   'values': [[rng.uniform(-30, 160) for _ in range(3)] for _ in range(3)]}
        cases.append(dict(op='calculate', engine=engine, request=request, rpm=rpm, grip=grip, rpm_threshold=2500, tolerance=0.3))
        for per_rpm in (False, True):
            for shape in ({'kind': 'power', 'n': 1.8}, {'kind': 's_curve', 'center': 60, 'steepness': 8}):
                cases.append(dict(op='generate', engine=engine, rpm=rpm, grip=grip, shape=shape, max_percent=83, per_rpm=per_rpm))
        etv = {'rpm': [1000, 4000], 'axis': [0, 5, 10, 20, 50, 100], 'values': [[10, 25, 25, 80, 80, 80], [5, 5, 20, 40, 30, 90]]}
        for fix in ('zero_gas', 'flat_spots', 'dips'):
            cases.append(dict(op='postprocess', table=etv, fix=fix, ramp_to=20))
        cases.append(dict(op='reverse', engine=engine, table=etv))
    fixture_count = 0
    original_count = 0

    def exercise_engine(engine, request, etv_maps=()):
        nonlocal fixture_count
        fixture_count += 1
        e, req = tables.to_json(engine), tables.to_json(request)
        rpm = sorted(set(engine.rpm.tolist() + request.rpm.tolist()))
        grip = request.axis.tolist()
        thresholds = [float(engine.rpm[0]), float(engine.rpm[len(engine.rpm)//2]), float(engine.rpm[-1])]
        for tolerance in (0.0, 0.3, 2.0):
            for threshold in thresholds:
                calculation = dict(op='calculate', engine=e, request=req, rpm=rpm, grip=grip,
                                   rpm_threshold=threshold, tolerance=tolerance)
                cases.append(calculation)
                result = api.calculate(calculation)['table']
                for fix in ('zero_gas', 'flat_spots', 'dips'):
                    cases.append(dict(op='postprocess', table=result, fix=fix, ramp_to=20))
        shapes = [dict(kind='power', n=n) for n in (0.6, 1.0, 1.8, 2.7)]
        shapes.append(dict(kind='s_curve', center=60, steepness=8))
        for shape in shapes:
            for per_rpm in (False, True):
                for percent in (0, 83, 100):
                    generation = dict(op='generate', engine=e, rpm=rpm, grip=grip, shape=shape,
                                      max_percent=percent, per_rpm=per_rpm)
                    cases.append(generation)
                    generated = api.generate(generation)['table']
                    cases.append(dict(op='calculate', engine=e, request=generated, rpm=rpm, grip=grip,
                                      rpm_threshold=thresholds[1], tolerance=0.3))
        for etv in etv_maps:
            t = tables.to_json(etv)
            cases.append(dict(op='reverse', engine=e, table=t))
            for ramp in (0, 20, 100):
                cases.append(dict(op='postprocess', table=t, fix='zero_gas', ramp_to=ramp))
            for fix in ('flat_spots', 'dips'):
                cases.append(dict(op='postprocess', table=t, fix=fix))

    sample_dir = args.reference / 'sample_data'
    sample_engine = tables.read_csv((sample_dir / 'engine_torque_map.csv').read_text(encoding='utf-8'))
    sample_request = tables.read_csv((sample_dir / 'demand_map.csv').read_text(encoding='utf-8'))
    exercise_engine(sample_engine, sample_request)
    for name in ('engine_torque_map.csv', 'demand_map.csv'):
        cases.append(dict(op='parse', path=str((sample_dir / name).resolve())))
    private_dir = args.reference / '.reference'
    engine_path, demand_path = private_dir / 'Enginetorque .dss', private_dir / 'torque demand.dss'
    if engine_path.exists():
        original_engine = dss.parse_dss(engine_path.read_text(encoding='utf-8'))['Torque.Tables.ACTIVE_TORQUE'].table
        original_count += 1
        cases.append(dict(op='parse', path=str(engine_path.resolve())))
        maps = []
        if demand_path.exists():
            original_maps = dss.parse_dss(demand_path.read_text(encoding='utf-8'))
            original_count += 1
            cases.append(dict(op='parse', path=str(demand_path.resolve())))
            for found in original_maps.values():
                assert found.unit == '%', 'Original demand export must be recognized as throttle.'
                try:
                    dss.require_torque(found, 'torque request')
                except dss.UnitError:
                    pass
                else:
                    raise AssertionError('Original throttle export was accepted as torque.')
                maps.append(found.table)
        exercise_engine(original_engine, sample_request, maps)

    expected = []
    for c in cases:
        if c['op'] == 'parse':
            path = Path(c['path'])
            if path.suffix == '.dss':
                imported = []
                for found in dss.parse_dss(path.read_text(encoding='utf-8')).values():
                    unit = found.unit or ''
                    imported.append(dict(path=found.path, unit=unit, table=tables.to_json(found.table),
                                         rpm_path=found.rpm_path or '', rpm_unit=found.rpm_unit,
                                         other_path=found.other_path or '', other_unit=found.other_unit,
                                         torque_allowed=not unit.strip() or unit.strip().lower() == 'nm'))
            else:
                imported = [dict(path=path.name, unit='', table=tables.to_json(tables.read_csv(path.read_text(encoding='utf-8'))),
                                 rpm_path='', rpm_unit='', other_path='', other_unit='', torque_allowed=True)]
            expected.append(dict(tables=imported))
        elif c['op'] != 'reverse':
            expected.append(getattr(api, c['op'])(c))
        else:
            e, t = tables.from_json(c['engine'], 'engine'), tables.from_json(c['table'], 'ETV')
            import numpy as np
            values = []
            for rpm, row in zip(t.rpm, t.values):
                torque, _ = core.engine_row(e, rpm)
                values.append(np.round(np.interp(row, e.axis, torque), 2).tolist())
            expected.append({'table': {'rpm': t.rpm.tolist(), 'axis': t.axis.tolist(), 'values': values}})
    # Keep each invocation below the native batch input limit; private values stay in pipes.
    for start in range(0, len(cases), 50):
        chunk = cases[start:start+50]
        run = subprocess.run([str(args.binary.resolve()), '--batch'], input=json.dumps(chunk),
                             text=True, capture_output=True, check=True)
        actual = json.loads(run.stdout)
        assert len(actual) == len(chunk), 'Native batch returned the wrong result count.'
        for offset, (want, got) in enumerate(zip(expected[start:start+50], actual)):
            for key in ('table', 'status', 'outside', 'changed', 'tables'):
                if key in want:
                    # Never print original calibration numbers in failure diagnostics.
                    assert got.get(key) == want[key], f'case {start+offset} ({chunk[offset]["op"]}) differs in {key}'
    print(f'{len(cases)} native/reference cases match exactly; {fixture_count} engine fixtures; '
          f'{original_count} optional original ECU files included.')


if __name__ == '__main__':
    main()
