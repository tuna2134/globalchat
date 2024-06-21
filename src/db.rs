use sqlx::SqlitePool;

pub async fn create_globalchat(
    pool: &SqlitePool,
    name: String,
    author_id: i64,
) -> anyhow::Result<()> {
    sqlx::query!(
        r#"
        INSERT INTO globalchat (name, created_by)
        VALUES (?, ?)
        "#,
        name,
        author_id,
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
        INSERT INTO globalchat_channels (id, name)
        VALUES (?, ?)
        "#,
        channel_id,
        name,
    )
    .execute(pool)
    .await?;
    Ok(())
}
