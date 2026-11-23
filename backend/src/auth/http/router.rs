use crate::api::{ApiError, ApiResult};
use crate::state::AppState;
use axum::{
    Json, Router,
    routing::{get, post},
};

use super::requests::{LoginRequest, RefreshRequest, RegisterRequest};
use super::responses::{LoginResponse, MeResponse, RefreshResponse, RegisterResponse};

pub(crate) fn router() -> Router<AppState> {
    Router::new()
        .route("/register", post(register))
        .route("/login", post(login))
        .route("/refresh", post(refresh))
        .route("/me", get(me))
}

async fn register(Json(body): Json<RegisterRequest>) -> ApiResult<RegisterResponse> {
    if body.email.trim().is_empty() || body.password.is_empty() {
        return Err(ApiError::InvalidInput);
    }

    Err(ApiError::NotImplemented)
}

async fn login(Json(body): Json<LoginRequest>) -> ApiResult<LoginResponse> {
    if body.email.trim().is_empty() || body.password.trim().is_empty() {
        return Err(ApiError::InvalidInput);
    }

    Err(ApiError::NotImplemented)
}

async fn refresh(Json(body): Json<RefreshRequest>) -> ApiResult<RefreshResponse> {
    if body.refresh_token.trim().is_empty() {
        return Err(ApiError::InvalidInput);
    }

    Err(ApiError::NotImplemented)
}

async fn me() -> ApiResult<MeResponse> {
    Err(ApiError::NotImplemented)
}
