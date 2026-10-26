//! The two fonts of the look: IBM Plex (Sans for text, Mono for values) and
//! JetBrains Mono. Lucide's icon font stands in
//! every family right after the chosen font, so an icon can stand in any
//! text; egui's own fonts follow as the fallback for the few signs neither
//! has (★, ▼, ▲). The shared licensed Manual symbol faces follow those fallbacks.

use eframe::egui::{FontData, FontDefinitions, FontFamily};
use std::sync::Arc;

/// The family of the medium weight (button words), the semibold (names that
/// lead a block), the bold (the window's heading), the mono's medium (values
/// that matter: the Values window, the head) and the icons alone.
pub const MEDIUM: &str = "medium";
pub const MONO_MEDIUM: &str = "mono-medium";
pub const SEMIBOLD: &str = "semibold";
pub const BOLD: &str = "bold";
pub const ICONS: &str = "icons";
/// Paddockwerk's wordmark: IBM Plex Sans SemiBold whichever font the look chose
/// (`look::brand`).
pub const BRAND: &str = "brand";

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Font {
    Plex,
    JetBrains,
}

impl Font {
    pub const ALL: [Font; 2] = [Font::Plex, Font::JetBrains];

    pub fn name(self) -> &'static str {
        match self {
            Font::Plex => "IBM Plex",
            Font::JetBrains => "JetBrains Mono",
        }
    }

    /// Its word in `[Look]` (`font = plex`).
    pub fn key(self) -> &'static str {
        match self {
            Font::Plex => "plex",
            Font::JetBrains => "jetbrains",
        }
    }

    pub fn of_key(key: &str) -> Option<Font> {
        Font::ALL.into_iter().find(|font| font.key() == key)
    }
}

macro_rules! asset {
    ($file:literal) => {
        include_bytes!(concat!("../../assets/look/", $file))
    };
}

const FILES: [(&str, &[u8]); 13] = [
    ("plex-sans", asset!("fonts/IBMPlexSans-Regular.ttf")),
    ("plex-sans-medium", asset!("fonts/IBMPlexSans-Medium.ttf")),
    (
        "plex-sans-semibold",
        asset!("fonts/IBMPlexSans-SemiBold.ttf"),
    ),
    ("plex-sans-bold", asset!("fonts/IBMPlexSans-Bold.ttf")),
    ("plex-mono", asset!("fonts/IBMPlexMono-Regular.ttf")),
    ("plex-mono-medium", asset!("fonts/IBMPlexMono-Medium.ttf")),
    ("jetbrains", asset!("fonts/JetBrainsMono-Regular.ttf")),
    ("jetbrains-medium", asset!("fonts/JetBrainsMono-Medium.ttf")),
    (
        "jetbrains-semibold",
        asset!("fonts/JetBrainsMono-SemiBold.ttf"),
    ),
    ("jetbrains-bold", asset!("fonts/JetBrainsMono-Bold.ttf")),
    ("lucide", asset!("icons/lucide.ttf")),
    ("manual-symbols", asset!("fonts/DejaVuSansMono.ttf")),
    ("manual-math", asset!("fonts/NewCMMath-Regular.otf")),
];

/// egui's definitions for *font*: the chosen files first in each family,
/// then the icons, then egui's built-in fonts.
pub fn definitions(font: Font) -> FontDefinitions {
    let mut defs = FontDefinitions::default();
    // egui's text fonts, then Hack for the signs only it has (▼, ▲).
    let mut fallback = defs.families[&FontFamily::Proportional].clone();
    fallback.push("Hack".to_owned());
    fallback.extend(["manual-symbols".to_owned(), "manual-math".to_owned()]);
    for (name, bytes) in FILES {
        defs.font_data
            .insert(name.to_owned(), Arc::new(FontData::from_static(bytes)));
    }
    let [text, medium, semibold, bold, mono, mono_medium] = match font {
        Font::Plex => [
            "plex-sans",
            "plex-sans-medium",
            "plex-sans-semibold",
            "plex-sans-bold",
            "plex-mono",
            "plex-mono-medium",
        ],
        Font::JetBrains => [
            "jetbrains",
            "jetbrains-medium",
            "jetbrains-semibold",
            "jetbrains-bold",
            "jetbrains",
            "jetbrains-medium",
        ],
    };
    let chain = |first: &str| {
        let mut names = vec![first.to_owned(), "lucide".to_owned()];
        names.extend(fallback.iter().cloned());
        names
    };
    defs.families.insert(FontFamily::Proportional, chain(text));
    defs.families.insert(FontFamily::Monospace, chain(mono));
    for (family, first) in [
        (MEDIUM, medium),
        (SEMIBOLD, semibold),
        (BOLD, bold),
        (MONO_MEDIUM, mono_medium),
    ] {
        defs.families
            .insert(FontFamily::Name(family.into()), chain(first));
    }
    defs.families
        .insert(FontFamily::Name(BRAND.into()), chain("plex-sans-semibold"));
    defs.families
        .insert(FontFamily::Name(ICONS.into()), vec!["lucide".to_owned()]);
    defs
}
