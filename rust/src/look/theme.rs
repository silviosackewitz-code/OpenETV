//! RaceAnalysis native themes, fonts, accent roles and sizing, copied for RaceDash.
//! Source and adaptations: docs/native-look-provenance.md.

use super::accent::{DEFAULT_ACCENT, accent_roles};
use crate::look::fonts::{self, Font};
use eframe::egui::{self, Color32, CornerRadius, FontFamily, FontId, Id, Stroke, TextStyle, vec2};
use serde_json::Value;

const fn hex(rgb: u32) -> Color32 {
    Color32::from_rgb((rgb >> 16) as u8, (rgb >> 8) as u8, rgb as u8)
}

/// `rgba(r,g,b,alpha)` of CSS, premultiplied as CSS blends it.
pub const fn wash(rgb: u32, alpha: f32) -> Color32 {
    const fn part(rgb: u32, shift: u32, alpha: f32) -> u8 {
        (((rgb >> shift) & 0xff) as f32 * alpha + 0.5) as u8
    }
    Color32::from_rgba_premultiplied(
        part(rgb, 16, alpha),
        part(rgb, 8, alpha),
        part(rgb, 0, alpha),
        (alpha * 255. + 0.5) as u8,
    )
}

/// --fs-1 … --fs-5.
pub const FS_1: f32 = 11.;
pub const FS_2: f32 = 12.;
pub const FS_3: f32 = 13.;
pub const FS_5: f32 = 20.;

/// A colour of theme.css or of the backend's accent roles: `#rrggbb`, or `rgba(r,g,b,alpha)`
/// premultiplied as CSS blends it.
pub fn colour(css: &str) -> Option<Color32> {
    let css = css.trim();
    if let Some(digits) = css.strip_prefix('#') {
        return (digits.len() == 6)
            .then(|| u32::from_str_radix(digits, 16).ok())
            .flatten()
            .map(hex);
    }
    let parts: Vec<f32> = css
        .strip_prefix("rgba(")?
        .strip_suffix(')')?
        .split(',')
        .map(|part| part.trim().parse().ok())
        .collect::<Option<_>>()?;
    let [r, g, b, alpha] = parts[..] else {
        return None;
    };
    Some(wash(
        ((r as u32) << 16) | ((g as u32) << 8) | b as u32,
        alpha,
    ))
}

/// A theme's roles (docs/design.md, "Roles") and the shape of its controls.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Palette {
    pub ink: Color32,
    pub ink_2: Color32,
    pub ink_3: Color32,
    pub bg: Color32,
    pub panel: Color32,
    pub line: Color32,
    pub line_soft: Color32,
    pub accent: Color32,
    pub accent_fill: Color32,
    pub on_accent: Color32,
    pub accent_soft: Color32,
    /// The accent's roles on the dark panel — accent, fill, on-accent, soft —, for the lap band
    /// and the small map, which lie on the dark theme's panel in a light theme too.
    pub dark_accent: [Color32; 4],
    pub dark: bool,
    /// --r and --r-l: buttons and panels; dialogs and cards.
    pub radius: u8,
    pub radius_l: u8,
    /// Smaller paddings and rows.
    pub dense: bool,
}

/// The dark theme's accent roles of the default accent (theme.css).
const DARK_ACCENT: [Color32; 4] = [
    hex(0xff2222),
    hex(0xd40000),
    hex(0xffffff),
    wash(0xff2222, 0.16),
];

pub const DARK: Palette = Palette {
    ink: hex(0xe6eaf1),
    ink_2: hex(0x9aa6b8),
    ink_3: hex(0x798595),
    bg: hex(0x0f1319),
    panel: hex(0x161b23),
    line: hex(0x2a323e),
    line_soft: hex(0x212934),
    accent: hex(0xff2222),
    accent_fill: hex(0xd40000),
    on_accent: hex(0xffffff),
    accent_soft: wash(0xff2222, 0.16),
    dark_accent: DARK_ACCENT,
    dark: true,
    radius: 5,
    radius_l: 8,
    dense: false,
};

