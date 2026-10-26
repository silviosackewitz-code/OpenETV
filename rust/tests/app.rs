use eframe::egui;
use egui_kittest::{Harness, kittest::Queryable};
use openetv::{
    app::{App, Session, parse_axis},
    model,
};
#[path = "support/picture.rs"]
mod picture;

#[test]
fn project_roundtrip_validation_and_staleness() {
    let mut s = Session::sample();
    s.calculate().unwrap();
    assert!(s.fresh());
    let dir = tempfile::tempdir().unwrap();
    let file = dir.path().join("test.openetv");
    s.save(&file).unwrap();
    let mut loaded = Session::load(&file).unwrap();
    assert_eq!(loaded.engine, s.engine);
    assert_eq!(
        loaded.result.as_ref().unwrap().table,
        s.result.as_ref().unwrap().table
    );
    assert!(loaded.fresh());
    loaded.calculate().unwrap();
    assert!(loaded.fresh());
    loaded.request.values[0][0] += 1.;
    assert!(!loaded.fresh());
    let saved = std::fs::read(&file).unwrap();
    s.engine.values[0][0] = f64::NAN;
    assert!(s.save(&file).is_err());
    assert_eq!(std::fs::read(&file).unwrap(), saved);
    let mut value: serde_json::Value = serde_json::from_slice(&saved).unwrap();
    value["result"]["status"] = serde_json::json!([]);
    std::fs::write(&file, serde_json::to_vec(&value).unwrap()).unwrap();
    assert!(Session::load(&file).is_err());
    assert_eq!(
        parse_axis("100, 0, 20; 20", true).unwrap(),
        vec![0., 20., 100.]
    );
    assert!(parse_axis("0,NaN", true).is_err());
    assert!(parse_axis("101", true).is_err());
}
#[test]
fn native_workflow_and_pictures() {
    let collector = picture::Collector::default();
    let mut h = Harness::builder()
        .with_size(egui::vec2(1400., 1000.))
        .renderer(collector.clone())
        .build_ui_state(|ui, app: &mut App| app.show(ui), App::new());
    h.run();
    h.get_by_label("3   Calculation").click();
    h.run();
    assert_eq!(h.state().step, 2);
    h.get_by_label("Calculate ETV map").click();
    h.run();
    assert_eq!(h.state().step, 3);
    assert!(h.state().session.fresh());
    let raw = h.state().session.result.as_ref().unwrap().table.clone();
    h.get_by_label("Zero-gas fix").click();
    h.run();
    let fixed = h.state().session.result.as_ref().unwrap().table.clone();
    assert_ne!(raw, fixed);
    assert_eq!(fixed, model::zero_gas_fix(&raw, 20.).unwrap());
    h.get_by_label("Undo").click();
    h.run();
    assert_eq!(h.state().session.result.as_ref().unwrap().table, raw);
    h.get_by_label("Redo").click();
    h.run();
    assert_eq!(h.state().session.result.as_ref().unwrap().table, fixed);
    h.get_by_label("Reset to calculated").click();
    h.run();
    assert_eq!(h.state().session.result.as_ref().unwrap().table, raw);
    if let Some(dir) = std::env::var_os("OPENETV_PICTURES") {
        for step in 0..5 {
            h.state_mut().step = step;
            h.run();
            picture::picture(
                &h,
                &collector,
                &std::path::PathBuf::from(&dir).join(format!("native-step-{}.png", step + 1)),
                1.,
            );
        }
        h.get_by_label("How it works").click();
        h.run();
        picture::picture(
            &h,
            &collector,
            &std::path::PathBuf::from(dir).join("native-principle.png"),
            1.,
        );
    }
}

#[test]
fn saves_refuse_an_external_change() {
    let s = Session::sample();
    let d = tempfile::tempdir().unwrap();
    let p = d.path().join("p.openetv");
    s.save(&p).unwrap();
    let expected = std::fs::read(&p).unwrap();
    std::fs::write(&p, b"changed elsewhere").unwrap();
    assert!(s.save_checked(&p, &expected).is_err());
    assert_eq!(std::fs::read(&p).unwrap(), b"changed elsewhere");
}

#[test]
fn verified_projects_keep_corrections_but_mismatched_baselines_stay_stale() {
    let mut s = Session::sample();
    s.calculate().unwrap();
    let raw = s.raw.clone().unwrap();
    s.result.as_mut().unwrap().table = model::zero_gas_fix(&raw, 20.).unwrap();
    let d = tempfile::tempdir().unwrap();
    let p = d.path().join("p.openetv");
    s.save(&p).unwrap();
    let restored = Session::load(&p).unwrap();
    assert!(restored.fresh());
    assert_eq!(
        restored.result.unwrap().table,
        s.result.as_ref().unwrap().table
    );
    s.raw.as_mut().unwrap().values[0][0] += 1.;
    s.save(&p).unwrap();
    let stale = Session::load(&p).unwrap();
    assert!(!stale.fresh());
    assert_eq!(stale.result.unwrap().table, s.result.unwrap().table);
}
