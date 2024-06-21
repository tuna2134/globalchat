use sqlx::SqlitePool;

pub async fn create_globalchat(
    pool: &SqlitePool,
    name: String,
    channel_id: i64,
) -> anyhow::Result<()> {
    sqlx::query!(
        r#"
        INSERT INTO globalchat (name, channels)
        VALUES (?, ?)
        "#,
        name,
        channel_id,
    )
    .execute(pool)
    .await?;
    Ok(())
}
