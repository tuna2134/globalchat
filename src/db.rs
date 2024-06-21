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

pub async fn add_channel_to_globalchat(
    pool: &SqlitePool,
    name: String,
    channel_id: i64,
) -> anyhow::Result<()> {
    sqlx::query!(
        r#"
        UPDATE globalchat
        SET channels = channels || ?
        WHERE name = ?
        "#,
        channel_id,
        name,
    )
    .execute(pool)
    .await?;
    Ok(())
}
