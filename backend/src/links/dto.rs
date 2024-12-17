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
    #[serde(rename = "duplicateId")]
    pub duplicate_id: Option<usize>,
    #[serde(rename = "duplicatePath")]
    pub duplicate_path: Option<String>,
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
    #[serde(rename = "showDuplicate")]
    pub show_duplicate: Option<bool>,
}

#[derive(Deserialize)]
pub struct TagUnreachableParams {
    pub id: usize,
    #[serde(rename = "isReachable")]
    pub is_reachable: Option<bool>,
}

#[derive(Deserialize)]
pub struct IdDto {
    pub id: usize,
}

#[derive(Deserialize)]
pub struct IdDublicateDto {
    #[serde(rename = "linkId")]
    pub link_id: usize,
    #[serde(rename = "duplicateId")]
    pub duplicate_id: usize,
}
