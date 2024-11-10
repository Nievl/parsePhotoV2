use std::env;

use super::dto::CreateDto;
use crate::utils::get_now_time;
use log::{error, info};
use rusqlite::{params, Connection, Result};

pub struct MediafilesDbService {
    db_name: String,
}

impl MediafilesDbService {
    pub fn new() -> Self {
        let db_name = env::var("DB_NAME").expect("DB_NAME must be set");
        Self { db_name }
    }

    fn open_connection(&self) -> Result<Connection> {
        Connection::open(&self.db_name)
    }

    pub fn create_one(&self, dto: &CreateDto) -> Result<&str> {
        let mut conn = self.open_connection()?;
        let date_added = get_now_time();

        let tx = conn.transaction()?;

        match tx.execute(
            "INSERT INTO mediafiles (path, name, hash, size, date_added) VALUES(?, ?, ?, ?, ?)",
            params![dto.path, dto.name, dto.hash, dto.size, date_added],
        ) {
            Ok(_) => {
                // Получаем ID последней вставленной записи
                let mediafile_id = tx.last_insert_rowid();
                info!("Mediafile created with ID: {}", mediafile_id);
                return match tx.execute(
                    "INSERT INTO mediafiles_links (link_id, mediafile_id) VALUES (?, ?)",
                    params![dto.link_id, mediafile_id],
                ) {
                    Ok(changes) => {
                        if changes == 1 {
                            tx.commit()?;
                            Ok("One mediafile created")
                        } else {
                            Ok("No mediafile created")
                        }
                    }
                    Err(e) => {
                        error!("Error creating mediafile: {}", e);
                        Err(e)
                    }
                };
            }
            Err(e) => {
                error!("Error creating mediafile: {}", e);
                return Err(e);
            }
        };
    }

    pub fn remove(&self, id: usize) -> Result<&str> {
        let conn = self.open_connection()?;

        match conn.execute(
            "
            DELETE FROM mediafiles WHERE id = ?;
            DELETE FROM mediafiles_links WHERE mediafile_id = ?;
            ",
            [id, id],
        ) {
            Ok(changes) => {
                if changes == 1 {
                    return Ok("One mediafile removed");
                } else {
                    return Ok("No mediafile removed");
                }
            }
            Err(e) => {
                error!("Error removing mediafile: {}", e);
                return Err(e);
            }
        };
    }
}
