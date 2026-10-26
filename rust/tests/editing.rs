use eframe::egui;
use egui_kittest::{Harness, kittest::Queryable};
use openetv::app::App;
#[test]
fn paste_edit_undo_and_result_axis_protection() {
    let mut h = Harness::builder()
        .with_size(egui::vec2(1400., 1000.))
        .build_ui_state(|ui, app: &mut App| app.show(ui), App::new());
    h.run();
    let original = h.state().session.engine.clone();
    h.get_by_label("Value at 3000 RPM, 0 percent").click();
    h.run();
    h.event(egui::Event::Paste("1\t2\n3\t4".into()));
    h.run();
    assert_eq!(&h.state().session.engine.values[0][0..2], &[1., 2.]);
    assert_eq!(&h.state().session.engine.values[1][0..2], &[3., 4.]);
    h.get_by_label("Undo").click();
    h.run();
    assert_eq!(h.state().session.engine, original);
    h.get_by_label("Value at 3000 RPM, 0 percent").click();
    h.run();
    h.event(egui::Event::Paste("7\tNaN".into()));
    h.run();
    assert_eq!(h.state().session.engine, original);
    assert!(!h.state().message.is_empty());
    h.state_mut().session.calculate().unwrap();
    h.state_mut().step = 3;
    h.run();
    h.get_by_label("Table tools").click();
    h.run();
    assert!(h.query_by_label("+ Column").is_none());
    assert!(h.query_by_label("+ RPM row").is_none());
}
