use crate::app_settings::AppSettings;
use anyhow::Context as _;
use gpui::{App, AppContext, Entity, Global, Task};
use semver::Version;
use std::path::PathBuf;
use std::time::Duration;

const CHECK_INTERVAL: Duration = Duration::from_secs(60 * 60);
const GITHUB_OWNER: &str = "zanmato";
const GITHUB_REPO: &str = "broquest";
const UPDATED_FROM_MARKER: &str = ".updated_from";

/// The release asset built for this platform and the path of the executable
/// inside it, as produced by `.github/workflows/build-and-package.yml`. `None`
/// on platforms no release is built for, where updating is disabled.
fn release_asset() -> Option<(&'static str, &'static str)> {
    if cfg!(all(target_os = "linux", target_arch = "x86_64")) {
        Some(("broquest-linux-x86_64", "broquest"))
    } else if cfg!(all(target_os = "windows", target_arch = "x86_64")) {
        Some(("broquest-windows-x86_64.exe", "broquest.exe"))
    } else if cfg!(all(target_os = "macos", target_arch = "aarch64")) {
        Some((
            "broquest-macos-arm64.tar.gz",
            "Broquest.app/Contents/MacOS/Broquest",
        ))
    } else {
        None
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum UpdateState {
    UpToDate,
    /// A newer release exists and has not been downloaded.
    Available(String),
    /// A newer release exists, but this install cannot replace its own
    /// executable (a system package owns it), so the user is sent to the
    /// release page instead.
    Manual(String),
    Downloading(String),
    /// The release is downloaded and applied by restarting.
    Ready(String),
}

pub struct UpdateManager {
    pub state: Entity<UpdateState>,
    /// The version this install ran before the update applied on the previous
    /// run, present only on the first launch after updating.
    pub updated_from: Option<String>,
}

impl Global for UpdateManager {}

impl UpdateManager {
    pub fn new(cx: &mut App) -> Self {
        let updated_from = match Self::updates_dir() {
            Ok(updates_dir) => {
                let marker_path = updates_dir.join(UPDATED_FROM_MARKER);
                match std::fs::read_to_string(&marker_path) {
                    Ok(version) => {
                        if let Err(error) = std::fs::remove_file(&marker_path) {
                            tracing::warn!("Failed to remove update marker file: {error}");
                        }
                        Some(version)
                    }
                    Err(error) if error.kind() == std::io::ErrorKind::NotFound => None,
                    Err(error) => {
                        tracing::warn!("Failed to read update marker file: {error}");
                        None
                    }
                }
            }
            Err(error) => {
                tracing::warn!("Failed to resolve updates directory: {error:#}");
                None
            }
        };

        Self {
            state: cx.new(|_| UpdateState::UpToDate),
            updated_from,
        }
    }

    /// `None` when no manager is installed, as in tests that build the app
    /// without running `main`.
    pub fn try_global(cx: &App) -> Option<&Self> {
        cx.try_global::<Self>()
    }

    pub fn start_polling(cx: &mut App) {
        if release_asset().is_none() {
            tracing::info!("No release is built for this platform, update checks are disabled");
            return;
        }

        cx.spawn(async move |cx| {
            loop {
                let check = cx.update(|cx| {
                    AppSettings::global(cx)
                        .settings
                        .general
                        .check_for_updates
                        .then(|| Self::try_global(cx).map(|this| this.state.clone()))
                        .flatten()
                });
                if let Some(state) = check {
                    match smol::unblock(Self::check_latest_release).await {
                        Ok(Some(found)) => {
                            cx.update(|cx| {
                                state.update(cx, |state, cx| {
                                    // A running download reports its own result.
                                    if !matches!(state, UpdateState::Downloading(_)) {
                                        *state = found;
                                        cx.notify();
                                    }
                                });
                            });
                        }
                        Ok(None) => {}
                        Err(error) => tracing::error!("Failed to check for updates: {error:#}"),
                    }
                }
                cx.background_executor().timer(CHECK_INTERVAL).await;
            }
        })
        .detach();
    }

    /// The latest release when it is newer than this build.
    fn newer_release() -> anyhow::Result<Option<self_update::update::Release>> {
        let release = self_update::backends::github::Update::configure()
            .repo_owner(GITHUB_OWNER)
            .repo_name(GITHUB_REPO)
            .bin_name("broquest")
            .current_version(env!("CARGO_PKG_VERSION"))
            .show_output(false)
            .build()?
            .get_latest_release()?;

        let remote_version = Version::parse(
            release
                .version
                .strip_prefix('v')
                .unwrap_or(&release.version),
        )?;
        Ok((remote_version > Version::parse(env!("CARGO_PKG_VERSION"))?).then_some(release))
    }

    fn check_latest_release() -> anyhow::Result<Option<UpdateState>> {
        let Some(release) = Self::newer_release()? else {
            return Ok(None);
        };
        Ok(Some(if !Self::executable_is_replaceable() {
            UpdateState::Manual(release.version)
        } else if Self::staged_binary_path(&release.version)?.exists() {
            UpdateState::Ready(release.version)
        } else {
            UpdateState::Available(release.version)
        }))
    }

    /// Whether the running executable can be swapped in place, which takes
    /// creating a file next to it. False for a `.deb` install in `/usr/bin`.
    fn executable_is_replaceable() -> bool {
        let Some(directory) = std::env::current_exe()
            .ok()
            .and_then(|executable| Some(executable.parent()?.to_path_buf()))
        else {
            return false;
        };
        let probe = directory.join(format!(".broquest-update-probe-{}", std::process::id()));
        match std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&probe)
        {
            Ok(_) => {
                if let Err(error) = std::fs::remove_file(&probe) {
                    tracing::warn!("Failed to remove update probe file: {error}");
                }
                true
            }
            Err(_) => false,
        }
    }

    /// Download the available release. The state moves to `Downloading` right
    /// away and to `Ready` once the executable is staged; on failure it falls
    /// back to `Available` and the error is returned for the caller to show.
    pub fn download_update(cx: &mut App) -> Task<anyhow::Result<()>> {
        let Some(state) = Self::try_global(cx).map(|this| this.state.clone()) else {
            return Task::ready(Err(anyhow::anyhow!("Updates are not available")));
        };
        let UpdateState::Available(version) = state.read(cx).clone() else {
            return Task::ready(Ok(()));
        };
        state.update(cx, |state, cx| {
            *state = UpdateState::Downloading(version.clone());
            cx.notify();
        });

        cx.spawn(async move |cx| {
            let result = smol::unblock(Self::stage_latest_release).await;
            cx.update(|cx| {
                state.update(cx, |state, cx| {
                    *state = match &result {
                        Ok(Some(staged_version)) => UpdateState::Ready(staged_version.clone()),
                        Ok(None) => UpdateState::UpToDate,
                        Err(_) => UpdateState::Available(version),
                    };
                    cx.notify();
                });
            });
            result.map(|_| ())
        })
    }

    /// Download the latest release, when it is newer than this build, and
    /// stage its executable. Returns the staged version.
    fn stage_latest_release() -> anyhow::Result<Option<String>> {
        let (asset_name, executable_in_asset) =
            release_asset().context("No release is built for this platform")?;
        let Some(release) = Self::newer_release()? else {
            return Ok(None);
        };

        let staged_binary = Self::staged_binary_path(&release.version)?;
        if staged_binary.exists() {
            return Ok(Some(release.version));
        }

        tracing::info!("Downloading update {}", release.version);
        let asset = release
            .assets
            .iter()
            .find(|asset| asset.name == asset_name)
            .with_context(|| format!("Release {} has no {asset_name} asset", release.version))?;

        // Starting from an empty directory drops releases staged earlier and
        // never applied. The download is unpacked in a scratch directory and
        // renamed into place, so an interrupted one never leaves a truncated
        // file at the staged path.
        let updates_dir = Self::updates_dir()?;
        if updates_dir.exists() {
            std::fs::remove_dir_all(&updates_dir)?;
        }
        let download_dir = updates_dir.join("download");
        std::fs::create_dir_all(&download_dir)?;

        let archive_path = download_dir.join(&asset.name);
        let mut archive = std::fs::File::create(&archive_path)?;
        let mut download = self_update::Download::from_url(&asset.download_url);
        download.set_header(reqwest::header::ACCEPT, "application/octet-stream".parse()?);
        download.show_progress(false);
        download.download_to(&mut archive)?;
        drop(archive);

        // A bare executable is copied under the file name of the requested
        // path, an archive member keeps its full path.
        let extract_dir = download_dir.join("extracted");
        self_update::Extract::from_source(&archive_path)
            .extract_file(&extract_dir, executable_in_asset)?;
        let extracted = [
            extract_dir.join(executable_in_asset),
            extract_dir.join(
                std::path::Path::new(executable_in_asset)
                    .file_name()
                    .context("Release executable path has no file name")?,
            ),
        ]
        .into_iter()
        .find(|path| path.is_file())
        .context("Downloaded release does not contain the executable")?;

        if let Some(staged_dir) = staged_binary.parent() {
            std::fs::create_dir_all(staged_dir)?;
        }
        std::fs::rename(&extracted, &staged_binary)?;
        if let Err(error) = std::fs::remove_dir_all(&download_dir) {
            tracing::warn!("Failed to remove update download directory: {error}");
        }

        tracing::info!("Update {} staged", release.version);
        Ok(Some(release.version))
    }

    fn updates_dir() -> anyhow::Result<PathBuf> {
        Ok(dirs::data_local_dir()
            .context("Failed to get data directory")?
            .join("broquest")
            .join("updates"))
    }

    /// Staged executables are kept per version, so one downloaded for a
    /// release that has since been superseded is never applied.
    fn staged_binary_path(version: &str) -> anyhow::Result<PathBuf> {
        let name = if cfg!(target_os = "windows") {
            "broquest.exe"
        } else {
            "broquest"
        };
        Ok(Self::updates_dir()?.join(version).join(name))
    }

    /// Replace the running executable with the staged one and restart. On
    /// success the app is quitting when this returns.
    pub fn apply_pending_update(cx: &mut App) -> anyhow::Result<()> {
        let state = Self::try_global(cx)
            .context("Updates are not available")?
            .state
            .read(cx)
            .clone();
        let UpdateState::Ready(version) = state else {
            anyhow::bail!("No update has been downloaded");
        };
        let staged_binary = Self::staged_binary_path(&version)?;
        anyhow::ensure!(
            staged_binary.exists(),
            "No downloaded update found at {}",
            staged_binary.display()
        );

        // Resolved before the swap: on Linux the path of a replaced executable
        // reads back with a " (deleted)" suffix.
        let executable =
            std::env::current_exe().context("Failed to resolve the executable path")?;

        self_update::self_replace::self_replace(&staged_binary)
            .with_context(|| format!("Failed to replace {}", executable.display()))?;

        if let Err(error) = std::fs::write(
            Self::updates_dir()?.join(UPDATED_FROM_MARKER),
            env!("CARGO_PKG_VERSION"),
        ) {
            tracing::warn!("Failed to write update marker file: {error}");
        }
        if let Err(error) = std::fs::remove_file(&staged_binary) {
            tracing::warn!("Failed to remove staged update binary: {error}");
        }

        tracing::info!("Update applied, restarting...");
        // macOS relaunches the enclosing .app bundle, which GPUI finds itself.
        if !cfg!(target_os = "macos") {
            cx.set_restart_path(executable);
        }
        cx.restart();
        Ok(())
    }

    pub fn changelog_url(version: &str) -> String {
        format!("https://github.com/{GITHUB_OWNER}/{GITHUB_REPO}/releases/tag/{version}")
    }
}
