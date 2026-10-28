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

#[test]
fn contextual_manual_and_result_views() {
    let mut app = App::new();
    app.session.calculate().unwrap();
    app.step = 3;
    let mut h = Harness::builder()
        .with_size(egui::vec2(1400., 1000.))
        .build_ui_state(|ui, app: &mut App| app.show(ui), app);
    h.run();
    let before = h.state().session.result.clone();
    h.get_by_label("Torque comparison").click();
    h.run();
    assert!(h.query_by_label("Cyan: modelled torque · amber: requested torque. This is a model comparison, not a dyno measurement.").is_some());
    h.get_by_label("?").click();
    h.run();
    assert!(h.query_by_label("Corrections").is_some());
    assert!(h.query_by_label("Read the result").is_some());
    assert_eq!(
        serde_json::to_value(before).unwrap(),
        serde_json::to_value(&h.state().session.result).unwrap()
    );
}

#[test]
fn help_pages_and_light_map_pictures() {
    let Some(dir) = std::env::var_os("OPENETV_PICTURES") else {
        return;
    };
    let dir = std::path::PathBuf::from(dir);
    let collector = picture::Collector::default();
    let mut h = Harness::builder()
        .with_size(egui::vec2(780., 760.))
        .renderer(collector.clone())
        .build_ui_state(
            |ui, topic: &mut usize| {
                let ready = openetv::look::theme::applied(ui.ctx()).is_some();
                openetv::look::apply(ui.ctx(), "light", [212, 0, 0]);
                if ready {
                    egui::CentralPanel::default().show(ui, |ui| openetv::manual::show(ui, topic));
                }
            },
            0usize,
        );
    for topic in 0..openetv::manual::PAGES.len() {
        *h.state_mut() = topic;
        h.run();
        picture::picture(&h, &collector, &dir.join(format!("manual-{topic}.png")), 1.);
    }
    let collector = picture::Collector::default();
    let mut h = Harness::builder()
        .with_size(egui::vec2(1100., 700.))
        .renderer(collector.clone())
        .build_ui_state(
            |ui, table: &mut model::Table| {
                openetv::look::apply(ui.ctx(), "light", [212, 0, 0]);
                egui::CentralPanel::default().show(ui, |ui| {
                    openetv::grid::editor(ui, "engine", table, None, false).unwrap();
                });
            },
            Session::sample().engine,
        );
    h.run();
    picture::picture(&h, &collector, &dir.join("light-map.png"), 1.);
}

#[test]
fn breakpoint_controls_add_edit_remove_and_undo() {
    let mut app = App::new();
    app.step = 2;
    let mut h = Harness::builder()
        .with_size(egui::vec2(1400., 1000.))
        .build_ui_state(|ui, app: &mut App| app.show(ui), app);
    h.run();
    let original = h.state().session.settings.clone();
    h.get_by_label("Add RPM").click();
    h.run();
    assert_eq!(
        parse_axis(&h.state().session.settings.rpm, false).unwrap()[0],
        0.
    );
    h.get_by_label("Remove RPM breakpoint 1").click();
    h.run();
    assert_eq!(h.state().session.settings.rpm, original.rpm);
    h.get_by_label("Undo").click();
    h.run();
    assert_eq!(
        parse_axis(&h.state().session.settings.rpm, false).unwrap()[0],
        0.
    );
    h.get_by_label("RPM breakpoint 1").focus();
    h.run();
    h.get_by_label("RPM breakpoint 1").type_text("1000");
    h.key_press(egui::Key::Enter);
    h.run();
    assert_eq!(
        parse_axis(&h.state().session.settings.rpm, false).unwrap()[0],
        1000.
    );
    h.get_by_label("RPM breakpoint 1").focus();
    h.run();
    h.get_by_label("RPM breakpoint 1").type_text("3000");
    h.key_press(egui::Key::Enter);
    h.run();
    assert_eq!(
        parse_axis(&h.state().session.settings.rpm, false).unwrap()[0],
        1000.
    );
    h.get_by_label("New pedal value").focus();
    h.run();
    h.get_by_label("New pedal value").type_text("15");
    h.key_press(egui::Key::Enter);
    h.run();
    h.get_by_label("Add pedal").click();
    h.run();
    let points = parse_axis(&h.state().session.settings.grip, true).unwrap();
    assert_eq!(&points[..4], &[0., 10., 15., 20.]);
    h.get_by_label("Add pedal").click();
    h.run();
    assert_eq!(
        parse_axis(&h.state().session.settings.grip, true).unwrap(),
        points
    );
    h.get_by_label("Calculate ETV map").click();
    h.run();
    assert!(h.state().session.fresh());
    assert_eq!(
        h.state().session.result.as_ref().unwrap().table.axis,
        points
    );
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("breakpoints.openetv");
    h.state().session.save(&path).unwrap();
    assert_eq!(
        Session::load(&path).unwrap().settings,
        h.state().session.settings
    );
    h.get_by_label("2   Torque request").click();
    h.run();
    h.get_by_label("Generate a torque request").click();
    h.run();
    assert!(h.query_by_label("RPM breakpoint 1").is_some());
    h.get_by_label("Generate request").click();
    h.run();
    assert_eq!(h.state().session.request.axis, points);
}

#[test]
fn tall_map_shows_all_rows_before_the_chart() {
    let mut app = App::new();
    app.session.engine = model::Table::new(
        (0..16).map(|i| f64::from(i * 500 + 3000)).collect(),
        vec![0., 50., 100.],
        vec![vec![0., 30., 60.]; 16],
    )
    .unwrap();
    let collector = picture::Collector::default();
    let mut h = Harness::builder()
        .with_size(egui::vec2(1400., 1100.))
        .renderer(collector.clone())
        .build_ui_state(|ui, app: &mut App| app.show(ui), app);
    h.run();
    let first = h.get_by_label("Value at 3000 RPM, 0 percent").rect();
    let last = h.get_by_label("Value at 10500 RPM, 0 percent").rect();
    assert!(last.top() - first.top() > 420.);
    assert!(last.bottom() < 1000.);
    assert!(h.get_by_label("Torque curve").rect().top() > last.bottom());
    h.get_by_label("Value at 10500 RPM, 0 percent").click();
    h.run();
    h.event(egui::Event::Paste("5".into()));
    h.run();
    assert_eq!(h.state().session.engine.values[15][0], 5.);
    if let Some(dir) = std::env::var_os("OPENETV_PICTURES") {
        picture::picture(
            &h,
            &collector,
            &std::path::PathBuf::from(dir).join("tall-map.png"),
            1.,
        );
    }
}
