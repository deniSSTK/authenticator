use sqlx::PgPool;
use uuid::Uuid;

pub(super) async fn insert_user(
    db: &PgPool,
    id: Uuid,
    email: &str,
    password_hash: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT INTO users (id, email, password_hash) VALUES ($1, $2, $3)",
    )
        .bind(id)
        .bind(email)
        .bind(password_hash)
        .execute(db)
        .await?;

    Ok(())
}