pub const LIGHT: Palette = Palette {
    ink: hex(0x141922),
    ink_2: hex(0x4a5568),
    ink_3: hex(0x626e81),
    bg: hex(0xeef0f4),
    panel: hex(0xffffff),
    line: hex(0xd3d8e2),
    line_soft: hex(0xe4e8ef),
    accent: hex(0xd40000),
    accent_fill: hex(0xd40000),
    on_accent: hex(0xffffff),
    accent_soft: wash(0xd40000, 0.14),
    dark_accent: DARK_ACCENT,
    dark: false,
    radius: 5,
    radius_l: 8,
    dense: false,
};

/// Graphite and steel: frames that show, square corners, dense rows, and
/// one signal colour — safety orange, with dark ink on its fill.
pub const INDUSTRIAL: Palette = Palette {
    ink: hex(0xeceef0),
    ink_2: hex(0xaab1b9),
    ink_3: hex(0x8a929c),
    bg: hex(0x16181b),
    panel: hex(0x202327),
    line: hex(0x4b525b),
    line_soft: hex(0x2c3035),
    accent: hex(0xff8c1a),
    accent_fill: hex(0xff7a00),
    on_accent: hex(0x111214),
    accent_soft: wash(0xff8c1a, 0.16),
    dark_accent: [
        hex(0xff8c1a),
        hex(0xff7a00),
        hex(0x111214),
        wash(0xff8c1a, 0.16),
    ],
    dark: true,
    radius: 0,
    radius_l: 0,
    dense: true,
};

/// Omarchy-like: the Tokyo Night palette, no rounding; with the terminal
/// font when chosen (`Look::choose_theme`). Its comment grey (#565f89) is
/// too faint for captions: --ink-3 is lifted to 4.5 : 1.
pub const TERMINAL: Palette = Palette {
    ink: hex(0xc0caf5),
    ink_2: hex(0xa9b1d6),
    ink_3: hex(0x868fba),
    bg: hex(0x1a1b26),
    panel: hex(0x1f2335),
    line: hex(0x3b4261),
    line_soft: hex(0x292e42),
    accent: hex(0x7aa2f7),
    accent_fill: hex(0x7aa2f7),
    on_accent: hex(0x1a1b26),
    accent_soft: wash(0x7aa2f7, 0.16),
    dark_accent: [
        hex(0x7aa2f7),
        hex(0x7aa2f7),
        hex(0x1a1b26),
        wash(0x7aa2f7, 0.16),
    ],
    dark: true,
    radius: 0,
    radius_l: 0,
    dense: false,
};

impl Palette {
    /// The palette the lap band and the small map lie on: the theme itself,
    /// or for the light theme the dark one with the accent's dark roles — they
    /// stay dark in every theme.
    pub fn band(&self) -> Palette {
        if self.dark {
            return *self;
        }
        let [accent, accent_fill, on_accent, accent_soft] = self.dark_accent;
        Palette {
            accent,
            accent_fill,
            on_accent,
            accent_soft,
            ..DARK
        }
    }

