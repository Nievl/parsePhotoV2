use log::info;
use sha2::{Digest, Sha256};

use super::{dto::CreateDto, mediafiles_db_service::MediafilesDbService};
use std::{fs::read, path::PathBuf, sync::Arc};

pub struct MediafilesService {
    mediafiles_db_service: Arc<MediafilesDbService>,
}

impl MediafilesService {
    pub fn new() -> Self {
        Self {
            mediafiles_db_service: Arc::new(MediafilesDbService::new()),
        }
    }

    pub async fn create_one(&self, dto: CreateDto) -> Result<String, String> {
        info!("creating mediafile, name: {}", &dto.name);

        self.mediafiles_db_service
            .create_one(&dto)
            .map(|s| s.to_string())
            .map_err(|e| e.to_string())
    }

    pub async fn remove(&self, id: usize) -> Result<String, String> {
        info!("Removing mediafile with id: {}", &id);

        self.mediafiles_db_service
            .remove(id)
            .map(|s| s.to_string())
            .map_err(|e| e.to_string())
    }
}

pub async fn calculate_hash_size(path: &PathBuf) -> Result<(String, usize), String> {
    let buffer = read(path).map_err(|err| err.to_string())?;
    let mut hasher = Sha256::new();

    hasher.update(&buffer);

    let hash_result = hasher.finalize();
    let hash_hex = format!("{:x}", hash_result);
    let size = buffer.len();

    Ok((hash_hex, size))
}