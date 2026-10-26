use openetv::model::*;
fn table(rpm: &[f64], axis: &[f64], values: &[&[f64]]) -> Table {
    Table::new(
        rpm.to_vec(),
        axis.to_vec(),
        values.iter().map(|r| r.to_vec()).collect(),
    )
    .unwrap()
}
#[test]
fn sorting_and_validation() {
    let t = table(&[2000., 1000.], &[100., 0.], &[&[4., 3.], &[2., 1.]]);
    assert_eq!(t.values, vec![vec![1., 2.], vec![3., 4.]]);
    assert!(Table::new(vec![1., 1.], vec![0.], vec![vec![1.], vec![2.]]).is_err());
    assert!(Table::new(vec![1.], vec![0.], vec![vec![f64::NAN]]).is_err());
}
#[test]
fn interpolation_and_dummy_rows() {
    let t = table(
        &[0., 1000., 3000.],
        &[0., 100.],
        &[&[0., 0.], &[-10., 50.], &[-20., 100.]],
    );
    assert_eq!(engine_row(&t, 500.), (vec![-10., 50.], true));
    assert_eq!(engine_row(&t, 2000.), (vec![-15., 75.], false));
    assert_eq!(lookup(&t, 2000., 50.), 30.);
    assert_eq!(lookup(&t, -1., 200.), 0.);
}
#[test]
fn inversion_status_and_threshold() {
    let e = table(
        &[1000., 2000.],
        &[0., 25., 50., 100.],
        &[&[-10., 50., 100., 90.], &[-10., 50., 100., 100.]],
    );
    let r = table(
        &[1000., 2000.],
        &[0., 25., 50., 100.],
        &[&[-20., 20., 95., 100.], &[-20., 20., 95., 100.]],
    );
    let out = calculate(&e, &r, &r.rpm, &r.axis, 1000., 0.3).unwrap();
    assert_eq!(out.table.values[0], vec![0., 12.5, 47.5, 50.]);
    assert_eq!(out.table.values[1][3], 100.);
    assert_eq!(
        out.status[0],
        vec![
            Status::BelowMin,
            Status::Ok,
            Status::NonMonotonic,
            Status::Saturated
        ]
    );
}
#[test]
fn fixes_and_rounding() {
    let t = table(
        &[1000.],
        &[0., 5., 10., 20., 100.],
        &[&[8., 20., 40., 40., 40.]],
    );
    assert_eq!(
        zero_gas_fix(&t, 20.).unwrap().values[0],
        vec![0., 10., 20., 40., 40.]
    );
    assert_eq!(flat_spot_fix(&t).values[0], vec![8., 20., 21.1, 23.2, 40.]);
    let d = table(&[1.], &[0., 1., 2.], &[&[10., 5., 20.]]);
    assert_eq!(monotonic_fix(&d).values[0], vec![10., 10., 20.]);
}
#[test]
fn generation_absolute_and_per_rpm() {
    let e = table(
        &[0., 1000., 2000.],
        &[0., 100.],
        &[&[0., 0.], &[-10., 50.], &[-10., 100.]],
    );
    let a = generate_request(
        &e,
        &[500., 1500.],
        &[0., 50., 100.],
        Shape::Power { n: 1. },
        100.,
        false,
    )
    .unwrap();
    assert_eq!(a.values, vec![vec![0., 50., 50.], vec![0., 50., 75.]]);
    let p = generate_request(
        &e,
        &[1500.],
        &[50., 100.],
        Shape::Power { n: 1. },
        80.,
        true,
    )
    .unwrap();
    assert_eq!(p.values[0], vec![30., 60.]);
    for shape in [
        Shape::Power { n: 1.8 },
        Shape::SCurve {
            center: 60.,
            steepness: 8.,
        },
    ] {
        assert_eq!(shape.value(0.), 0.);
        assert!((shape.value(1.) - 1.).abs() < 1e-14)
    }
}
#[test]
fn reverse_and_single_cells() {
    let e = table(&[1000.], &[0., 50., 100.], &[&[-10., 100., 80.]]);
    let t = table(
        &[500., 1500.],
        &[0., 50., 100.],
        &[&[-5., 25., 120.], &[0., 75., 100.]],
    );
    assert_eq!(
        request_from_etv(&e, &t).unwrap().values,
        vec![vec![-10., 45., 80.], vec![-10., 90., 80.]]
    );
    let one = table(&[1.], &[2.], &[&[3.]]);
    assert_eq!(lookup(&one, 20., 50.), 3.);
    assert!(calculate(&e, &t, &[1., 1.], &[0.], 0., 0.).is_err());
}

#[test]
fn reverse_interpolation_rounds_after_direct_linear_interpolation() {
    let engine =
        openetv::model::Table::new(vec![1000.], vec![0., 100.], vec![vec![0.1, 10.]]).unwrap();
    let etv =
        openetv::model::Table::new(vec![1000.], vec![0., 50., 100.], vec![vec![-1., 15., 101.]])
            .unwrap();
    let request = openetv::model::request_from_etv(&engine, &etv).unwrap();
    assert_eq!(request.values, vec![vec![0.1, 1.59, 10.]]);
    let single = openetv::model::Table::new(vec![1000.], vec![20.], vec![vec![7.]]).unwrap();
    assert_eq!(
        openetv::model::request_from_etv(&single, &etv)
            .unwrap()
            .values,
        vec![vec![7., 7., 7.]]
    );
}
