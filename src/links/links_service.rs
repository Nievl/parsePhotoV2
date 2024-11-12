use super::config;
use crate::{
    mediafiles::{
        dto::CreateDto,
        mediafiles_service::{download_file, get_hash_size_by_path, MediafilesService},
    },
    utils::{error_response, server_error_response, success_response},
};
use axum::{http::StatusCode, response::IntoResponse, Json};
use futures::stream::{FuturesUnordered, StreamExt};
use log::{error, info, warn};
use regex::Regex;
use reqwest;
use select::{
    document::Document,
    predicate::{Name, Or},
};
use std::{
    collections::HashSet,
    fs::{create_dir_all, read_dir},
    path::{Path, PathBuf},
    sync::Arc,
};
use tokio::spawn;

use super::dto::CreateLinkDto;
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
                    warn!("Link with id {} not found", &id);
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
        let total = media_urls.len();

        info!(
            "Media urls count: {} on page {}",
            &media_urls.len(),
            &link.path
        );

        let dir_path = create_directory(&link.name)
            .await
            .map_err(|e| server_error_response(e))?;

        let downloaded: Vec<CreateDto> = download_files_multi(media_urls, &dir_path, link.id)
            .await
            .map_err(|e| {
                error!("Error downloading files: {}", e);
                server_error_response(e)
            })?;

        let downloaded_count = downloaded.len();

        info!("Downloaded files: {}, from {}", downloaded_count, total);

        let existing_records: HashSet<(String, String)> = self
            .mediafiles_service
            .get_all_by_link_id(id)
            .await
            .map_err(|e| server_error_response(format!("Failed to get mediafiles: {}", e)))?
            .iter()
            .map(|record| (record.hash.clone(), record.path.clone()))
            .collect();

        // Identify missing records and add them to the database
        let new_records: Vec<CreateDto> = downloaded
            .into_iter()
            .filter(|file| !existing_records.contains(&(file.hash.clone(), file.path.clone())))
            .collect();

        for new_record in new_records {
            let path = new_record.path.clone();
            match self.mediafiles_service.create_one(new_record).await {
                Ok(_) => info!("Inserted new mediafile record: {}", path),
                Err(e) => error!("Failed to insert mediafile record: {}, error: {}", path, e),
            }
        }

        let progress = calculate_progress(total, downloaded_count);

        let is_downloaded = downloaded_count == total;
        return match self.links_db_service.update_files_number(
            id,
            downloaded_count,
            total,
            is_downloaded,
            progress,
        ) {
            Ok(_) => Ok(success_response(format!(
                "Downloaded {} files",
                downloaded_count
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
        let mediafiles_names = match read_dir(dir_path) {
            Ok(entries) => entries,
            Err(e) => {
                return Err(server_error_response(format!(
                    "Failed to read directory: {}",
                    e
                )))
            }
        };

        let existing_records: HashSet<(String, String)> = self
            .mediafiles_service
            .get_all_by_link_id(id)
            .await
            .map_err(|e| server_error_response(format!("Failed to get mediafiles: {}", e)))?
            .iter()
            .map(|record| (record.hash.clone(), record.path.clone()))
            .collect();

        for mediafile_name in mediafiles_names {
            let mediafile_name = match mediafile_name {
                Ok(entry) => entry,
                Err(e) => {
                    error!("Error reading directory entry: {}", e);
                    continue;
                }
            };
            let file_path = mediafile_name.path();

            let (hash, size) = match get_hash_size_by_path(&file_path).await {
                Ok((hash, size)) => (hash, size),
                Err(op) => {
                    error!("Error calculating hash and size: {}", op);
                    continue;
                }
            };

            let path_str = file_path.to_string_lossy().to_string();

            if existing_records.contains(&(hash.clone(), path_str.clone())) {
                info!("File with path {} already exists, skipping", path_str);
                continue;
            }

            match self
                .mediafiles_service
                .create_one(CreateDto {
                    name: file_path.file_name().unwrap().to_string_lossy().to_string(),
                    path: path_str,
                    hash,
                    size,
                    link_id: link.id,
                })
                .await
            {
                Ok(_) => info!(
                    "Record for {} file created successfully, link id: {}",
                    file_path.display(),
                    &link.id
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
        let mediafiles = read_dir(dir_path).unwrap().count();

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
        let existed_files_count = read_dir(dir_path).unwrap().count();
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
) -> Result<Vec<CreateDto>, String> {
    let download_futures = urls.into_iter().map(|url| {
        let dir_path = dir_path.to_path_buf(); // Клонируем путь для использования в разных потоках

        spawn(async move {
            let file_name = Regex::new(r".+/").unwrap().replace(&url, "").to_string();
            let file_path = dir_path.join(&file_name);

            if file_path.exists() {
                // нашли и обсчитали файл
                match get_hash_size_by_path(&file_path).await {
                    Ok((hash, size)) => {
                        return Ok(CreateDto {
                            name: file_name,
                            path: file_path.to_string_lossy().to_string(),
                            hash,
                            size,
                            link_id,
                        });
                    }
                    Err(e) => {
                        let m = format!(
                            "Error calculating hash and size: {}, path {}",
                            e,
                            file_path.display()
                        );
                        error!("{}", m);
                        return Err(m);
                    }
                };
            }

            if !is_valid_extension(&file_name) {
                let m = format!("{} is not an image", file_name);
                warn!("{}", m);
                return Err(m);
            }

            let download_url = if !url.starts_with("http") {
                format!("{}{}", *config::ROOT_URL, url)
            } else {
                url.clone()
            };

            info!("Downloading {} to {}", &download_url, file_path.display());

            match download_file(&download_url, &file_path, link_id).await {
                Ok(mediafile) => {
                    info!(
                        "Link_id: {}, {} bytes downloaded and saved to {}",
                        link_id,
                        mediafile.size,
                        &file_path.display(),
                    );
                    return Ok(mediafile);
                }
                Err(e) => {
                    let m = format!("Failed to download {}: {}", url, e);
                    error!("{}", m);
                    return Err(m);
                }
            }
        })
    });

    // Запускаем все загрузки параллельно
    let results: Vec<Result<CreateDto, String>> = FuturesUnordered::from_iter(download_futures)
        .filter_map(|res| async move { res.ok() })
        .collect::<Vec<_>>()
        .await;

    // Фильтруем результаты, чтобы исключить `None`, и собираем в `Vec<CreateDto>`
    let downloaded_files: Vec<CreateDto> = results
        .into_iter()
        .filter_map(|dto| if let Ok(dto) = dto { Some(dto) } else { None })
        .collect();

    Ok(downloaded_files)
}

async fn create_directory(name: &str) -> Result<PathBuf, String> {
    let dir_path = Path::new("result").join(name);
    if !dir_path.exists() {
        let _ = create_dir_all(&dir_path).map_err(|e| format!("Failed to create directory: {}", e));
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
    let mut media_urls = Vec::new();
    process_media_urls(page, |url| media_urls.push(url.to_string()));
    media_urls
}

fn count_media_files(page: &str) -> usize {
    let mut count = 0;
    process_media_urls(page, |_| count += 1);
    count
}

fn process_media_urls<F>(page: &str, mut f: F)
where
    F: FnMut(&str),
{
    let document = Document::from(page);

    for node in document
        .find(Or(Name("img"), Name("video")))
        .filter_map(|n| n.attr("src"))
    {
        f(node);
    }
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
