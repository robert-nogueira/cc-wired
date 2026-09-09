pub mod error;
pub mod fswatcher;
mod settings;
pub mod ws;

pub use error::Error;
pub use settings::*;
pub use ws::{ClientMessage, WsClient, WsError};

use fswatcher::FsWatcher;
use log::{debug, error, info, warn};
use notify::EventKind;
use std::path::PathBuf;
use tokio_tungstenite::tungstenite::Message;

pub async fn run(settings: Settings) -> Result<(), Error> {
    let _ = env_logger::Builder::from_env(
        env_logger::Env::default().default_filter_or("info"),
    )
    .try_init();

    let mut ws = WsClient::connect(
        settings.ws_url,
        std::time::Duration::from_secs(settings.connect_timeout_secs),
    )
    .await?;

    let mut watcher = FsWatcher::new(settings.watch.clone())?;

    info!(
        "cc-wired client started, watching {} target(s)",
        watcher.targets.len()
    );

    loop {
        tokio::select! {
            Some(res) = watcher.recv() => {
                match res {
                    Ok((computer_id, path, event)) => {
                        handle_fs_event(
                            &mut ws,
                            computer_id,
                            path,
                            event,
                        ).await;
                    }

                    Err(e) => {
                        error!("watch error: {e:?}");
                    }
                }
            }
            Some(msg) = ws.recv() => {
                match msg {
                    Ok(Message::Close(_)) | Err(_) => {
                        info!("connection lost, shutting down");
                        break;
                    }

                    Ok(other) => {
                        debug!("received: {other:?}");
                    }
                }
            }
            _ = tokio::signal::ctrl_c() => {
                info!("shutting down");
                let _ = ws.close().await;
                break;
            }
        }
    }

    Ok(())
}

async fn handle_fs_event(
    ws: &mut WsClient,
    computer_id: String,
    relative_path: PathBuf,
    event: notify::Event,
) {
    let path_str = relative_path.to_string_lossy().into_owned();

    let absolute = event.paths.first().cloned().unwrap_or_default();

    let message = match event.kind {
        EventKind::Create(_) => {
            let Ok(contents) = tokio::fs::read_to_string(&absolute).await
            else {
                warn!("failed to read {}, skipping", absolute.display());
                return;
            };
            info!("{path_str} created -> '{computer_id}'");
            ClientMessage::FileCreated {
                computer_id,
                path: path_str,
                contents,
            }
        }
        EventKind::Modify(_) => {
            let Ok(contents) = tokio::fs::read_to_string(&absolute).await
            else {
                warn!("failed to read {}, skipping", absolute.display());
                return;
            };
            info!("{path_str} changed -> '{computer_id}'");
            ClientMessage::FileChanged {
                computer_id,
                path: path_str,
                contents,
            }
        }
        EventKind::Remove(_) => {
            info!("{path_str} deleted -> '{computer_id}'");
            ClientMessage::FileDeleted {
                computer_id,
                path: path_str,
            }
        }
        _ => return,
    };

    match serde_json::to_string(&message) {
        Ok(json) => {
            if let Err(e) = ws.send(json.into()).await {
                warn!("failed to send update: {e:?}");
            }
        }

        Err(e) => {
            error!("failed to serialize message: {e:?}");
        }
    }
}
