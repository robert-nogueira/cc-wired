use actix_web::{HttpRequest, HttpResponse, web};
use actix_ws::Message;
use cc_wired_protocol::ClientMessage;
use futures_util::StreamExt;
use log::{info, warn};

use super::registry::{Registry, RouteOutcome};

pub async fn handle(
    req: HttpRequest,
    body: web::Payload,
    registry: web::Data<Registry>,
) -> actix_web::Result<HttpResponse> {
    let client_id = req
        .headers()
        .get("x-client-id")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("unknown")
        .to_string();

    let (response, mut session, mut stream) = actix_ws::handle(&req, body)?;

    info!("producer '{client_id}' connected");

    actix_web::rt::spawn(async move {
        while let Some(Ok(msg)) = stream.next().await {
            match msg {
                Message::Text(text) => dispatch(&registry, &text).await,
                Message::Ping(bytes) => {
                    if session.pong(&bytes).await.is_err() {
                        break;
                    }
                }
                Message::Close(_) => break,
                _ => {}
            }
        }
        info!("producer '{client_id}' disconnected");
    });

    Ok(response)
}

async fn dispatch(registry: &Registry, text: &str) {
    let Ok(message) = serde_json::from_str::<ClientMessage>(text) else {
        warn!("dropping unparseable producer message");
        return;
    };
    let computer_id = message.computer_id();

    match registry.route(computer_id, text).await {
        RouteOutcome::Delivered | RouteOutcome::SendFailed => {}
        RouteOutcome::NotConnected => {
            warn!("no computer connected for id '{computer_id}'")
        }
    }
}
