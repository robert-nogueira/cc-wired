use notify::{RecommendedWatcher, RecursiveMode, Watcher};
use std::path::{Path, PathBuf};
use tokio::sync::mpsc;

#[derive(Debug, serde::Deserialize)]
pub struct WatchTarget {
    pub dir: PathBuf,
    pub computer_id: String,
}

pub struct FsWatcher {
    _watcher: RecommendedWatcher,
    events: mpsc::UnboundedReceiver<notify::Result<notify::Event>>,
    targets: Vec<WatchTarget>,
}

impl FsWatcher {
    pub fn new(targets: Vec<WatchTarget>) -> notify::Result<Self> {
        let (tx, events) = mpsc::unbounded_channel();

        let mut watcher = notify::recommended_watcher(move |res| {
            let _ = tx.send(res);
        })?;

        for target in &targets {
            watcher.watch(&target.dir, RecursiveMode::Recursive)?;
        }

        Ok(Self {
            _watcher: watcher,
            events,
            targets,
        })
    }

    pub async fn recv(
        &mut self,
    ) -> Option<notify::Result<(String, notify::Event)>> {
        let res = self.events.recv().await?;

        Some(res.map(|event| {
            let computer_id = event
                .paths
                .first()
                .and_then(|p| self.target_for(p))
                .unwrap_or_default();

            (computer_id, event)
        }))
    }

    fn target_for(&self, path: &Path) -> Option<String> {
        self.targets
            .iter()
            .find(|t| path.starts_with(&t.dir))
            .map(|t| t.computer_id.clone())
    }
}
