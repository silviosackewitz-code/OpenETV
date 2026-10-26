//! Shared RaceAnalysis native look, via RaceDash; see assets/look/PROVENANCE.md.
#![allow(dead_code)]
mod accent;
pub mod fonts;
pub mod icons;
pub mod theme;

use eframe::egui::{self, Color32};

pub const SCOPE: Color32 = Color32::from_rgb(11, 14, 19);
pub const CYAN: Color32 = Color32::from_rgb(125, 211, 252);
pub const AMBER: Color32 = Color32::from_rgb(240, 166, 70);

pub fn fonts(ctx: &egui::Context) {
    ctx.set_fonts(fonts::definitions(fonts::Font::Plex));
}

pub fn apply(ctx: &egui::Context, theme: &str, accent: [u8; 3]) {
    configure(ctx, theme, accent, "plex", 90.);
}

/// Size is the shared appearance scale in percent, from 70 to 150.
pub fn configure(ctx: &egui::Context, selected: &str, accent: [u8; 3], font: &str, size: f32) {
    let theme =
        theme::Theme::of_key(selected).unwrap_or_else(|| match ctx.input(|i| i.raw.system_theme) {
            Some(egui::Theme::Light) => theme::Theme::Light,
            _ => theme::Theme::Dark,
        });
    theme::follow(
        ctx,
        theme::Look {
            theme,
            font: fonts::Font::of_key(font).unwrap_or(fonts::Font::Plex),
            accent,
            size: if size.is_finite() {
                size.clamp(70., 150.).round() as u16
            } else {
                theme::DEFAULT_SIZE
            },
        },
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn configured_themes_fonts_and_fallbacks_render() {
        for selected in ["dark", "light", "industrial", "terminal", "system"] {
            for face in fonts::Font::ALL {
                let ctx = egui::Context::default();
                configure(&ctx, selected, [0xd4, 0, 0], face.key(), 100.);
                assert_eq!(theme::applied(&ctx).unwrap().font, face);
                let mut output = ctx.run_ui(egui::RawInput::default(), |ui| {
                    ui.heading("OpenETV");
                    ui.label("Engine torque: 50 Nm");
                    ui.button("Calculate").on_hover_text("Calculate ETV map");
                });
                assert!(
                    !ctx.tessellate(output.shapes, output.pixels_per_point)
                        .is_empty()
                );
                output.textures_delta.clear();
            }
        }
    }
}
