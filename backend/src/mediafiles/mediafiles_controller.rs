use std::sync::Arc;

use axum::{
    extract::{Query, State},
    response::IntoResponse,
    routing,
};

use crate::links::dto::IdDto;

use super::mediafiles_service::MediafilesService;

pub struct MediafilesController {}

impl MediafilesController {
    pub async fn remove(
        State(service): State<Arc<MediafilesService>>,
        Query(query): Query<IdDto>,
    ) -> impl IntoResponse {
        service.remove(query.id).await
    }
}

pub fn mediafiles_routes() -> axum::Router {
    axum::Router::new()
        .route("/mediafiles", routing::delete(MediafilesController::remove))
        .with_state(Arc::new(MediafilesService::new()))
}