    /// This palette with the accent roles the backend derives from *accent* (`#rrggbb`) — for
    /// its own surfaces and for the dark panel.
    fn with_accent(self, accent: &str) -> Palette {
        let roles = |dark| -> [Color32; 4] {
            let roles = accent_roles(accent, dark);
            ["--accent", "--accent-fill", "--on-accent", "--accent-soft"]
                .map(|role| colour(&roles[role]).expect("the backend's colour"))
        };
        let [accent, accent_fill, on_accent, accent_soft] = roles(self.dark);
        Palette {
            accent,
            accent_fill,
            on_accent,
            accent_soft,
            dark_accent: roles(true),
            ..self
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Theme {
    Dark,
    Light,
    Industrial,
    Terminal,
}

impl Theme {
    pub const ALL: [Theme; 4] = [
        Theme::Dark,
        Theme::Light,
        Theme::Industrial,
        Theme::Terminal,
    ];

    pub fn name(self) -> &'static str {
        match self {
            Theme::Dark => "Dark",
            Theme::Light => "Light",
            Theme::Industrial => "Industrial",
            Theme::Terminal => "Terminal",
        }
    }

    pub fn palette(self) -> Palette {
        match self {
            Theme::Dark => DARK,
            Theme::Light => LIGHT,
            Theme::Industrial => INDUSTRIAL,
            Theme::Terminal => TERMINAL,
        }
    }

    /// Its word in `[Look]` (`theme = light`, `dark_theme = industrial`).
    pub fn key(self) -> &'static str {
        match self {
            Theme::Dark => "dark",
            Theme::Light => "light",
            Theme::Industrial => "industrial",
            Theme::Terminal => "terminal",
        }
    }

    pub fn of_key(key: &str) -> Option<Theme> {
        Theme::ALL.into_iter().find(|theme| theme.key() == key)
    }

    /// The accent chosen under Appearance is the accent of today's two themes; Industrial and
    /// Terminal keep their signal colour.
    pub fn takes_accent(self) -> bool {
        matches!(self, Theme::Dark | Theme::Light)
    }
}

/// The look as `[Look]` keeps it (the preferences' `look`, read and saved through
/// `/api/prefs`'s request functions): `theme` light, dark or auto (following the system), the
/// theme shown when dark (`dark_theme`), the font and the accent.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Stored {
    /// "auto", "light" or "dark".
    pub theme: String,
    pub dark_theme: Theme,
    pub font: Font,
    /// `#rrggbb`.
    pub accent: String,
    /// The size of everything, in per cent ([`SIZES`]).
    pub size: u16,
}

/// The sizes Settings… → Appearance and ⌘+ ⌘− step through, in per cent: 70 to 150 in steps of
/// ten. Below 70 the smallest text (11 points) gets too small to read on a screen without
/// Retina; above 150 a window of a laptop's size keeps too little room. egui itself would zoom
/// from 20 % to 500 %.
pub const SIZES: [u16; 9] = [70, 80, 90, 100, 110, 120, 130, 140, 150];
/// The size a library starts with and ⌘0 goes back to: the native app as dense as the Electron
/// app at its normal size (measured on the manual's pictures, 2026-10-19: at 100 % the native
/// text is 9–16 % wider and its rows 8 % taller than the Electron app's).
pub const DEFAULT_SIZE: u16 = 90;

/// The size one step from *size* in *direction* (+1 larger, -1 smaller), [`DEFAULT_SIZE`] for
/// 0; a size between the steps goes to the next step that way.
pub fn size_step(size: u16, direction: i32) -> u16 {
    match direction {
        0 => DEFAULT_SIZE,
        d if d > 0 => SIZES
            .iter()
            .copied()
            .find(|s| *s > size)
            .unwrap_or(SIZES[SIZES.len() - 1]),
        _ => SIZES
            .iter()
            .rev()
            .copied()
            .find(|s| *s < size)
            .unwrap_or(SIZES[0]),
    }
}

impl Default for Stored {
    /// `[Look]`'s defaults: today's look.
    fn default() -> Self {
        Self {
            theme: "auto".into(),
            dark_theme: Theme::Dark,
            font: Font::Plex,
            accent: DEFAULT_ACCENT.into(),
            size: DEFAULT_SIZE,
        }
    }
}

impl Stored {
    /// The preferences' `look` (validated by the backend); a key it lacks keeps its default.
    pub fn of(look: &Value) -> Self {
        let word = |key: &str| look[key].as_str().unwrap_or_default();
        let default = Self::default();
        Self {
            theme: match word("theme") {
                theme @ ("auto" | "light" | "dark") => theme.into(),
                _ => default.theme,
            },
            dark_theme: Theme::of_key(word("dark_theme"))
                .filter(|theme| theme.palette().dark)
                .unwrap_or(default.dark_theme),
            font: Font::of_key(word("font")).unwrap_or(default.font),
            accent: Some(word("accent"))
                .filter(|accent| colour(accent).is_some())
                .map_or(default.accent, str::to_owned),
            size: look["size"]
                .as_u64()
                .and_then(|size| u16::try_from(size).ok())
                .filter(|size| (SIZES[0]..=SIZES[SIZES.len() - 1]).contains(size))
                .unwrap_or(default.size),
        }
    }

