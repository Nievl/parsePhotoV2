use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Mediafile {
    pub id: usize,
    pub path: String,
    pub name: String,
    pub hash: String,
    pub size: usize,
    #[serde(rename = "dateAdded")]
    pub date_added: String,
}

pub struct CreateDto {
    pub name: String,
    pub path: String,
    pub hash: String,
    pub size: usize,
    pub link_id: usize,
}
