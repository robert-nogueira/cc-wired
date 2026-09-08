use actix_web::{HttpRequest, HttpResponse, web};
use actix_ws::Message;
use futures_util::StreamExt;
use log::info;

use super::registry::Registry;

pub async fn handle(
    req: HttpRequest,
    body: web::Payload,
    path: web::Path<String>,
    registry: web::Data<Registry>,
) -> actix_web::Result<HttpResponse> {
    let computer_id = path.into_inner();
    let (response, mut session, mut stream) = actix_ws::handle(&req, body)?;

    registry.register(computer_id.clone(), session.clone());
    info!("computer '{computer_id}' connected");

    actix_web::rt::spawn(async move {
        while let Some(Ok(msg)) = stream.next().await {
            match msg {
                Message::Ping(bytes) => {
                    if session.pong(&bytes).await.is_err() {
                        break;
                    }
                }
                Message::Close(_) => break,
                _ => {}
            }
        }
        registry.unregister(&computer_id);
        info!("computer '{computer_id}' disconnected");
    });

    Ok(response)
}
