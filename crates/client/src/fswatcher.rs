// fswatcher.rs
use notify::{Event, RecommendedWatcher, RecursiveMode, Watcher};
use std::path::{Path, PathBuf};
use tokio::sync::mpsc;

#[derive(Debug, Clone, serde::Deserialize)]
pub struct WatchTarget {
    pub dir: PathBuf,
    pub computer_id: String,
}

pub struct FsWatcher {
    _watcher: RecommendedWatcher,
    events: mpsc::UnboundedReceiver<notify::Result<Event>>,
    pub targets: Vec<WatchTarget>,
}

impl FsWatcher {
    pub fn new(targets: Vec<WatchTarget>) -> notify::Result<Self> {
        let (tx, events) = mpsc::unbounded_channel();

        let mut watcher = notify::recommended_watcher(move |res| {
            let _ = tx.send(res);
        })?;

        let mut resolved = Vec::with_capacity(targets.len());
        for target in targets {
            let canonical = match std::fs::canonicalize(&target.dir) {
                Ok(path) => path,
                Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                    eprintln!(
                        "warning: ignoring {} ({e})",
                        target.dir.display()
                    );
                    continue;
                }
                Err(e) => return Err(e.into()),
            };
            watcher.watch(&canonical, RecursiveMode::Recursive)?;
            resolved.push(WatchTarget {
                dir: canonical,
                ..target
            });
        }

        if resolved.is_empty() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "no valid watch directories in config",
            )
            .into());
        }

        Ok(Self {
            _watcher: watcher,
            events,
            targets: resolved,
        })
    }

    pub async fn recv(
        &mut self,
    ) -> Option<notify::Result<(String, PathBuf, Event)>> {
        let res = self.events.recv().await?;

        Some(res.map(|event| {
            let path = event.paths.first().cloned().unwrap_or_default();
            let (computer_id, relative) =
                self.resolve(&path).unwrap_or((String::new(), path.clone()));

            (computer_id, relative, event)
        }))
    }

    fn resolve(&self, path: &Path) -> Option<(String, PathBuf)> {
        self.targets
            .iter()
            .find(|t| path.starts_with(&t.dir))
            .map(|t| {
                let relative =
                    path.strip_prefix(&t.dir).unwrap_or(path).to_path_buf();
                (t.computer_id.clone(), relative)
            })
    }
}
