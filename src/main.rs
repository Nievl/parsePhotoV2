use axum::{response::Html, routing::get_service, Router, Server};
use log::{error, info};
use std::net::{SocketAddr, TcpListener};
use tower_http::services::{ServeDir, ServeFile};

mod config;
mod init_db;
mod links;
mod mediafiles;
mod utils;
use init_db::init_db_tables;
use links::links_controller::links_routes;
use mediafiles::mediafiles_controller::mediafiles_routes;

#[tokio::main]
async fn main() {
    config::init();
    env_logger::init();

    info!("Starting server on PORT {}", *config::PORT);

    if let Err(e) = init_db_tables() {
        error!("Error creating tables: {}", e);
        return;
    }

    let addr = SocketAddr::from(([127, 0, 0, 1], *config::PORT));
    let app = Router::new()
        .route(
            "/",
            get_service(ServeFile::new("web/index.html"))
                .handle_error(|_| async { Html("Error loading index.html") }),
        )
        .nest_service("/static", ServeDir::new("web/static"))
        .merge(links_routes())
        .merge(mediafiles_routes());

    let listener = TcpListener::bind(&addr).expect("Failed to bind PORT");

    info!("Listening on {}", addr);
    Server::from_tcp(listener)
        .unwrap()
        .serve(app.into_make_service())
        .await
        .unwrap();
}