    /// The theme shown: Light, or the dark theme — for `auto` as *system* is, dark when the
    /// system says nothing.
    pub fn theme_shown(&self, system: Option<egui::Theme>) -> Theme {
        match (self.theme.as_str(), system) {
            ("light", _) | ("auto", Some(egui::Theme::Light)) => Theme::Light,
            _ => self.dark_theme,
        }
    }

    /// The look to draw with when the system is *system*.
    pub fn shown(&self, system: Option<egui::Theme>) -> Look {
        Look {
            theme: self.theme_shown(system),
            font: self.font,
            accent: colour(&self.accent).map_or([0xd4, 0, 0], |c| [c.r(), c.g(), c.b()]),
            size: self.size,
        }
    }
}

/// The look drawn: the theme, the font, the accent (its red, green and blue) and the size in
/// per cent.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Look {
    pub theme: Theme,
    pub font: Font,
    pub accent: [u8; 3],
    pub size: u16,
}

impl Default for Look {
    /// Dark with IBM Plex in the default accent.
    fn default() -> Self {
        Stored::default().shown(None)
    }
}

impl Look {
    /// The theme's palette, in the accent chosen when the theme takes one.
    pub fn palette(&self) -> Palette {
        let palette = self.theme.palette();
        if !self.theme.takes_accent() {
            return palette;
        }
        let [r, g, b] = self.accent;
        palette.with_accent(&format!("#{r:02x}{g:02x}{b:02x}"))
    }
}

const PALETTE: &str = "look-palette";
const FONT: &str = "look-font";
const LOOK: &str = "look-applied";

/// The palette of the look applied last.
pub fn colours(ctx: &egui::Context) -> Palette {
    ctx.data(|d| d.get_temp(Id::new(PALETTE))).unwrap_or(DARK)
}

pub fn mono(size: f32) -> FontId {
    FontId::new(size, FontFamily::Monospace)
}

pub fn sans(size: f32) -> FontId {
    FontId::new(size, FontFamily::Proportional)
}

/// Mono Medium, for the values that matter.
pub fn value(size: f32) -> FontId {
    FontId::new(size, FontFamily::Name(fonts::MONO_MEDIUM.into()))
}

/// SemiBold, for names that lead a block.
pub fn strong(size: f32) -> FontId {
    FontId::new(size, FontFamily::Name(fonts::SEMIBOLD.into()))
}

/// Bold, for the window's heading.
pub fn heading(size: f32) -> FontId {
    FontId::new(size, FontFamily::Name(fonts::BOLD.into()))
}

/// The look applied last, when one was.
pub fn applied(ctx: &egui::Context) -> Option<Look> {
    ctx.data(|d| d.get_temp(Id::new(LOOK)))
}

/// Applies *look* to *ctx* when it is not the look applied last: every window of the app, as
/// they share the context. Whether it was applied.
pub fn follow(ctx: &egui::Context, look: Look) -> bool {
    let changed = applied(ctx) != Some(look);
    if changed {
        apply(ctx, look);
    }
    changed
}

