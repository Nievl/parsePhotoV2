use super::config;
use crate::{
    mediafiles::{
        self,
        mediafiles_service::{calculate_hash_size, MediafilesService},
    },
    utils::{error_response, server_error_response, success_response},
};
use axum::{http::StatusCode, response::IntoResponse, Json};
use futures::stream::{FuturesUnordered, StreamExt};
use log::{error, info, warn};
use regex::Regex;
use reqwest;
use select::{document::Document, predicate::Name};
use std::{
    fs,
    io::Write,
    path::{Path, PathBuf},
    sync::Arc,
};
use tokio::spawn;

use super::dto::{CreateLinkDto, DownloadedFiles};
use super::links_db_service::LinksDbService;

#[derive(Clone)]
pub struct LinksService {
    links_db_service: Arc<LinksDbService>,
    mediafiles_service: Arc<MediafilesService>,
}

impl LinksService {
    pub fn new() -> Self {
        Self {
            links_db_service: Arc::new(LinksDbService::new()),
            mediafiles_service: Arc::new(MediafilesService::new()),
        }
    }

    pub async fn create_one(&self, dto: CreateLinkDto) -> impl IntoResponse {
        if let Some(url_parts) = check_url(&dto.path) {
            let name = url_parts[1].trim();
            info!("creating link, name: {}, path: {}", &name, &dto.path);

            match &self.links_db_service.create_one(&dto.path, name) {
                Ok(m) => Ok(success_response(m.to_string())),
                Err(e) => Err(server_error_response(e.to_string())),
            }
        } else {
            Err(error_response(
                "Path is not a valid URL".to_string(),
                StatusCode::BAD_REQUEST,
            ))
        }
    }

    pub async fn get_all(&self, is_reachable: bool) -> impl IntoResponse {
        info!("Getting all links is_reachable: {}", &is_reachable);

        return match self.links_db_service.get_all(is_reachable) {
            Ok(links) => Ok((StatusCode::OK, Json(links))),
            Err(e) => {
                error!("Error getting links: {}", e);
                Err(server_error_response("Error getting links".to_string()))
            }
        };
    }

    pub async fn remove(&self, id: usize) -> impl IntoResponse {
        info!("Removing link with id: {}", &id);

        match &self.links_db_service.remove(id) {
            Ok(m) => Ok(success_response(m.to_string())),
            Err(e) => Err(e.to_string()),
        }
    }

    pub async fn tag_unreachable(&self, id: usize, is_reachable: bool) -> impl IntoResponse {
        info!("Tagging link with id: {} as {}", &id, &is_reachable);

        match self.links_db_service.tag_unreachable(id, is_reachable) {
            Ok(m) => Ok(success_response(m)),
            Err(e) => Err(e.to_string()),
        }
    }

    pub async fn download(&self, id: usize) -> impl IntoResponse {
        info!("Downloading link with id: {}", &id);

        let link = match self.links_db_service.get_one(id) {
            Ok(link) => match link {
                Some(link) => link,
                None => {
                    info!("Link with id {} not found", &id);
                    return Err(error_response(
                        "Link not found".to_string(),
                        StatusCode::NOT_FOUND,
                    ));
                }
            },
            Err(e) => return Err(server_error_response(e.to_string())),
        };

        info!("Link with path: {} exist in DB", &link.path);

        let page = get_page(&link.path)
            .await
            .map_err(|e| (error_response(e.to_string(), StatusCode::NOT_FOUND)))?;

        let media_urls = get_media_urls(&page);

        info!(
            "Media urls count: {} on page {}",
            &media_urls.len(),
            &link.path
        );

        let dir_path = create_directory(&link.name)
            .await
            .map_err(|e| server_error_response(e))?;

        let downloaded_files = download_files_multi(media_urls, &dir_path, link.id)
            .await
            .map_err(|e| {
                error!("Error downloading files: {}", e);
                server_error_response(e)
            })?;

        info!(
            "Downloaded files: {}, from {}",
            &downloaded_files.downloaded, &downloaded_files.total
        );

        let progress = calculate_progress(downloaded_files.total, downloaded_files.downloaded);

        let is_downloaded = downloaded_files.downloaded == downloaded_files.total;
        return match self.links_db_service.update_files_number(
            id,
            downloaded_files.downloaded,
            downloaded_files.total,
            is_downloaded,
            progress,
        ) {
            Ok(_) => Ok(success_response(format!(
                "Downloaded {} files",
                downloaded_files.downloaded
            ))),
            Err(e) => Err(server_error_response(e.to_string())),
        };
    }

