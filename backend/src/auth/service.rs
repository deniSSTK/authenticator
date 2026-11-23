use argon2::password_hash;
use uuid::Uuid;

pub(super) struct RegisterUser {
    pub(super) id: Uuid,
    pub(super) email: String,
}

#[derive(Debug)]
pub(super) enum RegisteredError {
    InvalidInput,
    HashTask(tokio::task::JoinError),
    Hash(password_hash::Error),
    Database(sqlx::Error),
}