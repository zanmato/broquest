use anyhow::Result;
use rust_embed::RustEmbed;
use std::path::PathBuf;

#[derive(RustEmbed)]
#[folder = "$CARGO_MANIFEST_DIR/assets/themes"]
#[include = "*.json"]
pub struct EmbeddedThemes;

pub struct ThemesManager;

impl ThemesManager {
    /// Initialize themes by copying embedded themes to user's local directory
    pub fn init() -> Result<()> {
        let themes_dir = Self::themes_dir();

        // Create the themes directory if it doesn't exist
        std::fs::create_dir_all(&themes_dir)?;

        // Copy all embedded themes to the user's themes directory
        for theme_file in EmbeddedThemes::iter() {
            if let Some(embedded_file) = EmbeddedThemes::get(&theme_file) {
                let theme_path = themes_dir.join(theme_file.as_ref());

                // Only copy if the theme doesn't already exist or needs updating
                match Self::should_copy_theme(&theme_path, &embedded_file.data) {
                    Ok(true) => {
                        std::fs::write(&theme_path, embedded_file.data)?;
                        println!("Installed theme: {}", theme_file);
                    }
                    Ok(false) => {
                        // Theme already exists and is up to date
                    }
                    Err(e) => {
                        eprintln!("Error checking theme {}: {}", theme_file, e);
                        // Try to copy anyway if we can't check
                        std::fs::write(&theme_path, embedded_file.data)?;
                        println!("Installed theme: {} (forced)", theme_file);
                    }
                }
            }
        }

        Ok(())
    }

    /// Get the themes directory path in user's local data directory
    fn themes_dir() -> PathBuf {
        let mut path = dirs::data_dir().unwrap_or_else(|| PathBuf::from("."));
        path.push("broquest");
        path.push("themes");
        path
    }

    /// Check if a theme should be copied (doesn't exist or differs from the
    /// embedded copy).
    ///
    /// Compares contents rather than file size: a theme edit that swaps one
    /// colour for another of the same length leaves the size unchanged, and a
    /// size check would silently skip installing it.
    fn should_copy_theme(theme_path: &PathBuf, embedded: &[u8]) -> Result<bool> {
        if !theme_path.exists() {
            return Ok(true);
        }

        let current = std::fs::read(theme_path)?;
        Ok(current != embedded)
    }

    /// Get the themes directory path (public)
    pub fn themes_directory() -> PathBuf {
        Self::themes_dir()
    }
}
