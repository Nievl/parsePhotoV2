use log::{error, info};
use rusqlite::{Connection, Result};
use std::path::Path;

use crate::config;

pub fn init_db_tables() -> Result<()> {
    let db_name = config::DB_NAME.clone();
    info!("Checking database at {}", db_name);

    if !Path::new(&db_name).exists() {
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

    conn.execute(
        "CREATE TABLE IF NOT EXISTS mediafiles (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            path TEXT NOT NULL UNIQUE,
            name TEXT NOT NULL,
            hash TEXT NOT NULL,
            size INTEGER NOT NULL,
            date_added DATETIME DEFAULT CURRENT_TIMESTAMP
        )",
        [],
    )?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS mediafiles_links (
            link_id INTEGER NOT NULL,
            mediafile_id INTEGER NOT NULL,
            FOREIGN KEY (link_id) REFERENCES links(id) ON DELETE CASCADE,
            FOREIGN KEY (mediafile_id) REFERENCES mediafiles(id) ON DELETE CASCADE,
            PRIMARY KEY (link_id, mediafile_id)
        )",
        [],
    )?;

    info!("Database tables checked and created if necessary");

    Ok(())
}
