//! The little color arithmetic the conversion needs: parsing VS Code's hex
//! forms, mixing, compositing and WCAG contrast.

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Rgba {
    pub red: f32,
    pub green: f32,
    pub blue: f32,
    pub alpha: f32,
}

impl Rgba {
    pub const BLACK: Rgba = Rgba::opaque(0.0, 0.0, 0.0);
    pub const WHITE: Rgba = Rgba::opaque(1.0, 1.0, 1.0);

    pub const fn opaque(red: f32, green: f32, blue: f32) -> Self {
        Self {
            red,
            green,
            blue,
            alpha: 1.0,
        }
    }

    /// Parse `#rgb`, `#rgba`, `#rrggbb` or `#rrggbbaa`, the forms VS Code
    /// accepts in a theme.
    pub fn parse(text: &str) -> Option<Self> {
        let hex = text.trim().strip_prefix('#')?;
        if !hex.is_ascii() {
            return None;
        }
        let digits: Vec<u8> = match hex.len() {
            3 | 4 => hex
                .chars()
                .map(|digit| u8::from_str_radix(&format!("{digit}{digit}"), 16).ok())
                .collect::<Option<_>>()?,
            6 | 8 => (0..hex.len())
                .step_by(2)
                .map(|start| u8::from_str_radix(&hex[start..start + 2], 16).ok())
                .collect::<Option<_>>()?,
            _ => return None,
        };
        let channel = |index: usize| digits.get(index).map(|value| *value as f32 / 255.0);
        Some(Self {
            red: channel(0)?,
            green: channel(1)?,
            blue: channel(2)?,
            alpha: channel(3).unwrap_or(1.0),
        })
    }

    /// `#rrggbb`, or `#rrggbbaa` when not opaque.
    pub fn to_hex(self) -> String {
        let byte = |value: f32| (value.clamp(0.0, 1.0) * 255.0).round() as u8;
        let mut hex = format!(
            "#{:02x}{:02x}{:02x}",
            byte(self.red),
            byte(self.green),
            byte(self.blue)
        );
        if byte(self.alpha) != 255 {
            hex.push_str(&format!("{:02x}", byte(self.alpha)));
        }
        hex
    }

    pub fn with_alpha(self, alpha: f32) -> Self {
        Self { alpha, ..self }
    }

    pub fn is_opaque(self) -> bool {
        self.alpha >= 0.999
    }

    /// `amount` of the way from `self` to `other`, alpha included.
    pub fn mix(self, other: Rgba, amount: f32) -> Self {
        let lerp = |from: f32, to: f32| from + (to - from) * amount;
        Self {
            red: lerp(self.red, other.red),
            green: lerp(self.green, other.green),
            blue: lerp(self.blue, other.blue),
            alpha: lerp(self.alpha, other.alpha),
        }
    }

    /// The opaque color `self` shows as when painted over `background`.
    pub fn over(self, background: Rgba) -> Self {
        let background = if background.is_opaque() {
            background
        } else {
            background.over(Rgba::BLACK)
        };
        Self::opaque(
            background.red + (self.red - background.red) * self.alpha,
            background.green + (self.green - background.green) * self.alpha,
            background.blue + (self.blue - background.blue) * self.alpha,
        )
    }

    /// WCAG relative luminance of the opaque color.
    pub fn luminance(self) -> f32 {
        let linear = |value: f32| {
            if value <= 0.03928 {
                value / 12.92
            } else {
                ((value + 0.055) / 1.055).powf(2.4)
            }
        };
        0.2126 * linear(self.red) + 0.7152 * linear(self.green) + 0.0722 * linear(self.blue)
    }

    /// WCAG contrast ratio of `self` painted over `background`.
    pub fn contrast(self, background: Rgba) -> f32 {
        let background = background.over(Rgba::BLACK);
        let first = self.over(background).luminance();
        let second = background.luminance();
        (first.max(second) + 0.05) / (first.min(second) + 0.05)
    }

    pub fn is_dark(self) -> bool {
        self.over(Rgba::BLACK).luminance() < 0.2
    }

    /// `self`, moved towards `toward` just far enough to reach `minimum`
    /// contrast against `background`. Themes tuned for VS Code's chrome can
    /// dim text that Blanco-style UIs show in more places.
    pub fn with_contrast(self, background: Rgba, toward: Rgba, minimum: f32) -> Self {
        let opaque = self.over(background);
        if opaque.contrast(background) >= minimum {
            return self;
        }
        for step in 1..=20 {
            let candidate = opaque.mix(toward, step as f32 / 20.0);
            if candidate.contrast(background) >= minimum {
                return candidate;
            }
        }
        toward
    }

    /// Black or white, whichever reads better on `self`.
    pub fn readable_text(self) -> Self {
        if Rgba::WHITE.contrast(self) >= Rgba::BLACK.contrast(self) {
            Rgba::WHITE
        } else {
            Rgba::BLACK
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_hex_form_parses() {
        assert_eq!(
            Rgba::parse("#fff").map(Rgba::to_hex).as_deref(),
            Some("#ffffff")
        );
        assert_eq!(
            Rgba::parse("#fff8").map(Rgba::to_hex).as_deref(),
            Some("#ffffff88")
        );
        assert_eq!(
            Rgba::parse("#1A1B26").map(Rgba::to_hex).as_deref(),
            Some("#1a1b26")
        );
        assert_eq!(
            Rgba::parse("#3d59a1DD").map(Rgba::to_hex).as_deref(),
            Some("#3d59a1dd")
        );
        assert_eq!(
            Rgba::parse("#3d59a1ff").map(Rgba::to_hex).as_deref(),
            Some("#3d59a1")
        );
        assert!(Rgba::parse("red").is_none());
        assert!(Rgba::parse("#12345").is_none());
        assert!(Rgba::parse("#gggggg").is_none());
    }

    #[test]
    fn contrast_matches_wcag() {
        let ratio = Rgba::BLACK.contrast(Rgba::WHITE);
        assert!((ratio - 21.0).abs() < 0.01, "{ratio}");
    }

    #[test]
    fn dim_text_is_raised_to_the_minimum() {
        let background = Rgba::parse("#16161e").unwrap();
        let dim = Rgba::parse("#3b3e52").unwrap();
        let raised = dim.with_contrast(background, Rgba::WHITE, 3.0);
        assert!(raised.contrast(background) >= 3.0);
        let fine = Rgba::parse("#a9b1d6").unwrap();
        assert_eq!(fine.with_contrast(background, Rgba::WHITE, 3.0), fine);
    }
}
