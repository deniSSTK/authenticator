use serde::Serialize;
use uuid::Uuid;

#[derive(Serialize)]
pub(super) struct RegisterResponse {
    pub(super) data: AuthResponse,
}

#[derive(Serialize)]
pub(super) struct LoginResponse {
    pub(super) data: AuthResponse,
}

#[derive(Serialize)]
pub(super) struct RefreshResponse {
    pub(super) data: AuthResponse,
}

#[derive(Serialize)]
pub(super) struct MeResponse {
    pub(super) data: AuthResponse,
}

#[derive(Serialize)]
pub(super) struct AuthResponse {
    pub(super) tokens: Tokens,
    pub(super) user: AuthUser,
}

#[derive(Serialize)]
pub(super) struct Tokens {
    pub(super) access_token: String,
    pub(super) refresh_token: String,
    pub(super) refresh_token_expires_in: i64,
}

#[derive(Serialize)]
pub(super) struct AuthUser {
    pub(super) id: Uuid,
}
