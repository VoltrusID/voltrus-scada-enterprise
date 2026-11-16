//! The white-label theme: colours for the light and the dark theme, checked for
//! contrast, and a font from a fixed list (or one uploaded file).
//!
//! Colours are written into CSS variables by the web client, so they are only ever
//! `#rrggbb`. Contrast follows WCAG 2.x: text on the background and on surfaces
//! (cards, dialogs) needs 4.5:1; the accent (links, active tabs, highlighted
//! buttons and borders) needs 3:1 against the background and surfaces.

use serde::{Deserialize, Serialize};

/// Minimum contrast of text on the background and on surfaces.
pub const TEXT_CONTRAST: f64 = 4.5;
/// Minimum contrast of the accent on the background and on surfaces.
pub const ACCENT_CONTRAST: f64 = 3.0;

/// Fonts a theme may use: system font stacks, nothing loaded from the network.
/// `custom` is the uploaded file (see `handlers::put_font`).
pub const FONTS: &[(&str, &str)] = &[
    ("system", "Voltrus default (system UI)"),
    ("sans", "Helvetica / Arial"),
    ("humanist", "Segoe UI / Noto Sans"),
    ("rounded", "Rounded"),
    ("serif", "Serif (Charter / Georgia)"),
    ("mono", "Monospace"),
    ("custom", "Uploaded font"),
];

/// One theme's colours; empty keeps the Voltrus colour.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Palette {
    pub background: String,
    pub surface: String,
    pub text: String,
    pub accent: String,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Theme {
    pub dark: Palette,
    pub light: Palette,
    /// One of [`FONTS`]; empty means `system`.
    pub font: String,
}

/// The Voltrus colours of the core client (`index.css`: `--bg`, `--card`, `--text`,
/// `--cyan`), which apply where a theme is empty.
fn default_colours(dark: bool) -> [&'static str; 4] {
    if dark {
        ["#09090b", "#111113", "#e4e4e7", "#22c55e"]
    } else {
        ["#ffffff", "#f9fafb", "#111827", "#16a34a"]
    }
}

/// `#rgb` or `#rrggbb` (any case) as lower-case `#rrggbb`; empty stays empty.
pub fn normalize_colour(value: &str) -> Result<String, String> {
    let v = value.trim();
    if v.is_empty() {
        return Ok(String::new());
    }
    let hex = v.strip_prefix('#').unwrap_or("");
    if !hex.chars().all(|c| c.is_ascii_hexdigit()) {
        return Err(format!("{v} is not a colour like #0055aa"));
    }
    match hex.len() {
        6 => Ok(format!("#{}", hex.to_ascii_lowercase())),
        3 => Ok(format!(
            "#{}",
            hex.chars()
                .flat_map(|c| [c, c])
                .collect::<String>()
                .to_ascii_lowercase()
        )),
        _ => Err(format!("{v} is not a colour like #0055aa")),
    }
}

fn rgb(hex: &str) -> [f64; 3] {
    let h = hex.trim_start_matches('#');
    let c = |i: usize| u8::from_str_radix(&h[i..i + 2], 16).unwrap_or(0) as f64 / 255.0;
    [c(0), c(2), c(4)]
}

/// WCAG relative luminance of `#rrggbb`.
pub fn luminance(hex: &str) -> f64 {
    let lin = |c: f64| {
        if c <= 0.03928 {
            c / 12.92
        } else {
            ((c + 0.055) / 1.055).powf(2.4)
        }
    };
    let [r, g, b] = rgb(hex);
    0.2126 * lin(r) + 0.7152 * lin(g) + 0.0722 * lin(b)
}

/// WCAG contrast ratio of two `#rrggbb` colours, 1 to 21.
pub fn contrast(a: &str, b: &str) -> f64 {
    let (la, lb) = (luminance(a), luminance(b));
    let (hi, lo) = if la > lb { (la, lb) } else { (lb, la) };
    (hi + 0.05) / (lo + 0.05)
}

fn pick<'a>(value: &'a str, default: &'a str) -> &'a str {
    if value.is_empty() { default } else { value }
}

