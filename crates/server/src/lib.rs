pub mod settings;

use actix_web::{App, HttpRequest, HttpResponse, HttpServer, web};
use actix_ws::Message;
use futures_util::StreamExt;
use log::{info, warn};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Mutex;

type Registry = Mutex<HashMap<String, actix_ws::Session>>;

pub async fn run(settings: settings::Settings) -> std::io::Result<()> {
    let _ = env_logger::try_init();
    let addr = format!("{}:{}", settings.host, settings.port);

    info!("Listening on: {addr}");

    // actix-web's server future holds !Send state (its per-worker LocalSets),
    // so it has to be driven by its own actix_rt::System on a dedicated
    // thread rather than awaited directly from our Send-bound async_trait.
    tokio::task::spawn_blocking(move || {
        actix_web::rt::System::new().block_on(async move {
            let registry = web::Data::new(Registry::default());

            HttpServer::new(move || {
                App::new()
                    .app_data(registry.clone())
                    .route("/", web::get().to(producer_ws))
                    .route("/{computer_id}", web::get().to(consumer_ws))
            })
            .bind(addr)?
            .run()
            .await
        })
    })
    .await
    .expect("actix server thread panicked")
}

async fn producer_ws(
    req: HttpRequest,
    body: web::Payload,
    registry: web::Data<Registry>,
) -> actix_web::Result<HttpResponse> {
    let (response, mut session, mut stream) = actix_ws::handle(&req, body)?;

    actix_web::rt::spawn(async move {
        while let Some(Ok(msg)) = stream.next().await {
            match msg {
                Message::Text(text) => {
                    route_to_consumer(&registry, &text).await
                }
                Message::Ping(bytes) => {
                    if session.pong(&bytes).await.is_err() {
                        break;
                    }
                }
                Message::Close(_) => break,
                _ => {}
            }
        }
    });

    Ok(response)
}

async fn route_to_consumer(registry: &Registry, text: &str) {
    let Ok(value) = serde_json::from_str::<Value>(text) else {
        warn!("dropping non-JSON producer message");
        return;
    };
    let Some(computer_id) = value.get("computer_id").and_then(Value::as_str)
    else {
        warn!("dropping producer message without computer_id");
        return;
    };

    let target = registry.lock().unwrap().get(computer_id).cloned();
    match target {
        Some(mut consumer) => {
            if consumer.text(text.to_owned()).await.is_err() {
                registry.lock().unwrap().remove(computer_id);
            }
        }
        None => warn!("no computer connected for id '{computer_id}'"),
    }
}

async fn consumer_ws(
    req: HttpRequest,
    body: web::Payload,
    path: web::Path<String>,
    registry: web::Data<Registry>,
) -> actix_web::Result<HttpResponse> {
    let computer_id = path.into_inner();
    let (response, mut session, mut stream) = actix_ws::handle(&req, body)?;

    registry
        .lock()
        .unwrap()
        .insert(computer_id.clone(), session.clone());
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
        registry.lock().unwrap().remove(&computer_id);
        info!("computer '{computer_id}' disconnected");
    });

    Ok(response)
}