    pub async fn check_downloaded(&self, id: usize) -> impl IntoResponse {
        info!("Checking if link with id: {} is downloaded", &id);

        let link = match self.links_db_service.get_one(id) {
            Ok(link) => match link {
                Some(link) => link,
                None => {
                    info!("Link with id {} not found", &id);
                    return Err(error_response(
                        "Link not found".to_string(),
                        StatusCode::NOT_FOUND,
                    ));
                }
            },
            Err(e) => return Err(server_error_response(e.to_string())),
        };

        info!("Link with path: {} exist in DB", &link.path);

        let dir_path = Path::new("result").join(&link.name);
        let dir_exists = dir_path.exists();

        if dir_exists {
            info!("Directory: {} exists", &dir_path.to_string_lossy());
        }

        let page = match get_page(&link.path).await {
            Ok(page) => Some(page),
            Err(_) => None,
        };

        if page.is_some() {
            info!("Page: {} is exists", &link.path);
        }

        if !dir_exists && page.is_none() {
            return Ok(success_response(format!(
                "{} does not exist and page not found",
                dir_path.display()
            )));
        } else if dir_exists && page.is_none() {
            return match self
                .handle_downloaded_dir_without_page(link.id, &dir_path)
                .await
            {
                Ok(m) => Ok(success_response(m)),
                Err(e) => Err(server_error_response(e)),
            };
        } else if dir_exists && page.is_some() {
            return match self
                .handle_dir_and_page(link.id, &dir_path, page.as_ref().unwrap())
                .await
            {
                Ok(m) => Ok(success_response(m)),
                Err(e) => Err(server_error_response(e)),
            };
        } else {
            return match self
                .handle_page_without_dir(link.id, page.as_ref().unwrap())
                .await
            {
                Ok(m) => Ok(success_response(m)),
                Err(e) => Err(server_error_response(e)),
            };
        }
    }

    pub async fn add_files_to_link(&self, id: usize) -> impl IntoResponse {
        info!("Adding files to link with id: {}", &id);

        let link = match self.links_db_service.get_one(id) {
            Ok(link) => match link {
                Some(link) => link,
                None => {
                    return Err(error_response(
                        "Link not found".to_string(),
                        StatusCode::NOT_FOUND,
                    ))
                }
            },
            Err(e) => return Err(server_error_response(e.to_string())),
        };

        let dir_path = Path::new("result").join(&link.name);
        info!("Directory path: {}", &dir_path.to_string_lossy());
        let mediafiles_names = match fs::read_dir(dir_path) {
            Ok(entries) => entries,
            Err(e) => {
                return Err(server_error_response(format!(
                    "Failed to read directory: {}",
                    e
                )))
            }
        };

        for mediafile_name in mediafiles_names {
            let mediafile_name = match mediafile_name {
                Ok(entry) => entry,
                Err(e) => {
                    error!("Error reading directory entry: {}", e);
                    continue;
                }
            };
            let file_path = mediafile_name.path();

            let (hash, size) = match calculate_hash_size(&file_path).await {
                Ok((hash, size)) => (hash, size),
                Err(op) => {
                    error!("Error calculating hash and size: {}", op);
                    continue;
                }
            };
            info!("Hash: {}", hash);
            info!("Size: {}", size);

            match self
                .mediafiles_service
                .create_one(mediafiles::dto::CreateDto {
                    name: file_path.file_name().unwrap().to_string_lossy().to_string(),
                    path: file_path.to_string_lossy().to_string(),
                    hash,
                    size,
                    link_id: link.id,
                })
                .await
            {
                Ok(_) => info!(
                    "Record for {} file created successfully",
                    file_path.display()
                ),
                Err(e) => error!("Error creating mediafile: {}", e),
            };
        }

        Ok(success_response("".to_string()))
    }
    async fn handle_downloaded_dir_without_page(
        &self,
        link_id: usize,
        dir_path: &Path,
    ) -> Result<String, String> {
        let mediafiles = fs::read_dir(dir_path).unwrap().count();

        if mediafiles > 0 {
            match self
                .links_db_service
                .update_files_number(link_id, 0, mediafiles, true, 100)
            {
                Ok(_) => Ok(format!(
                    "id: {}, Files detected in directory, download marked as complete",
                    link_id,
                )),
                Err(e) => Err(e.to_string()),
            }
        } else {
            Ok(format!("{} directory is empty", dir_path.display()))
        }
    }

    async fn handle_dir_and_page(
        &self,
        link_id: usize,
        dir_path: &Path,
        page: &str,
    ) -> Result<String, String> {
        let mediafiles = count_media_files(page);
        let existed_files_count = fs::read_dir(dir_path).unwrap().count();
        let progress = (existed_files_count * 100) / mediafiles;
        let is_downloaded = existed_files_count == mediafiles;

        match self.links_db_service.update_files_number(
            link_id,
            mediafiles,
            existed_files_count,
            is_downloaded,
            progress,
        ) {
            Ok(_) => Ok(format!(
                "id: {}, Downloaded {} out of {} media files",
                link_id, existed_files_count, mediafiles
            )),
            Err(e) => Err(e.to_string()),
        }
    }

