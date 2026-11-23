use axum::Json;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde::Serialize;

#[derive(Serialize)]
struct ErrorResponse {
    code: &'static str,
}

pub(crate) enum ApiError {
    InvalidInput,
    NotImplemented,
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, code) = match self {
            Self::InvalidInput => (StatusCode::UNPROCESSABLE_ENTITY, "invalid_input"),
            Self::NotImplemented => (StatusCode::NOT_IMPLEMENTED, "not_implemented"),
        };

        (status, Json(ErrorResponse { code })).into_response()
    }
}
