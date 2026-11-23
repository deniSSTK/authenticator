use serde::Deserialize;

#[derive(Deserialize)]
pub(super) struct RegisterRequest {
    pub(super) email: String,
    pub(super) password: String,
}

#[derive(Deserialize)]
pub(super) struct LoginRequest {
    pub(super) email: String,
    pub(super) password: String,
}

#[derive(Deserialize)]
pub(super) struct RefreshRequest {
    pub(super) refresh_token: String,
}
