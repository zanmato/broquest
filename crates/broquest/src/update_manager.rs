use gpui::{App, AppContext, Entity, Global};
use semver::Version;
use std::path::PathBuf;
use std::time::Duration;

const CHECK_INTERVAL: Duration = Duration::from_secs(60 * 60); // 1 hour
const GITHUB_OWNER: &str = "zanmato";
const GITHUB_REPO: &str = "broquest";

#[derive(Clone, Debug)]
pub struct ReleaseInfo {
    pub version: String,
    #[allow(dead_code)]
    pub html_url: String,
}

pub struct UpdateManager {
    pub pending_update: Entity<Option<ReleaseInfo>>,
    pub just_updated_from: Entity<Option<String>>,
}

impl Global for UpdateManager {}

impl UpdateManager {
    pub fn new(cx: &mut App) -> Self {
        // Check for marker file indicating we just updated
        let marker_path = Self::updates_dir().join(".updated_from");
        let just_updated_from = if marker_path.exists() {
            let version = std::fs::read_to_string(&marker_path).ok();
            let _ = std::fs::remove_file(&marker_path);
            cx.new(|_cx| version)
        } else {
            cx.new(|_cx| None)
        };

        Self {
            pending_update: cx.new(|_cx| None),
            just_updated_from,
        }
    }

    pub fn global(cx: &App) -> &Self {
        cx.global::<Self>()
    }

    pub fn start_polling(cx: &mut App) {
        cx.spawn(async move |cx| {
            loop {
                let _ = cx.update(|cx| {
                    Self::poll_for_updates(cx);
                });
                cx.background_executor().timer(CHECK_INTERVAL).await;
            }
        })
        .detach();
    }

    fn poll_for_updates(cx: &mut App) {
        let pending_update = Self::global(cx).pending_update.clone();

        cx.spawn(async move |cx| {
            // Check for staged update first
            let staged_exists = Self::staged_binary_path().exists();

            if staged_exists {
                // Already have staged update, just update UI
                if let Ok(Some(info)) = Self::fetch_latest_release().await {
                    let _ = cx.update(|cx| {
                        pending_update.update(cx, |state, cx| {
                            *state = Some(info);
                            cx.notify();
                        });
                    });
                }
                return Ok::<_, anyhow::Error>(());
            }

            // Check for new release
            if let Ok(Some(release)) = Self::fetch_latest_release().await {
                tracing::info!("Update found: {}, downloading...", release.version);

                // Download in blocking context
                let result = smol::unblock(|| Self::download_update_to_staging()).await;

                match result {
                    Ok(()) => {
                        tracing::info!("Update staged successfully");
                        let _ = cx.update(|cx| {
                            pending_update.update(cx, |state, cx| {
                                *state = Some(release);
                                cx.notify();
                            });
                        });
                    }
                    Err(e) => tracing::error!("Failed to stage update: {}", e),
                }
            }
            Ok(())
        })
        .detach();
    }

    async fn fetch_latest_release() -> anyhow::Result<Option<ReleaseInfo>> {
        let url = format!(
            "https://api.github.com/repos/{}/{}/releases/latest",
            GITHUB_OWNER, GITHUB_REPO
        );

        let response = reqwest::Client::new()
            .get(&url)
            .header("User-Agent", format!("broquest/{}", env!("CARGO_PKG_VERSION")))
            .header("Accept", "application/vnd.github+json")
            .send()
            .await?;

        if !response.status().is_success() {
            return Ok(None);
        }

        let json: serde_json::Value = response.json().await?;
        let tag_name = json["tag_name"].as_str().unwrap_or_default();
        let html_url = json["html_url"].as_str().unwrap_or_default();

        let remote_version_str = tag_name.strip_prefix('v').unwrap_or(tag_name);
        let remote_version = Version::parse(remote_version_str)?;
        let current_version = Version::parse(env!("CARGO_PKG_VERSION"))?;

        if remote_version > current_version {
            Ok(Some(ReleaseInfo {
                version: tag_name.to_string(),
                html_url: html_url.to_string(),
            }))
        } else {
            Ok(None)
        }
    }

