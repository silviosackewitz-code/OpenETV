//! Lucide's icons from rust/ui-assets/icons/lucide.ttf (lifted from ui-preview), and the icon
//! button:
//! the icon shows, the word stays the button's name — for a screen reader,
//! for a test that finds it, and at the head of its hover text
//! (docs/design.md, "Tooltips": the name, a dash and one short line).

use crate::look::fonts;
use eframe::egui::{
    self, Button, Color32, FontFamily, FontId, IntoAtoms, Response, RichText, Stroke, Ui,
    WidgetInfo, WidgetType, vec2,
};

/// The generated table; `ALL` serves the test.
#[allow(dead_code)]
mod lucide {
    include!("../../assets/look/icons/lucide.rs");
}
pub use lucide::*;

/// The icon's size on a button: the app's 14 px icons beside a 12 px word,
/// a little larger here where the icon stands alone.
pub const SIZE: f32 = 16.;

pub fn font(size: f32) -> FontId {
    FontId::new(size, FontFamily::Name(fonts::ICONS.into()))
}

pub fn text(icon: char) -> RichText {
    RichText::new(icon.to_string()).font(font(SIZE))
}

/// A square button with *icon* alone.
pub fn button(ui: &Ui, icon: char) -> Button<'static> {
    let side = ui.spacing().interact_size.y;
    Button::new(text(icon)).min_size(vec2(side, side))
}

/// The filled button of the view's one main action, *icon* in *ink* on *fill*.
pub fn primary(ui: &Ui, icon: char, fill: Color32, ink: Color32) -> Button<'static> {
    let side = ui.spacing().interact_size.y;
    Button::new(text(icon).color(ink))
        .fill(fill)
        .stroke(Stroke::new(1., fill))
        .min_size(vec2(side, side))
}

/// A toggle, or a choice of a segmented row, whose size does not follow the pointer. egui draws
/// an unselected one bare and frames it only while hovered, and the bare form is narrower by the
/// frame's stroke: a hover moved its neighbours and could tip a bar over its overflow. Framed at
/// rest too, the frame invisible, a hover changes how it looks and never where things stand.
pub fn steady<'a>(ui: &mut Ui, selected: bool, atoms: impl IntoAtoms<'a>) -> Response {
    // The style changed for this one widget, no child Ui: it stands in its row as before.
    let style = ui.style().clone();
    if !selected {
        let rest = &mut ui.visuals_mut().widgets.inactive;
        rest.bg_fill = Color32::TRANSPARENT;
        rest.weak_bg_fill = Color32::TRANSPARENT;
        rest.bg_stroke.color = Color32::TRANSPARENT;
    }
    let response = ui.add(Button::selectable(selected, atoms).frame_when_inactive(true));
    ui.set_style(style);
    response
}

/// Draws *add* in a part of *ui* whose ids derive from *id* alone, not from how many widgets
/// came before it: a bar's end keeps its buttons' ids when the row before it reflows, so a press
/// and its release meet the same button. In a row the part is one row high; in a wrapping row it
/// goes to the next line when it no longer fits, as a widget of its own would.
pub fn pinned<R>(ui: &mut Ui, id: egui::Id, add: impl FnOnce(&mut Ui) -> R) -> R {
    let seen = id.with("pinned-width");
    let horizontal = ui.layout().is_horizontal();
    if horizontal && ui.layout().main_wrap {
        let width: f32 = ui.data(|d| d.get_temp(seen)).unwrap_or(0.);
        if width > ui.available_size_before_wrap().x {
            ui.end_row();
        }
    }
    let mut room = ui.available_rect_before_wrap();
    if horizontal {
        room.max.y = room.min.y + room.height().min(ui.spacing().interact_size.y);
    }
    let shown = ui.scope_builder(egui::UiBuilder::new().id(id).max_rect(room), add);
    ui.data_mut(|d| d.insert_temp(seen, shown.response.rect.width()));
    shown.inner
}

