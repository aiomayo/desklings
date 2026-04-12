use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{mpsc, Arc};
use std::time::{Duration, Instant};

use arc_swap::ArcSwap;
use notify::{EventKind, RecursiveMode, Watcher};
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter};
use tracing::{info, warn};

use crate::error::DesklingResult;
use crate::state::RuntimeMap;

use super::compiled::CompiledConfig;
use super::loader::{load_deskling_by_slug, LoadedDeskling};
use super::settings::SettingsStore;

pub const RELOAD_EVENT: &str = "deskling_reloaded";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DesklingReloadedEvent {
    pub slug: String,
}

const DEBOUNCE: Duration = Duration::from_millis(250);

#[derive(Debug)]
pub struct LiveConfig {
    inner: ArcSwap<CompiledConfig>,
    version: AtomicU64,
}

impl LiveConfig {
    #[must_use]
    pub fn new(initial: CompiledConfig) -> Arc<Self> {
        Arc::new(Self {
            inner: ArcSwap::from_pointee(initial),
            version: AtomicU64::new(0),
        })
    }

    #[must_use]
    pub fn load(&self) -> Arc<CompiledConfig> {
        self.inner.load_full()
    }

    #[must_use]
    pub fn version(&self) -> u64 {
        self.version.load(Ordering::Acquire)
    }

    pub fn store(&self, next: CompiledConfig) {
        self.inner.store(Arc::new(next));
        self.version.fetch_add(1, Ordering::AcqRel);
    }
}

pub fn spawn_watcher(
    runtimes: RuntimeMap,
    _settings: Arc<SettingsStore>,
    desklings_dir: &Path,
    app: AppHandle,
) -> DesklingResult<()> {
    let (tx, rx) = mpsc::channel();

    let mut watcher = notify::recommended_watcher(move |res| {
        let _ = tx.send(res);
    })?;

    watcher.watch(desklings_dir, RecursiveMode::Recursive)?;
    info!(dir = %desklings_dir.display(), "hot-reload watcher started");

    let desklings_dir_owned = desklings_dir.to_path_buf();
    std::thread::Builder::new()
        .name("desklings-hot-reload".into())
        .spawn(move || {
            let desklings_dir = desklings_dir_owned;
            let _watcher = watcher;
            let mut last_reload = Instant::now()
                .checked_sub(DEBOUNCE)
                .unwrap_or_else(Instant::now);
            let mut pending_slugs: HashSet<String> = HashSet::new();

            for event in rx {
                let Ok(event) = event else {
                    continue;
                };

                if !matches!(
                    event.kind,
                    EventKind::Modify(_) | EventKind::Create(_) | EventKind::Remove(_)
                ) {
                    continue;
                }

                for path in &event.paths {
                    if let Some(slug) = slug_for_path(&desklings_dir, path) {
                        pending_slugs.insert(slug);
                    }
                }

                let now = Instant::now();
                if now.duration_since(last_reload) < DEBOUNCE {
                    continue;
                }
                last_reload = now;

                let slugs_to_reload: Vec<String> = pending_slugs.drain().collect();
                for slug in slugs_to_reload {
                    let targets: Vec<(Arc<LiveConfig>, String)> = {
                        let guard = match runtimes.lock() {
                            Ok(g) => g,
                            Err(_) => continue,
                        };
                        guard
                            .values()
                            .filter(|rt| rt.slug == slug)
                            .map(|rt| (rt.live(), rt.window_label.clone()))
                            .collect()
                    };

                    if targets.is_empty() {
                        continue;
                    }

                    match load_deskling_by_slug(&desklings_dir, &slug) {
                        Ok(LoadedDeskling { config, slug: reloaded_slug, .. }) => {
                            info!(
                                %reloaded_slug,
                                instances = targets.len(),
                                "hot-reload: deskling reloaded"
                            );
                            for (live, window_label) in &targets {
                                live.store(config.clone());
                                let payload = DesklingReloadedEvent {
                                    slug: reloaded_slug.clone(),
                                };
                                if let Err(e) =
                                    app.emit_to(window_label, RELOAD_EVENT, &payload)
                                {
                                    warn!(error = %e, "hot-reload: failed to emit event");
                                }
                            }
                        }
                        Err(e) => {
                            warn!(error = %e, %slug, "hot-reload: reload failed, keeping old config");
                        }
                    }
                }
            }
        })
        .map_err(|source| crate::error::DesklingError::io(desklings_dir, source))?;

    Ok(())
}

fn slug_for_path(desklings_dir: &Path, path: &PathBuf) -> Option<String> {
    let rel = path.strip_prefix(desklings_dir).ok()?;
    rel.components()
        .next()
        .and_then(|c| c.as_os_str().to_str())
        .map(str::to_string)
}
