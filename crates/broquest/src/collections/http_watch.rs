//! Reloading a collection of `.http` files when its files change on disk.

use std::path::{Path, PathBuf};
use std::time::Duration;

use gpui::{Context, Task};
use notify::Watcher as _;

use super::manager::CollectionManager;

/// Keeps a folder watched for as long as it is held.
pub struct HttpWatch {
    _watcher: notify::RecommendedWatcher,
    _task: Task<()>,
}

/// Files whose changes reload the collection.
fn is_relevant(path: &Path) -> bool {
    if matches!(
        path.extension().and_then(|ext| ext.to_str()),
        Some("http" | "rest")
    ) {
        return true;
    }
    matches!(
        path.file_name().and_then(|name| name.to_str()),
        Some("http-client.env.json" | "http-client.private.env.json" | ".env" | "settings.json")
    )
}

/// Watch the collection at `collection_path` and reload it after changes.
pub fn watch(collection_path: String, cx: &mut Context<CollectionManager>) -> Option<HttpWatch> {
    let dir = PathBuf::from(&collection_path);
    let (tx, rx) = smol::channel::bounded::<()>(1);
    let mut watcher = notify::RecommendedWatcher::new(
        move |event: notify::Result<notify::Event>| match event {
            Ok(event) if event.need_rescan() || event.paths.iter().any(|p| is_relevant(p)) => {
                let _ = tx.try_send(());
            }
            Ok(_) => {}
            Err(error) => tracing::warn!("File watch error: {error}"),
        },
        notify::Config::default().with_follow_symlinks(false),
    )
    .map_err(|error| tracing::warn!("Could not watch {}: {error}", dir.display()))
    .ok()?;
    watcher
        .watch(&dir, notify::RecursiveMode::Recursive)
        .map_err(|error| tracing::warn!("Could not watch {}: {error}", dir.display()))
        .ok()?;

    let task = cx.spawn(async move |manager, cx| {
        while rx.recv().await.is_ok() {
            // Let a burst of changes, such as an editor's save, settle.
            smol::Timer::after(Duration::from_millis(250)).await;
            while rx.try_recv().is_ok() {}
            let reloaded = manager.update(cx, |manager, cx| {
                manager.reload_http_collection(&collection_path, cx);
            });
            if reloaded.is_err() {
                break;
            }
        }
    });

    Some(HttpWatch {
        _watcher: watcher,
        _task: task,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_request_and_environment_files_count() {
        for path in [
            "/c/auth.http",
            "/c/sub/list.rest",
            "/c/http-client.env.json",
            "/c/http-client.private.env.json",
            "/c/.env",
            "/c/.vscode/settings.json",
        ] {
            assert!(is_relevant(Path::new(path)), "{path}");
        }
        for path in ["/c/readme.md", "/c/out/response.json", "/c/.git/index"] {
            assert!(!is_relevant(Path::new(path)), "{path}");
        }
    }
}