    fn updates_dir() -> PathBuf {
        dirs::data_local_dir()
            .expect("Failed to get data directory")
            .join("se.zanmato.broquest")
            .join("updates")
    }

    fn staged_binary_path() -> PathBuf {
        let name = if cfg!(target_os = "windows") {
            "broquest.exe"
        } else {
            "broquest"
        };
        Self::updates_dir().join(name)
    }

    fn download_update_to_staging() -> anyhow::Result<()> {
        std::fs::create_dir_all(Self::updates_dir())?;

        let mut builder = self_update::backends::github::Update::configure();
        builder
            .repo_owner(GITHUB_OWNER)
            .repo_name(GITHUB_REPO)
            .bin_name("broquest")
            .current_version(env!("CARGO_PKG_VERSION"))
            .no_confirm(true)
            .show_output(false)
            .show_download_progress(false);

        Self::configure_platform_target(&mut builder);

        let updater = builder.build()?;
        let release = updater.get_latest_release()?;

        let target = updater.target();
        let update_ext = if cfg!(target_os = "windows") {
            ".exe"
        } else if cfg!(target_os = "macos") {
            ".tar.gz"
        } else {
            "" // Linux binary without extension
        };

        let asset = release
            .assets
            .iter()
            .find(|a| a.name.contains(&target) && a.name.ends_with(update_ext))
            .cloned()
            .ok_or_else(|| anyhow::anyhow!("No asset found for target: {}", target))?;

        let updates = Self::updates_dir();
        let tmp_archive_path = updates.join(&asset.name);

        // Download
        let mut tmp_archive = std::fs::File::create(&tmp_archive_path)?;
        let mut download = self_update::Download::from_url(&asset.download_url);
        download.set_header(
            reqwest::header::ACCEPT,
            "application/octet-stream".parse()?,
        );
        download.show_progress(false);
        download.download_to(&mut tmp_archive)?;

        // Extract
        let bin_name = if cfg!(target_os = "macos") {
            "Broquest.app/Contents/MacOS/Broquest"
        } else if cfg!(target_os = "windows") {
            "broquest.exe"
        } else {
            "broquest"
        };

        self_update::Extract::from_source(&tmp_archive_path).extract_file(&updates, bin_name)?;

        let _ = std::fs::remove_file(&tmp_archive_path);

        Ok(())
    }

    fn configure_platform_target(builder: &mut self_update::backends::github::UpdateBuilder) {
        if cfg!(target_os = "macos") {
            builder
                .target("macos")
                .identifier("arm64")
                .bin_path_in_archive("Broquest.app/Contents/MacOS/Broquest");
        } else if cfg!(target_os = "linux") {
            builder.target("linux");
        } else if cfg!(target_os = "windows") {
            builder.target("windows");
        }
    }

    pub fn apply_pending_update() {
        let staged = Self::staged_binary_path();
        if !staged.exists() {
            tracing::warn!("No staged update found at {:?}", staged);
            return;
        }

        // Store current version before replacing (for post-update notification)
        let current = env!("CARGO_PKG_VERSION").to_string();
        let _ = std::fs::write(Self::updates_dir().join(".updated_from"), &current);

        if let Err(e) = self_update::self_replace::self_replace(&staged) {
            tracing::error!("Failed to apply update: {}", e);
            return;
        }

        let _ = std::fs::remove_file(&staged);
        tracing::info!("Update applied, restarting...");
        Self::restart_app();
    }

    fn restart_app() {
        let exe = std::env::current_exe().expect("Failed to get current exe path");

        #[cfg(target_os = "macos")]
        {
            // On macOS, restart the .app bundle if we're inside one
            if let Some(app_bundle) = exe
                .parent()
                .and_then(|p| p.parent())
                .and_then(|p| p.parent())
            {
                if app_bundle.extension().map_or(false, |ext| ext == "app") {
                    let _ = std::process::Command::new("open").arg(app_bundle).spawn();
                    std::process::exit(0);
                }
            }
        }

        let _ = std::process::Command::new(exe).spawn();
        std::process::exit(0);
    }

    pub fn changelog_url(version: &str) -> String {
        format!(
            "https://github.com/{}/{}/releases/tag/{}",
            GITHUB_OWNER, GITHUB_REPO, version
        )
    }
}
