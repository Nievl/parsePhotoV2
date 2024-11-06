use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Link {
    pub id: usize,
    pub path: String,
    pub name: String,
    #[serde(rename = "isDownloaded")]
    pub is_downloaded: bool,
    #[serde(rename = "isReachable")]
    pub is_reachable: bool,
    pub progress: usize,
    #[serde(rename = "downloadedMediafiles")]
    pub downloaded_mediafiles: usize,
    pub mediafiles: usize,
    #[serde(rename = "dateUpdate")]
    pub date_update: Option<String>,
    #[serde(rename = "dateCreate")]
    pub date_create: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct IResult {
    pub success: bool,
    pub message: String,
}

#[derive(Deserialize)]
pub struct CreateLinkDto {
    pub path: String,
}

#[derive(Deserialize)]
pub struct BooleanQuery {
    #[serde(rename = "isReachable")]
    pub is_reachable: Option<bool>,
}

#[derive(Deserialize)]
pub struct TagUnreachableParams {
    pub id: usize,
    pub is_reachable: Option<bool>,
}

#[derive(Deserialize)]
pub struct IdDto {
    pub id: usize,
}

pub struct DownloadedFiles {
    pub downloaded: usize,
    pub total: usize,
}
