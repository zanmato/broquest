//! On-disk format of a collection.
//!
//! broquest historically stored collections as a directory of TOML files
//! (`collection.toml` + one `.toml` per request). We also support Bruno's
//! OpenCollection YAML format (a single `opencollection.yml`, or a non-bundled
//! directory tree). The format is a per-collection property so that a
//! collection created natively can be switched to save as OpenCollection.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum CollectionFormat {
    /// Native broquest layout: `collection.toml` + directory-per-request TOML.
    #[default]
    Broquest,
    /// Bruno OpenCollection: `opencollection.yml` (bundled or non-bundled tree).
    OpenCollection,
}

impl CollectionFormat {
    /// Stable string used when persisting the format in the database.
    pub fn as_db_str(&self) -> &'static str {
        match self {
            Self::Broquest => "broquest",
            Self::OpenCollection => "opencollection",
        }
    }

    /// Parse the value stored in the database, defaulting to `Broquest` for
    /// legacy rows (written before the `format` column existed).
    pub fn from_db_str(value: &str) -> Self {
        match value {
            "opencollection" => Self::OpenCollection,
            _ => Self::Broquest,
        }
    }
}
