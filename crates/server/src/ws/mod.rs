mod consumer;
mod producer;
mod registry;

use actix_web::web;

pub use registry::Registry;

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.route("/", web::get().to(producer::handle))
        .route("/{computer_id}", web::get().to(consumer::handle));
}
