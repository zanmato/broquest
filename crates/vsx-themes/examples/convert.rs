//! Convert a VS Code theme extension to a gpui-component theme file.
//!
//! ```sh
//! cargo run -p vsx-themes --example convert -- path/to/extension-folder > theme.json
//! cargo run -p vsx-themes --example convert -- theme.vsix --sql > theme.json
//! cargo run -p vsx-themes --example convert -- --open-vsx enkia.tokyo-night --sql
//! ```

use std::io::Write as _;
use std::path::PathBuf;

use anyhow::{Context as _, bail};
use vsx_themes::{ConvertOptions, Package, SQL, convert_package, gallery};

fn main() -> anyhow::Result<()> {
    let mut source = None;
    let mut extension_id = None;
    let mut options = ConvertOptions::default();
    let mut arguments = std::env::args().skip(1);
    while let Some(argument) = arguments.next() {
        match argument.as_str() {
            "--sql" => options = SQL,
            "--open-vsx" => {
                extension_id = Some(arguments.next().context("--open-vsx takes an id")?)
            }
            _ if source.is_none() => source = Some(PathBuf::from(argument)),
            _ => bail!("unexpected argument {argument}"),
        }
    }

    let package = match (extension_id, source) {
        (Some(id), None) => {
            let client = gallery::Client::new(gallery::OPEN_VSX)?;
            let extension = client.find(&id)?;
            Package::from_vsix(&client.download(&extension)?)?
        }
        (None, Some(path)) if path.is_dir() => Package::from_folder(&path)?,
        (None, Some(path)) => Package::from_vsix(&std::fs::read(&path)?)?,
        _ => bail!("give an extension folder, a .vsix, or --open-vsx publisher.name"),
    };
    let converted = convert_package(&package, &options)?;
    std::io::stdout().write_all(converted.to_json().as_bytes())?;
    Ok(())
}
