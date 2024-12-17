use crate::links::dto::IResult;
use axum::Json;
use chrono::Utc;
use reqwest::StatusCode;

pub fn get_now_time() -> String {
    Utc::now().format("%Y-%m-%d %H:%M:%S").to_string()
}

pub fn server_error_response(message: String) -> (StatusCode, Json<IResult>) {
    return (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(IResult {
            success: false,
            message,
        }),
    );
}

pub fn error_response(message: String, status: StatusCode) -> (StatusCode, Json<IResult>) {
    return (
        status,
        Json(IResult {
            success: false,
            message,
        }),
    );
}

pub fn success_response(message: String) -> (StatusCode, Json<IResult>) {
    return (
        StatusCode::OK,
        Json(IResult {
            success: true,
            message,
        }),
    );
}
