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
    /// A folder of `.http` and `.rest` files. Read-only for now.
    Http,
}

impl CollectionFormat {
    /// Stable string used when persisting the format in the database.
    pub fn as_db_str(&self) -> &'static str {
        match self {
            Self::Broquest => "broquest",
            Self::OpenCollection => "opencollection",
            Self::Http => "http",
        }
    }

    /// A short tag for the sidebar. The default format has none.
    pub fn tag(&self) -> Option<&'static str> {
        match self {
            Self::Broquest => None,
            Self::OpenCollection => Some("OC"),
            Self::Http => Some("HTTP"),
        }
    }

    /// The format's name.
    pub fn label(&self) -> &'static str {
        match self {
            Self::Broquest => "Broquest TOML",
            Self::OpenCollection => "OpenCollection (Bruno)",
            Self::Http => ".http files",
        }
    }

    /// Where the collection is stored and what it keeps.
    pub fn description(&self) -> &'static str {
        match self {
            Self::Broquest => {
                "A collection.toml and one TOML file per request. Everything is saved."
            }
            Self::OpenCollection => {
                "Bruno's YAML format, in opencollection.yml or a folder of YAML files. Everything is saved."
            }
            Self::Http => {
                "A folder of .http and .rest files, as used by the JetBrains HTTP Client and the VS Code REST Client. Requests, groups and environments are saved to the files. Collection docs, variables and auth have no place in them and are not saved."
            }
        }
    }

    /// Parse the value stored in the database, defaulting to `Broquest` for
    /// legacy rows (written before the `format` column existed).
    pub fn from_db_str(value: &str) -> Self {
        match value {
            "opencollection" => Self::OpenCollection,
            "http" => Self::Http,
            _ => Self::Broquest,
        }
    }
}
