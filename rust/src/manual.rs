use eframe::egui::{self, TextFormat, text::LayoutJob};

pub const PAGES: [(&str, &str); 9] = [
    (
        "Build a throttle map",
        include_str!("../help/getting-started.md"),
    ),
    ("Engine torque", include_str!("../help/engine-torque.md")),
    ("Torque request", include_str!("../help/torque-request.md")),
    ("Calculation", include_str!("../help/calculation.md")),
    ("ETV map", include_str!("../help/etv-map.md")),
    ("Export", include_str!("../help/export.md")),
    ("Edit a map", include_str!("../help/editing.md")),
    (
        "Calculation reference",
        include_str!("../help/reference.md"),
    ),
    (
        "Projects and appearance",
        include_str!("../help/projects.md"),
    ),
];

fn text(ui: &mut egui::Ui, line: &str) {
    let mut job = LayoutJob::default();
    for (i, part) in line.split("**").enumerate() {
        job.append(
            part,
            0.,
            TextFormat {
                font_id: if i % 2 == 1 {
                    egui::FontId::new(
                        egui::TextStyle::Body.resolve(ui.style()).size,
                        egui::FontFamily::Name(crate::look::fonts::SEMIBOLD.into()),
                    )
                } else {
                    egui::TextStyle::Body.resolve(ui.style())
                },
                color: if i % 2 == 1 {
                    ui.visuals().strong_text_color()
                } else {
                    ui.visuals().text_color()
                },
                extra_letter_spacing: if i % 2 == 1 { 0.2 } else { 0. },
                ..Default::default()
            },
        );
    }
    ui.add(egui::Label::new(job).wrap());
}

pub fn show(ui: &mut egui::Ui, topic: &mut usize) {
    *topic = (*topic).min(PAGES.len() - 1);
    ui.horizontal(|ui| {
        ui.label("Topic");
        egui::ComboBox::from_id_salt("manual-topic")
            .width(240.)
            .selected_text(PAGES[*topic].0)
            .show_ui(ui, |ui| {
                for (i, (title, _)) in PAGES.iter().enumerate() {
                    ui.selectable_value(topic, i, *title);
                }
            });
        if ui.button("Start here").clicked() {
            *topic = 0;
        }
    });
    ui.separator();
    egui::ScrollArea::vertical()
        .id_salt(("manual", *topic))
        .show(ui, |ui| {
            let mut lines = PAGES[*topic].1.lines().enumerate().peekable();
            while let Some((n, line)) = lines.next() {
                if line.starts_with('|') {
                    let width = ui.available_width() - 16.;
                    egui::Grid::new(("help-table", *topic, n))
                        .num_columns(2)
                        .striped(true)
                        .spacing([16., 10.])
                        .show(ui, |ui| {
                            let mut current = line;
                            loop {
                                if !current.starts_with("|---") {
                                    for (column, cell) in
                                        current.trim_matches('|').split('|').enumerate()
                                    {
                                        ui.vertical(|ui| {
                                            ui.set_width(
                                                width * if column == 0 { 0.32 } else { 0.68 },
                                            );
                                            if current == line {
                                                text(ui, &format!("**{}**", cell.trim()));
                                            } else {
                                                text(ui, cell.trim());
                                            }
                                        });
                                    }
                                    ui.end_row();
                                }
                                if lines.peek().is_some_and(|(_, l)| l.starts_with('|')) {
                                    current = lines.next().unwrap().1;
                                } else {
                                    break;
                                }
                            }
                        });
                } else if let Some(title) = line.strip_prefix("# ") {
                    ui.add_space(10.);
                    ui.heading(title);
                    ui.add_space(8.);
                } else if let Some(title) = line.strip_prefix("## ") {
                    ui.add_space(12.);
                    ui.label(
                        egui::RichText::new(title)
                            .family(egui::FontFamily::Name(crate::look::fonts::SEMIBOLD.into()))
                            .size(16.),
                    );
                    ui.add_space(4.);
                } else if line.is_empty() {
                    ui.add_space(6.);
                } else {
                    text(ui, line);
                }
            }
        });
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn concise_help_pages_render() {
        for (index, (title, body)) in PAGES.iter().enumerate() {
            assert!(body.starts_with(&format!("# {title}\n")));
            assert!(
                body.split_whitespace().count() < 300,
                "{title} has grown too long"
            );
            let ctx = egui::Context::default();
            crate::look::fonts(&ctx);
            let mut topic = index;
            let mut output = ctx.run_ui(egui::RawInput::default(), |ui| show(ui, &mut topic));
            assert!(!output.shapes.is_empty());
            output.textures_delta.clear();
        }
    }
}
