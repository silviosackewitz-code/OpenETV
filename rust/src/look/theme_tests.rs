use super::*;
use std::collections::HashMap;

/// The roles of one block of theme.css, `--name` → value.
fn block(css: &str, selector: &str) -> HashMap<String, String> {
    let start = css
        .find(selector)
        .unwrap_or_else(|| panic!("theme.css has {selector}"));
    let open = start + css[start..].find('{').unwrap() + 1;
    let body = &css[open..open + css[open..].find('}').unwrap()];
    let mut roles = HashMap::new();
    // Comments may hold `:` and `;` too.
    let mut text = String::new();
    let mut rest = body;
    while let Some(open) = rest.find("/*") {
        text.push_str(&rest[..open]);
        rest = &rest[open + rest[open..].find("*/").unwrap() + 2..];
    }
    text.push_str(rest);
    for declaration in text.split(';') {
        if let Some((name, value)) = declaration.split_once(':') {
            let name = name.trim();
            if name.starts_with("--") {
                roles.insert(name.to_string(), value.trim().to_string());
            }
        }
    }
    roles
}

fn parsed(value: &str) -> Color32 {
    super::colour(value).unwrap_or_else(|| panic!("a colour: {value}"))
}

fn roles(p: &Palette) -> [(&'static str, Color32); 11] {
    [
        ("--ink", p.ink),
        ("--ink-2", p.ink_2),
        ("--ink-3", p.ink_3),
        ("--bg", p.bg),
        ("--panel", p.panel),
        ("--line", p.line),
        ("--line-soft", p.line_soft),
        ("--accent", p.accent),
        ("--accent-fill", p.accent_fill),
        ("--on-accent", p.on_accent),
        ("--accent-soft", p.accent_soft),
    ]
}

#[test]
fn the_dark_and_light_roles_equal_theme_css() {
    let css = include_str!("../../assets/look/theme-reference.css");
    let light = block(css, ":root{");
    let dark = block(css, ":root[data-theme=\"dark\"]");
    for (palette, css_roles, name) in [(LIGHT, &light, "light"), (DARK, &dark, "dark")] {
        for (role, ours) in roles(&palette) {
            let theirs = css_roles
                .get(role)
                .unwrap_or_else(|| panic!("theme.css's {name} block has {role}"));
            assert_eq!(ours, parsed(theirs), "{name} {role}: {theirs}");
        }
    }
    for (step, ours) in [
        ("--fs-1", FS_1),
        ("--fs-2", FS_2),
        ("--fs-3", FS_3),
        ("--fs-5", FS_5),
    ] {
        assert_eq!(light[step], format!("{ours}px"), "{step}");
    }
    assert_eq!(light["--r"], format!("{}px", DARK.radius));
    assert_eq!(light["--r-l"], format!("{}px", DARK.radius_l));
}

fn derived(accent: &str, dark: bool, role: &str) -> Color32 {
    parsed(&accent_roles(accent, dark)[role])
}

/// The default accent's roles, derived by the backend, are theme.css's: Dark and Light in
/// it are the constants, and the light theme's band is the dark theme.
#[test]
fn the_default_accent_derives_the_roles_of_theme_css() {
    for (theme, palette) in [(Theme::Dark, DARK), (Theme::Light, LIGHT)] {
        let look = Look {
            theme,
            font: Font::Plex,
            accent: [0xd4, 0, 0],
            size: DEFAULT_SIZE,
        };
        assert_eq!(look.palette(), palette, "{}", theme.name());
    }
    assert_eq!(Look::default().palette(), DARK);
    assert_eq!(LIGHT.band(), DARK);
    assert_eq!(DARK.band(), DARK);
}

/// Another accent colours Dark and Light — derived for their own panels, and for the dark
/// panel on the light theme's band — and leaves Industrial's and Terminal's own.
#[test]
fn an_accent_colours_dark_light_and_the_band_and_not_the_new_themes() {
    let blue = "#0a5fd1";
    let look = |theme| Look {
        theme,
        font: Font::Plex,
        accent: [0x0a, 0x5f, 0xd1],
        size: DEFAULT_SIZE,
    };
    let light = look(Theme::Light).palette();
    assert_eq!(light.accent, derived(blue, false, "--accent"));
    assert_eq!(light.accent_fill, derived(blue, false, "--accent-fill"));
    assert_eq!(light.on_accent, derived(blue, false, "--on-accent"));
    assert_eq!(light.accent_soft, derived(blue, false, "--accent-soft"));
    assert_ne!(light.accent, LIGHT.accent);
    assert_eq!((light.ink, light.panel), (LIGHT.ink, LIGHT.panel));
    let band = light.band();
    assert_eq!(band.accent, derived(blue, true, "--accent"));
    assert_eq!((band.panel, band.dark), (DARK.panel, true));
    let dark = look(Theme::Dark).palette();
    assert_eq!(dark.accent_fill, derived(blue, true, "--accent-fill"));
    assert_eq!(dark.band(), dark);
    assert_eq!(look(Theme::Industrial).palette(), INDUSTRIAL);
    assert_eq!(look(Theme::Terminal).palette(), TERMINAL);
}

/// `[Look]`'s theme: light, dark — the dark theme chosen —, or auto as the system is; dark
/// when the system says nothing. A word the backend would refuse keeps its default.
#[test]
fn auto_follows_the_system_and_dark_shows_the_dark_theme_chosen() {
    let stored = Stored::of(&serde_json::json!({
        "theme": "auto", "dark_theme": "industrial", "font": "jetbrains", "accent": "#0a5fd1",
        "size": 120
    }));
    assert_eq!(
        stored,
        Stored {
            theme: "auto".into(),
            dark_theme: Theme::Industrial,
            font: Font::JetBrains,
            accent: "#0a5fd1".into(),
            size: 120,
        }
    );
    let shown = |stored: &Stored, system| stored.shown(system).theme;
    assert_eq!(shown(&stored, Some(egui::Theme::Light)), Theme::Light);
    assert_eq!(shown(&stored, Some(egui::Theme::Dark)), Theme::Industrial);
    assert_eq!(shown(&stored, None), Theme::Industrial);
    let light = Stored {
        theme: "light".into(),
        ..stored.clone()
    };
    assert_eq!(shown(&light, Some(egui::Theme::Dark)), Theme::Light);
    let dark = Stored {
        theme: "dark".into(),
        dark_theme: Theme::Terminal,
        ..stored.clone()
    };
    assert_eq!(shown(&dark, Some(egui::Theme::Light)), Theme::Terminal);
    assert_eq!(dark.shown(None).font, Font::JetBrains);
    assert_eq!(stored.shown(None).accent, [0x0a, 0x5f, 0xd1]);
    let wrong = Stored::of(
        &serde_json::json!({"theme": "sepia", "dark_theme": "light", "font": 3, "size": 300}),
    );
    assert_eq!(wrong, Stored::default());
    assert_eq!(Stored::default().shown(None), Look::default());
}

/// ⌘+ and ⌘− step through the sizes, from a size between the steps to the next one that
/// way, and stop at the ends; ⌘0 is the default.
#[test]
fn the_size_keys_step_through_the_sizes() {
    assert_eq!(size_step(90, 1), 100);
    assert_eq!(size_step(90, -1), 80);
    assert_eq!(size_step(95, 1), 100);
    assert_eq!(size_step(95, -1), 90);
    assert_eq!(size_step(150, 1), 150);
    assert_eq!(size_step(70, -1), 70);
    assert_eq!(size_step(130, 0), DEFAULT_SIZE);
}

fn luminance(c: Color32) -> f64 {
    let lin = |v: u8| {
        let v = v as f64 / 255.;
        if v <= 0.04045 {
            v / 12.92
        } else {
            ((v + 0.055) / 1.055).powf(2.4)
        }
    };
    0.2126 * lin(c.r()) + 0.7152 * lin(c.g()) + 0.0722 * lin(c.b())
}

fn contrast(a: Color32, b: Color32) -> f64 {
    let (a, b) = (luminance(a), luminance(b));
    (a.max(b) + 0.05) / (a.min(b) + 0.05)
}

/// docs/design.md, "Contrast": every text role 4.5 : 1 to the panel and
/// the window, in every theme — the new ones too; text on the accent's
/// fill to the fill.
#[test]
fn every_theme_keeps_the_contrast_of_text() {
    for theme in Theme::ALL {
        let p = theme.palette();
        for (role, ink) in [
            ("ink", p.ink),
            ("ink-2", p.ink_2),
            ("ink-3", p.ink_3),
            ("accent", p.accent),
        ] {
            for (surface, under) in [("panel", p.panel), ("bg", p.bg)] {
                let ratio = contrast(ink, under);
                assert!(
                    ratio >= 4.5,
                    "{}: {role} on {surface} is {ratio:.2}",
                    theme.name()
                );
            }
        }
        let ratio = contrast(p.on_accent, p.accent_fill);
        assert!(ratio >= 4.5, "{}: on-accent is {ratio:.2}", theme.name());
        let band = p.band();
        assert!(band.dark, "{}: the lap band stays dark", theme.name());
    }
}
