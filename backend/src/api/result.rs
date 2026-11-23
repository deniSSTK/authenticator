use crate::api::ApiError;
use axum::Json;
use axum::http::StatusCode;

pub(crate) type ApiResult<T> = Result<(StatusCode, Json<T>), ApiError>;

pub(crate) fn created<T>(body: T) -> ApiResult<T> {
    Ok((StatusCode::CREATED, Json(body)))
}

pub(crate) fn ok<T>(body: T) -> ApiResult<T> {
    Ok((StatusCode::OK, Json(body)))
}