/// Applies *look* to *ctx*: its fonts (only when they changed — egui compares
/// whole font files), its theme's style and its size — egui's zoom factor, one for every
/// window of the context: everything drawn, text, spacing and lettering, grows
/// or shrinks together, which the text styles alone would not do. egui's own ⌘+ ⌘− ⌘0 (free
/// steps of 10 % from 20 % to 500 %, never saved) are switched off: the app's step through
/// [`SIZES`] and are saved by the application. Takes effect with the next
/// frame.
pub fn apply(ctx: &egui::Context, look: Look) {
    ctx.options_mut(|o| o.zoom_with_keyboard = false);
    let zoom = f32::from(look.size) / 100.;
    if ctx.zoom_factor() != zoom {
        ctx.set_zoom_factor(zoom);
    }
    if ctx.data(|d| d.get_temp::<Font>(Id::new(FONT))) != Some(look.font) {
        ctx.set_fonts(fonts::definitions(look.font));
        ctx.data_mut(|d| d.insert_temp(Id::new(FONT), look.font));
    }
    ctx.data_mut(|d| d.insert_temp(Id::new(LOOK), look));
    let c = look.palette();
    ctx.data_mut(|d| d.insert_temp(Id::new(PALETTE), c));
    let theme = if c.dark {
        egui::Theme::Dark
    } else {
        egui::Theme::Light
    };
    ctx.set_theme(theme);
    let mut style = (*ctx.style_of(theme)).clone();
    let v = &mut style.visuals;
    v.dark_mode = c.dark;
    v.override_text_color = Some(c.ink);
    v.panel_fill = c.bg;
    v.window_fill = c.panel;
    v.window_stroke = Stroke::new(1., c.line);
    v.window_corner_radius = CornerRadius::same(c.radius_l);
    v.menu_corner_radius = CornerRadius::same(c.radius_l);
    v.extreme_bg_color = c.bg;
    v.faint_bg_color = c.line_soft;
    v.hyperlink_color = c.accent;
    // A chosen row or choice is neutral, never the accent (docs/design.md).
    v.selection.bg_fill = c.line;
    v.selection.stroke = Stroke::new(1., c.ink);
    let radius = CornerRadius::same(c.radius);
    for (w, fill, stroke) in [
        (&mut v.widgets.noninteractive, c.panel, c.line_soft),
        (&mut v.widgets.inactive, c.panel, c.line),
        (&mut v.widgets.hovered, c.panel, c.ink_3),
        (&mut v.widgets.active, c.line_soft, c.ink_3),
        (&mut v.widgets.open, c.panel, c.ink_3),
    ] {
        w.bg_fill = fill;
        w.weak_bg_fill = fill;
        w.bg_stroke = Stroke::new(1., stroke);
        w.fg_stroke = Stroke::new(1., c.ink);
        w.corner_radius = radius;
        w.expansion = 0.;
    }
    v.widgets.noninteractive.fg_stroke = Stroke::new(1., c.ink_2);
    // `.btn:disabled` in theme.css: dimmed to 45 %.
    v.disabled_alpha = 0.45;
    if c.dense {
        style.spacing.button_padding = vec2(8., 4.);
        style.spacing.item_spacing = vec2(6., 4.);
        style.spacing.interact_size.y = 24.;
    } else {
        style.spacing.button_padding = vec2(12., 6.);
        style.spacing.item_spacing = vec2(8., 6.);
        style.spacing.interact_size.y = 28.;
    }
    style.text_styles.insert(TextStyle::Body, sans(FS_3));
    style.text_styles.insert(
        TextStyle::Button,
        FontId::new(FS_2 + 1., FontFamily::Name(fonts::MEDIUM.into())),
    );
    style.text_styles.insert(TextStyle::Small, sans(FS_1));
    style.text_styles.insert(TextStyle::Monospace, mono(FS_3));
    style
        .text_styles
        .insert(TextStyle::Heading, heading(FS_5 + 6.));
    ctx.set_global_style(style);
}

/// Semantic error colour from RaceAnalysis.
pub fn danger(c: &Palette) -> Color32 {
    if c.dark { hex(0xff6b5e) } else { hex(0xd02b1f) }
}
/// Semantic success colour from RaceAnalysis.
pub fn ok(c: &Palette) -> Color32 {
    if c.dark { hex(0x3ecf8e) } else { hex(0x187b4e) }
}

#[cfg(test)]
#[path = "theme_tests.rs"]
mod tests;