impl Palette {
    fn normalized(&self, which: &str) -> Result<Palette, String> {
        let n = |field: &str, v: &str| {
            normalize_colour(v).map_err(|e| format!("{which} theme, {field}: {e}"))
        };
        Ok(Palette {
            background: n("background", &self.background)?,
            surface: n("surface", &self.surface)?,
            text: n("text", &self.text)?,
            accent: n("accent", &self.accent)?,
        })
    }

    /// Check the contrast of this palette (Voltrus colours filling the gaps). The
    /// palette's own accent is checked; without one the client paints the brand
    /// accent, darkened or lightened just enough to reach 3:1 in this theme (one
    /// brand colour rarely reads on both white and near-black).
    fn check(&self, dark: bool) -> Result<(), String> {
        let which = if dark { "Dark" } else { "Light" };
        let [bg, surface, text, _] = default_colours(dark);
        let bg = pick(&self.background, bg);
        let surface = pick(&self.surface, surface);
        let text = pick(&self.text, text);
        let mut checks = vec![
            ("text on the background", text, bg, TEXT_CONTRAST),
            ("text on surfaces", text, surface, TEXT_CONTRAST),
        ];
        if !self.accent.is_empty() {
            checks.push((
                "the accent on the background",
                &self.accent,
                bg,
                ACCENT_CONTRAST,
            ));
            checks.push((
                "the accent on surfaces",
                &self.accent,
                surface,
                ACCENT_CONTRAST,
            ));
        }
        for (what, a, b, min) in checks {
            let ratio = contrast(a, b);
            if ratio + 1e-9 < min {
                return Err(format!(
                    "{which} theme: contrast of {what} ({a} on {b}) is {ratio:.2}:1, it must be at least {min}:1"
                ));
            }
        }
        Ok(())
    }
}

impl Theme {
    /// Normalise every colour and the font, then check contrast in both themes with
    /// `accent` (the brand's single accent colour) where a theme has no accent.
    pub fn validated(&self, accent: &str) -> Result<Theme, String> {
        let theme = Theme {
            dark: self.dark.normalized("Dark")?,
            light: self.light.normalized("Light")?,
            font: match self.font.trim() {
                "" => String::new(),
                f if FONTS.iter().any(|(id, _)| *id == f) => f.to_string(),
                f => {
                    return Err(format!(
                        "font {f:?} is not one of {}",
                        FONTS
                            .iter()
                            .map(|(id, _)| *id)
                            .collect::<Vec<_>>()
                            .join(", ")
                    ));
                }
            },
        };
        normalize_colour(accent)?;
        theme.dark.check(true)?;
        theme.light.check(false)?;
        Ok(theme)
    }

    pub fn is_empty(&self) -> bool {
        *self == Theme::default()
    }
}

/// A font file accepted for upload: its MIME type from the file's signature.
pub fn font_mime(data: &[u8]) -> Option<&'static str> {
    match data.get(..4)? {
        b"wOF2" => Some("font/woff2"),
        b"wOFF" => Some("font/woff"),
        [0, 1, 0, 0] | b"true" => Some("font/ttf"),
        b"OTTO" => Some("font/otf"),
        _ => None,
    }
}

/// Largest uploaded font file.
pub const MAX_FONT_BYTES: usize = 256 * 1024;

#[cfg(test)]
mod tests {
    use super::*;

    fn palette(bg: &str, surface: &str, text: &str, accent: &str) -> Palette {
        Palette {
            background: bg.into(),
            surface: surface.into(),
            text: text.into(),
            accent: accent.into(),
        }
    }

    #[test]
    fn contrast_matches_wcag() {
        assert!((contrast("#000000", "#ffffff") - 21.0).abs() < 1e-9);
        assert!((contrast("#ffffff", "#ffffff") - 1.0).abs() < 1e-9);
        // The WCAG example pair: #777777 on white is 4.48:1.
        let r = contrast("#777777", "#ffffff");
        assert!((r - 4.48).abs() < 0.01, "{r}");
    }

