use axum::{
    extract::{Query, State},
    response::IntoResponse,
    routing::{delete, get, post},
    Json, Router,
};

use std::sync::Arc;

use super::{
    dto::{BooleanQuery, CreateLinkDto, IdDto, TagUnreachableParams},
    links_service::LinksService,
};

#[derive(Clone)]
pub struct LinksController {}

impl LinksController {
    pub async fn create(
        State(service): State<Arc<LinksService>>,
        Json(create_dto): Json<CreateLinkDto>,
    ) -> impl IntoResponse {
        service.create_one(create_dto).await
    }

    pub async fn get_all(
        State(service): State<Arc<LinksService>>,
        Query(query): Query<BooleanQuery>,
    ) -> impl IntoResponse {
        service.get_all(query.is_reachable.unwrap_or(false)).await
    }

    pub async fn remove(
        State(service): State<Arc<LinksService>>,
        Query(query): Query<IdDto>,
    ) -> impl IntoResponse {
        service.remove(query.id).await
    }

    pub async fn download_files(
        State(service): State<Arc<LinksService>>,
        Query(query): Query<IdDto>,
    ) -> impl IntoResponse {
        service.download(query.id).await
    }

    pub async fn check_downloaded(
        State(service): State<Arc<LinksService>>,
        Query(query): Query<IdDto>,
    ) -> impl IntoResponse {
        service.check_downloaded(query.id).await
    }

    pub async fn tag_unreachable(
        State(service): State<Arc<LinksService>>,
        Query(query): Query<TagUnreachableParams>,
    ) -> impl IntoResponse {
        service
            .tag_unreachable(query.id, query.is_reachable.unwrap_or(false))
            .await
    }

    pub async fn scan_files_for_link(
        State(service): State<Arc<LinksService>>,
        Query(query): Query<IdDto>,
    ) -> impl IntoResponse {
        service.scan_files_for_link(query.id).await
    }

    pub async fn scan_files(State(service): State<Arc<LinksService>>) -> impl IntoResponse {
        service.scan_files().await
    }
}

pub fn links_routes() -> Router {
    Router::new()
        .route("/links", post(LinksController::create))
        .route("/links", get(LinksController::get_all))
        .route("/links", delete(LinksController::remove))
        .route("/links/download", get(LinksController::download_files))
        .route(
            "/links/check_downloaded",
            get(LinksController::check_downloaded),
        )
        .route(
            "/links/tag_unreachable",
            get(LinksController::tag_unreachable),
        )
        .route(
            "/links/scan_files_for_link",
            get(LinksController::scan_files_for_link),
        )
        .route("/links/scan_files", get(LinksController::scan_files))
        .with_state(Arc::new(LinksService::new()))
}
