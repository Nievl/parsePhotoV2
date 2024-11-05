use axum::{
    extract::{Query, State},
    response::IntoResponse,
    Json,
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
        Json(create_link_dto): Json<CreateLinkDto>,
    ) -> impl IntoResponse {
        return service.create_one(create_link_dto).await;
    }

    pub async fn get_all(
        State(service): State<Arc<LinksService>>,
        Query(query): Query<BooleanQuery>,
    ) -> impl IntoResponse {
        return service.get_all(query.is_reachable.unwrap_or(false)).await;
    }

    pub async fn remove(
        State(service): State<Arc<LinksService>>,
        Query(query): Query<IdDto>,
    ) -> impl IntoResponse {
        return service.remove(query.id).await;
    }

    pub async fn download_files(
        State(service): State<Arc<LinksService>>,
        Query(query): Query<IdDto>,
    ) -> impl IntoResponse {
        return service.download(query.id).await;
    }

    pub async fn check_downloaded(
        State(service): State<Arc<LinksService>>,
        Query(query): Query<IdDto>,
    ) -> impl IntoResponse {
        return service.check_downloaded(query.id).await;
    }

    pub async fn tag_unreachable(
        State(service): State<Arc<LinksService>>,
        Query(query): Query<TagUnreachableParams>,
    ) -> impl IntoResponse {
        return service
            .tag_unreachable(query.id, query.is_reachable.unwrap_or(false))
            .await;
    }
}

pub fn links_routes() -> axum::Router {
    axum::Router::new()
        .route("/links", axum::routing::post(LinksController::create))
        .route("/links", axum::routing::get(LinksController::get_all))
        .route("/links", axum::routing::delete(LinksController::remove))
        .route(
            "/links/download",
            axum::routing::get(LinksController::download_files),
        )
        .route(
            "/links/check_downloaded",
            axum::routing::get(LinksController::check_downloaded),
        )
        .route(
            "/links/tag_unreachable",
            axum::routing::get(LinksController::tag_unreachable),
        )
        .with_state(Arc::new(LinksService::new()))
}
