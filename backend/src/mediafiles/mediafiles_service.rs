use sha2::{Digest, Sha256};

use super::{
    dto::{CreateDto, Mediafile},
    mediafiles_db_service::MediafilesDbService,
};
use std::{
    fs::{read, File},
    io::Write,
    path::{Path, PathBuf},
    sync::Arc,
};

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
        self.mediafiles_db_service
            .create_one(&dto)
            .map(|s| s.to_string())
            .map_err(|e| e.to_string())
    }

    pub async fn remove(&self, id: usize) -> Result<String, String> {
        self.mediafiles_db_service
            .remove(id)
            .map(|s| s.to_string())
            .map_err(|e| e.to_string())
    }

    pub async fn get_all_by_link_id(&self, link_id: usize) -> Result<Vec<Mediafile>, String> {
        self.mediafiles_db_service
            .get_all_by_link_id(link_id)
            .map_err(|e| e.to_string())
    }
}

pub async fn get_hash_size_by_path(path: &PathBuf) -> Result<(String, usize), String> {
    let buffer = read(path).map_err(|err| err.to_string())?;
    let (hash, size) = calculate_hash_size(&buffer).await;

    Ok((hash, size))
}

pub async fn download_file(
    url: &str,
    file_path: &Path,
    link_id: usize,
) -> Result<CreateDto, String> {
    let response = fetch_and_write_file(url, file_path).await?;
    let (hash, size) = calculate_hash_size(&response).await;
    let name = file_path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or_default()
        .to_string();

    Ok(CreateDto {
        name,
        path: file_path.to_string_lossy().into_owned(),
        hash,
        size,
        link_id,
    })
}

pub async fn fetch_and_write_file(url: &str, file_path: &Path) -> Result<Vec<u8>, String> {
    let response = reqwest::get(url)
        .await
        .map_err(|e| format!("Request failed: {}", e))?
        .bytes()
        .await
        .map_err(|e| format!("Failed to read bytes: {}", e))?;

    let mut file = File::create(file_path).map_err(|e| format!("Failed to create file: {}", e))?;

    file.write_all(&response)
        .map_err(|e| format!("Failed to write to file: {}", e))?;
    file.flush()
        .map_err(|e| format!("Failed to flush file: {}", e))?;

    Ok(response.to_vec())
}

pub async fn calculate_hash_size(data: &Vec<u8>) -> (String, usize) {
    let mut hasher = Sha256::new();

    hasher.update(data);

    let hash = format!("{:x}", hasher.finalize());
    let size = data.len();

    (hash, size)
}