    #[test]
    fn colours_are_normalized_and_anything_else_refused() {
        assert_eq!(normalize_colour("#0055AA").unwrap(), "#0055aa");
        assert_eq!(normalize_colour(" #0af ").unwrap(), "#00aaff");
        assert_eq!(normalize_colour("").unwrap(), "");
        for bad in ["red", "#ggg", "0055aa", "#0055aa;x", "#0055aa00", "url(x)"] {
            assert!(normalize_colour(bad).is_err(), "{bad}");
        }
    }

    #[test]
    fn the_voltrus_defaults_pass() {
        assert!(Theme::default().validated("").is_ok());
        assert!(Theme::default().validated("#22c55e").is_ok());
    }

    #[test]
    fn low_contrast_text_is_refused_with_the_ratio() {
        let theme = Theme {
            light: palette("#ffffff", "", "#aaaaaa", ""),
            ..Default::default()
        };
        let err = theme.validated("").unwrap_err();
        assert!(err.contains("Light theme"), "{err}");
        assert!(err.contains("text on the background"), "{err}");
        assert!(err.contains("2.32:1"), "{err}");
        // A custom surface is checked too.
        let theme = Theme {
            dark: palette("#000000", "#555555", "#ffffff", ""),
            ..Default::default()
        };
        assert!(theme.validated("").is_ok());
        let theme = Theme {
            dark: palette("#000000", "#888888", "#ffffff", ""),
            ..Default::default()
        };
        assert!(
            theme
                .validated("")
                .unwrap_err()
                .contains("text on surfaces")
        );
    }

    #[test]
    fn a_theme_accent_is_checked_and_the_brand_accent_adjusted() {
        // The single brand accent is adjusted per theme by the client, never refused
        // for contrast (light yellow reads on dark, not on white)...
        assert!(Theme::default().validated("#ffee58").is_ok());
        assert!(Theme::default().validated("#22c55e").is_ok());
        // ...but must still be a colour.
        assert!(Theme::default().validated("yellow").is_err());
        // A theme's own accent is the operator's explicit choice and is checked.
        let theme = Theme {
            light: palette("", "", "", "#ffee58"),
            ..Default::default()
        };
        let err = theme.validated("").unwrap_err();
        assert!(
            err.contains("Light theme") && err.contains("accent"),
            "{err}"
        );
        let theme = Theme {
            light: palette("", "", "", "#8a6d00"),
            ..Default::default()
        };
        assert!(theme.validated("#ffee58").is_ok());
    }

    #[test]
    fn a_full_custom_theme_validates() {
        let theme = Theme {
            dark: palette("#0b1220", "#131c2e", "#e6edf7", "#4ea1ff"),
            light: palette("#ffffff", "#f3f6fa", "#0b1220", "#0055aa"),
            font: "humanist".into(),
        };
        let v = theme.validated("").unwrap();
        assert_eq!(v.font, "humanist");
        assert!(!v.is_empty());
    }

    #[test]
    fn only_listed_fonts_are_accepted() {
        for (id, _) in FONTS {
            let t = Theme {
                font: (*id).into(),
                ..Default::default()
            };
            assert!(t.validated("").is_ok(), "{id}");
        }
        let t = Theme {
            font: "Comic Sans; } body { background: url(http://x)".into(),
            ..Default::default()
        };
        assert!(t.validated("").is_err());
    }

    #[test]
    fn unknown_theme_fields_are_refused() {
        let r: Result<Theme, _> = serde_json::from_value(
            serde_json::json!({"dark": {"background": "#000000", "css": "x"}}),
        );
        assert!(r.is_err());
    }

    #[test]
    fn font_files_are_recognised_by_their_signature() {
        assert_eq!(font_mime(b"wOF2...."), Some("font/woff2"));
        assert_eq!(font_mime(b"wOFF...."), Some("font/woff"));
        assert_eq!(font_mime(&[0, 1, 0, 0, 9]), Some("font/ttf"));
        assert_eq!(font_mime(b"OTTO...."), Some("font/otf"));
        assert_eq!(font_mime(b"<svg"), None);
        assert_eq!(font_mime(b"@font-face"), None);
        assert_eq!(font_mime(b"wO"), None);
    }
}