/// Names *response* by *word*: its accessible name, and its hover text
/// `word — hint`. *pressed* is the state of a toggle, None for a button.
pub fn named(response: Response, word: &str, hint: &str, pressed: Option<bool>) -> Response {
    let enabled = response.enabled();
    response.widget_info(|| match pressed {
        Some(on) => WidgetInfo::selected(WidgetType::Button, enabled, on, word),
        None => WidgetInfo::labeled(WidgetType::Button, enabled, word),
    });
    let tip = format!("{word} — {hint}");
    if enabled {
        response.on_hover_text(tip)
    } else {
        response.on_disabled_hover_text(tip)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::look::fonts::Font;
    use eframe::egui;

    /// The width of a selectable *at rest* and with the pointer on it, under the app's rule that a
    /// widget at rest has a 1-point frame and none grows when hovered (theme.rs).
    fn rest_and_hovered(add: fn(&mut Ui) -> Response) -> (f32, f32) {
        let ctx = egui::Context::default();
        ctx.all_styles_mut(|s| {
            let w = &mut s.visuals.widgets;
            w.inactive.bg_stroke = Stroke::new(1., Color32::GRAY);
            for state in [&mut w.inactive, &mut w.hovered, &mut w.active] {
                state.expansion = 0.;
            }
        });
        let frame = |pointer: Option<egui::Pos2>| {
            let mut input = egui::RawInput {
                screen_rect: Some(egui::Rect::from_min_size(
                    egui::Pos2::ZERO,
                    vec2(400., 200.),
                )),
                ..Default::default()
            };
            input.events.extend(pointer.map(egui::Event::PointerMoved));
            let mut rect = egui::Rect::NOTHING;
            let mut output = ctx.run_ui(input, |ui| rect = add(ui).rect);
            output.textures_delta.clear();
            rect
        };
        let rest = frame(None);
        frame(Some(rest.center()));
        let hovered = frame(Some(rest.center()));
        (rest.width(), hovered.width())
    }

    /// A bare selectable grows by its frame's stroke under the pointer (what tipped the analyzer
    /// bar over its overflow); the steady toggle keeps its width.
    #[test]
    fn a_steady_toggle_keeps_its_width_under_the_pointer() {
        let (rest, hovered) = rest_and_hovered(|ui| ui.selectable_label(false, "Loop"));
        assert!((hovered - rest - 2.).abs() < 0.01, "{rest} → {hovered}");
        let (rest, hovered) = rest_and_hovered(|ui| steady(ui, false, "Loop"));
        assert!((hovered - rest).abs() < 0.01, "{rest} → {hovered}");
        let (rest, hovered) = rest_and_hovered(|ui| steady(ui, true, "Loop"));
        assert!((hovered - rest).abs() < 0.01, "{rest} → {hovered}");
    }

    /// Every icon of the table has its glyph in lucide.ttf: the icons family
    /// holds that font alone, so egui's replacement glyph means "missing".
    #[test]
    fn every_icon_has_a_glyph_in_lucide_ttf() {
        assert_eq!(LUCIDE_VERSION, "0.441.0", "the version icons.js names");
        let ctx = egui::Context::default();
        ctx.set_fonts(fonts::definitions(Font::Plex));
        let mut output = ctx.run_ui(egui::RawInput::default(), |_| {});
        output.textures_delta.clear();
        let mut names = std::collections::HashSet::new();
        for (name, lucide, glyph) in ALL {
            assert!(names.insert(*name), "{name} twice");
            let found = ctx.fonts_mut(|f| f.has_glyph(&font(SIZE), *glyph));
            assert!(
                found,
                "{name} ({lucide}, U+{:04X}) is not in lucide.ttf",
                *glyph as u32
            );
        }
        // The check can fail: a codepoint outside the font is missing.
        assert!(!ctx.fonts_mut(|f| f.has_glyph(&font(SIZE), 'A')));
    }
}