    async fn handle_page_without_dir(&self, link_id: usize, page: &str) -> Result<String, String> {
        let mediafiles = count_media_files(page);

        match self
            .links_db_service
            .update_files_number(link_id, mediafiles, 0, false, 0)
        {
            Ok(_) => Ok(format!("id: {}, Not downloaded yet", link_id)),
            Err(e) => Err(e.to_string()),
        }
    }
}

async fn download_files_multi(
    urls: Vec<String>,
    dir_path: &Path,
    link_id: usize,
) -> Result<DownloadedFiles, String> {
    let total_count = urls.len();
    let download_futures = urls.into_iter().map(|url| {
        let dir_path = dir_path.to_path_buf(); // Клонируем путь для использования в разных потоках

        spawn(async move {
            let file_name = Regex::new(r".+/").unwrap().replace(&url, "").to_string();
            let file_path = dir_path.join(&file_name);
            let use_root_url = !url.starts_with("http");

            if file_path.exists() {
                return Ok::<usize, String>(1); // Файл уже существует, пропускаем
            }

            if !is_valid_extension(&file_name) {
                warn!("{} is not an image", file_name);
                return Ok(0);
            }

            let download_url = if use_root_url {
                format!("{}{}", *config::ROOT_URL, url)
            } else {
                url.clone()
            };

            info!("Downloading {} to {}", &download_url, file_path.display());
            match download_file(&download_url, &file_path, link_id).await {
                Ok(_) => Ok(1), // Успешно скачан один файл
                Err(e) => {
                    error!("Failed to download {}: {}", url, e);
                    Ok(0)
                }
            }
        })
    });

    // Запускаем все загрузки параллельно
    let results: Vec<_> = FuturesUnordered::from_iter(download_futures)
        .filter_map(|res| async move { res.ok() })
        .collect()
        .await;

    // Считаем количество успешно загруженных файлов
    let downloaded_count: usize = results.iter().filter(|&count| *count == Ok(1)).count();

    Ok(DownloadedFiles {
        downloaded: downloaded_count,
        total: total_count,
    })
}

async fn create_directory(name: &str) -> Result<PathBuf, String> {
    let dir_path = Path::new("result").join(name);
    if !dir_path.exists() {
        let _ =
            fs::create_dir_all(&dir_path).map_err(|e| format!("Failed to create directory: {}", e));
    }
    Ok(dir_path)
}

async fn get_page(url: &str) -> Result<String, String> {
    reqwest::get(url)
        .await
        .map_err(|e| format!("Failed to fetch page: {}", e))?
        .text()
        .await
        .map_err(|e| format!("Failed to read page text: {}", e))
}

fn calculate_progress(total: usize, downloaded: usize) -> usize {
    ((downloaded as f64 / total as f64) * 100.0).round() as usize
}

fn get_media_urls(page: &str) -> Vec<String> {
    let document = Document::from(page);
    let mut media_urls: Vec<String> = Vec::new();

    for node in document.find(Name("img")).filter_map(|n| n.attr("src")) {
        media_urls.push(node.to_string());
    }
    for node in document.find(Name("video")).filter_map(|n| n.attr("src")) {
        media_urls.push(node.to_string());
    }

    media_urls
}

fn count_media_files(page: &str) -> usize {
    let document = Document::from(page);
    let mut count = 0;

    for _ in document.find(Name("img")).filter_map(|n| n.attr("src")) {
        count += 1;
    }
    for _ in document.find(Name("video")).filter_map(|n| n.attr("src")) {
        count += 1;
    }
    count
}

fn check_url(url: &str) -> Option<Vec<String>> {
    if url.starts_with("http://") || url.starts_with("https://") {
        Regex::new(r"(http[s]?://[^/\s]+/)(.*)")
            .unwrap()
            .captures(url)
            .map(|caps| {
                vec![
                    caps.get(1).map_or("", |m| m.as_str()).to_string(), // Base URL part
                    caps.get(2).map_or("", |m| m.as_str()).to_string(), // Remaining path part
                ]
            })
    } else {
        None
    }
}

fn is_valid_extension(file_name: &str) -> bool {
    config::EXTENSIONS
        .iter()
        .any(|ext| file_name.ends_with(ext))
}

async fn download_file(url: &str, file_path: &Path, link_id: usize) -> Result<(), String> {
    let response = reqwest::get(url)
        .await
        .map_err(|e| format!("Request failed: {}", e))?
        .bytes()
        .await
        .map_err(|e| format!("Failed to read bytes: {}", e))?;
    let message = format!(
        "Link id: {}, file {}, {} bytes",
        link_id,
        file_path.display(),
        response.len()
    );

    info!("{} {}", message, "downloaded",);

    let mut file =
        fs::File::create(file_path).map_err(|e| format!("Failed to create file: {}", e))?;

    file.write_all(&response).map_err(|e| {
        let error_message = format!("Failed to write to file: {}", e);
        error!("Failed to write to file: {}", e);
        error_message
    })?;

    info!("{} {}", message, "writed to file",);

    Ok(())
}
