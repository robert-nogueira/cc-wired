pub mod fswatcher;
mod settings;
pub mod ws;

pub use settings::*;
pub use ws::{ClientMessage, WsClient, WsError};

use fswatcher::FsWatcher;
use notify::EventKind;
use std::error::Error;
use tokio_tungstenite::tungstenite::Message;

pub async fn run() -> Result<(), Box<dyn Error>> {
    let mut ws = WsClient::connect().await?;
    let mut watcher = FsWatcher::new(SETTINGS.watch.clone())?;

    println!(
        "cc-wired client started, watching {} target(s)",
        SETTINGS.watch.len()
    );

    loop {
        tokio::select! {
            Some(res) = watcher.recv() => {
                match res {
                    Ok((computer_id, event)) => handle_fs_event(&mut ws, computer_id, event).await,
                    Err(e) => eprintln!("watch error: {e:?}"),
                }
            }
            Some(msg) = ws.recv() => {
                match msg {
                    Ok(Message::Close(_)) | Err(_) => {
                        println!("connection lost, shutting down");
                        break;
                    }
                    Ok(other) => println!("received: {other:?}"),
                }
            }
            _ = tokio::signal::ctrl_c() => {
                println!("shutting down");
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
    event: notify::Event,
) {
    for path in &event.paths {
        let path_str = path.to_string_lossy().into_owned();

        let message = match event.kind {
            EventKind::Create(_) => {
                let Ok(contents) = tokio::fs::read_to_string(path).await
                else {
                    continue;
                };
                ClientMessage::FileCreated {
                    computer_id: computer_id.clone(),
                    path: path_str,
                    contents,
                }
            }
            EventKind::Modify(_) => {
                let Ok(contents) = tokio::fs::read_to_string(path).await
                else {
                    continue;
                };
                ClientMessage::FileChanged {
                    computer_id: computer_id.clone(),
                    path: path_str,
                    contents,
                }
            }
            EventKind::Remove(_) => ClientMessage::FileDeleted {
                computer_id: computer_id.clone(),
                path: path_str,
            },
            _ => continue,
        };

        match serde_json::to_string(&message) {
            Ok(ws_msg) => {
                if let Err(e) = ws.send(ws_msg.into()).await {
                    eprintln!("failed to send update: {e:?}");
                }
            }
            Err(e) => eprintln!("failed to serialize message: {e:?}"),
        }
    }
}
