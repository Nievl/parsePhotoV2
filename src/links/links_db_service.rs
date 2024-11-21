use super::dto::Link;
use crate::utils::get_now_time;
use log::error;
use rusqlite::{params, Connection, Result};
use std::env;

pub struct LinksDbService {
    db_name: String,
}

impl LinksDbService {
    pub fn new() -> Self {
        let db_name = env::var("DB_NAME").expect("DB_NAME must be set");
        Self { db_name }
    }

    fn open_connection(&self) -> Result<Connection> {
        Connection::open(&self.db_name)
    }

    pub fn create_one(&self, path: &str, name: &str) -> Result<&str> {
        let conn = self.open_connection()?;

        return match conn.execute(
            "INSERT INTO links (path, name, is_reachable) VALUES (?, ?, 1)",
            params![path, name],
        ) {
            Ok(changes) => {
                if changes == 1 {
                    Ok("One path created")
                } else {
                    Ok("No path created")
                }
            }
            Err(e) => {
                error!("Error creating path: {}", e);
                Err(e)
            }
        };
    }

    pub fn get_all(&self, is_reachable: bool, show_duplicate: bool) -> Result<Vec<Link>> {
        let conn = self.open_connection()?;
        let query = if show_duplicate {
            "SELECT * FROM links WHERE is_reachable = ? AND duplicate_id IS NOT NULL ORDER BY is_downloaded"
        } else {
            "SELECT * FROM links WHERE is_reachable = ? AND duplicate_id IS NULL ORDER BY is_downloaded"
        };
        let mut stmt = conn.prepare(query)?;

        let rows = stmt.query_map([is_reachable], |row| {
            Ok(Link {
                id: row.get(0)?,
                path: row.get(1)?,
                name: row.get(2)?,
                is_downloaded: row.get(3)?,
                progress: row.get(4)?,
                downloaded_mediafiles: row.get(5)?,
                mediafiles: row.get(6)?,
                date_update: row.get(7)?,
                date_create: row.get(8)?,
                is_reachable: row.get(9)?,
                duplicate_id: row.get(10)?,
                duplicate_path: None,
            })
        })?;

        let result: Result<Vec<_>, _> = rows.collect();
        result
    }

    pub fn remove(&self, id: usize) -> Result<&str> {
        let conn = self.open_connection()?;

        return match conn.execute("DELETE FROM links WHERE id = ?", [id]) {
            Ok(changes) => {
                if changes == 1 {
                    Ok("One path removed")
                } else {
                    Ok("No path removed")
                }
            }
            Err(e) => {
                error!("Error removing path: {}", e);
                Err(e)
            }
        };
    }

    pub fn get_one(&self, id: usize) -> Result<Option<Link>> {
        let conn = self.open_connection()?;
        let mut stmt = conn.prepare("SELECT * FROM links WHERE id = ?")?;
        let mut rows = stmt.query([id])?;

        if let Some(row) = rows.next()? {
            let link = Link {
                id: row.get(0)?,
                path: row.get(1)?,
                name: row.get(2)?,
                is_downloaded: row.get(3)?,
                progress: row.get(4)?,
                downloaded_mediafiles: row.get(5)?,
                mediafiles: row.get(6)?,
                date_update: row.get(7)?,
                date_create: row.get(8)?,
                is_reachable: row.get(9)?,
                duplicate_id: row.get(10)?,
                duplicate_path: None,
            };
            Ok(Some(link))
        } else {
            Ok(None)
        }
    }

    pub fn tag_unreachable(&self, id: usize, is_reachable: bool) -> Result<String> {
        let conn = self.open_connection()?;
        let changes = conn.execute(
            "UPDATE links SET is_reachable = ?, date_update = ? WHERE id = ?",
            params![is_reachable, get_now_time(), id],
        )?;

        Ok(if changes == 1 {
            format!(
                "One path tagged as {}",
                if is_reachable {
                    "reachable"
                } else {
                    "unreachable"
                }
            )
        } else {
            "No path tagged".to_string()
        })
    }

    pub fn add_duplicate(&self, link_id: usize, duplicate_id: usize) -> Result<String> {
        let conn = self.open_connection()?;
        let changes = conn.execute(
            "UPDATE links SET duplicate_id = ?, date_update = ? WHERE id = ?",
            params![duplicate_id, get_now_time(), link_id],
        )?;

        Ok(if changes == 1 {
            format!(
                "Link id {} tagged as duplicate of {}",
                link_id, duplicate_id
            )
        } else {
            "No path tagged".to_string()
        })
    }

    pub fn update_files_number(
        &self,
        id: usize,
        mediafiles: usize,
        downloaded_mediafiles: usize,
        is_downloaded: bool,
        progress: usize,
    ) -> Result<String> {
        let conn = self.open_connection()?;

        let changes = conn.execute(
            "UPDATE links SET mediafiles = ?, downloaded_mediafiles = ?, is_downloaded = ?, progress = ?, date_update = ? WHERE id = ?",
            params![mediafiles, downloaded_mediafiles, is_downloaded, progress, get_now_time(), id],
        )?;
        Ok(if changes == 1 {
            "One path updated".to_string()
        } else {
            "No path updated".to_string()
        })
    }
}
