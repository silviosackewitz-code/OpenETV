//! Pure accent derivation copied from RaceAnalysis, without its session/backend dependency.

pub const DEFAULT_ACCENT: &str = "#d40000";

fn rgb(colour: &str) -> [f64; 3] {
    [1, 3, 5].map(|i| {
        u8::from_str_radix(&colour[i..i + 2], 16).expect("validated colour") as f64 / 255.0
    })
}
fn hex(rgb: [f64; 3]) -> String {
    format!(
        "#{:02x}{:02x}{:02x}",
        (rgb[0].clamp(0.0, 1.0) * 255.0).round_ties_even() as u8,
        (rgb[1].clamp(0.0, 1.0) * 255.0).round_ties_even() as u8,
        (rgb[2].clamp(0.0, 1.0) * 255.0).round_ties_even() as u8
    )
}
fn luminance(colour: &str) -> f64 {
    let [r, g, b] = rgb(colour).map(|c| {
        if c <= 0.03928 {
            c / 12.92
        } else {
            ((c + 0.055) / 1.055).powf(2.4)
        }
    });
    0.2126 * r + 0.7152 * g + 0.0722 * b
}
pub fn contrast(a: &str, b: &str) -> f64 {
    let (a, b) = (luminance(a), luminance(b));
    (a.max(b) + 0.05) / (a.min(b) + 0.05)
}
fn hls(colour: &str) -> [f64; 3] {
    let [r, g, b] = rgb(colour);
    let max = r.max(g).max(b);
    let min = r.min(g).min(b);
    let l = (min + max) / 2.0;
    if min == max {
        return [0.0, l, 0.0];
    }
    let range = max - min;
    let s = if l <= 0.5 {
        range / (max + min)
    } else {
        range / (2.0 - max - min)
    };
    let (rc, gc, bc) = ((max - r) / range, (max - g) / range, (max - b) / range);
    let h = if r == max {
        bc - gc
    } else if g == max {
        2.0 + rc - bc
    } else {
        4.0 + gc - rc
    };
    [(h / 6.0).rem_euclid(1.0), l, s]
}
fn hls_rgb([h, l, s]: [f64; 3]) -> [f64; 3] {
    if s == 0.0 {
        return [l, l, l];
    }
    let m2 = if l <= 0.5 {
        l * (1.0 + s)
    } else {
        l + s - l * s
    };
    let m1 = 2.0 * l - m2;
    let v = |h: f64| {
        let h = h.rem_euclid(1.0);
        if h < 1.0 / 6.0 {
            m1 + (m2 - m1) * h * 6.0
        } else if h < 0.5 {
            m2
        } else if h < 2.0 / 3.0 {
            m1 + (m2 - m1) * (2.0 / 3.0 - h) * 6.0
        } else {
            m1
        }
    };
    [v(h + 1.0 / 3.0), v(h), v(h - 1.0 / 3.0)]
}
fn towards(colour: &str, surfaces: &[&str; 2], wanted: f64, lighter: bool) -> String {
    let [h, mut l, s] = hls(colour);
    let step = if lighter { 0.01 } else { -0.01 };
    let mut result = colour.to_owned();
    while surfaces
        .iter()
        .map(|s| contrast(&result, s))
        .fold(f64::INFINITY, f64::min)
        < wanted
        && (0.0..=1.0).contains(&(l + step))
    {
        l = format!("{:.4}", l + step)
            .parse()
            .expect("finite lightness");
        result = hex(hls_rgb([h, l, s]));
    }
    result
}
/// Inputs are normalized colours and a light/dark theme from validated Look settings.
pub fn accent_roles(accent: &str, dark: bool) -> std::collections::BTreeMap<String, String> {
    let surfaces = if dark {
        ["#161b23", "#0f1319"]
    } else {
        ["#ffffff", "#eef0f4"]
    };
    let fill = towards(accent, &surfaces, 3.0, dark);
    let text = towards(accent, &surfaces, 4.5, dark);
    let ink = if contrast("#101317", &fill) > contrast("#ffffff", &fill) {
        "#101317"
    } else {
        "#ffffff"
    };
    let [r, g, b] = rgb(&text).map(|c| (c * 255.0).round_ties_even() as u8);
    [
        ("--accent", text),
        ("--accent-fill", fill),
        ("--on-accent", ink.into()),
        (
            "--accent-soft",
            format!("rgba({r},{g},{b},{})", if dark { "0.16" } else { "0.14" }),
        ),
    ]
    .into_iter()
    .map(|(k, v)| (k.into(), v))
    .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn colours_are_readable_and_rounding_is_even() {
        assert_eq!(hex([0.5 / 255.0, 1.5 / 255.0, 2.5 / 255.0]), "#000202");
        for accent in [
            DEFAULT_ACCENT,
            "#f26a00",
            "#e6b800",
            "#3fa535",
            "#0a5fd1",
            "#4a5568",
            "#ffffff",
            "#000000",
            "#ffff00",
            "#00ff00",
            "#0000ff",
            "#808080",
            "#eef0f4",
            "#161b23",
        ] {
            for dark in [false, true] {
                let roles = accent_roles(accent, dark);
                for surface in if dark {
                    ["#161b23", "#0f1319"]
                } else {
                    ["#ffffff", "#eef0f4"]
                } {
                    assert!(contrast(&roles["--accent"], surface) >= 4.5);
                    assert!(contrast(&roles["--accent-fill"], surface) >= 3.0);
                }
                assert!(contrast(&roles["--on-accent"], &roles["--accent-fill"]) >= 4.5);
            }
        }
    }
}
