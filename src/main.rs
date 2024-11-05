use axum::{response::Html, routing::get_service, Router, Server};
use log::{error, info};
use rusqlite::{Connection, Result};
use std::{
    net::{SocketAddr, TcpListener},
    path::Path,
};
use tower_http::services::{ServeDir, ServeFile};

mod links;
mod config;
use links::links_controller::links_routes;

#[tokio::main]
async fn main() {
    config::init();
    env_logger::init();

    info!("Starting server on PORT {}", *config::PORT);
    info!("Reading DB name {}", *config::DB_NAME);

    if let Err(e) = check_and_create_tables(&config::DB_NAME) {
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
        .merge(links_routes());

    let listener = TcpListener::bind(&addr).expect("Failed to bind PORT");

    info!("Listening on {}", addr);
    Server::from_tcp(listener)
        .unwrap()
        .serve(app.into_make_service())
        .await
        .unwrap();
}

fn check_and_create_tables(db_name: &str) -> Result<()> {
    info!("Checking database at {}", db_name);

    if !Path::new(db_name).exists() {
        error!("Database file {} does not exist", db_name);
        return Err(rusqlite::Error::SqliteFailure(
            rusqlite::ffi::Error::new(1),
            None,
        ));
    }

    let conn = Connection::open(db_name)?;
    conn.execute(
        "CREATE TABLE IF NOT EXISTS links (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            path TEXT NOT NULL UNIQUE,
            name TEXT,
            is_downloaded BOOLEAN NOT NULL DEFAULT 1,
            progress INTEGER DEFAULT 0,
            downloaded_mediafiles INTEGER DEFAULT 0,
            mediafiles INTEGER DEFAULT 0,
            date_update DATETIME DEFAULT CURRENT_TIMESTAMP,
            date_create DATETIME DEFAULT CURRENT_TIMESTAMP,
            is_reachable BOOLEAN NOT NULL DEFAULT 0
        )",
        [],
    )?;
    info!("Database tables checked and created if necessary");

    Ok(())
}
