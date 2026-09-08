use crate::settings::Settings;
use crate::ws::{self, Registry};
use actix_web::{App, HttpServer, web};
use log::info;

pub async fn run(settings: Settings) -> std::io::Result<()> {
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
                    .configure(ws::configure)
            })
            .bind(addr)?
            .run()
            .await
        })
    })
    .await
    .expect("actix server thread panicked")
}
