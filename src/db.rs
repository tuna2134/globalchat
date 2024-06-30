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

pub async fn get_globalchat_name_by_channel_id(
    pool: &SqlitePool,
    channel_id: i64,
) -> anyhow::Result<Option<String>> {
    let name = sqlx::query!(
        r#"
        SELECT name FROM globalchat_channels
        WHERE id = ?
        "#,
        channel_id,
    )
    .fetch_optional(pool)
    .await?
    .map(|r| r.name);
    Ok(name)
}

pub async fn get_globalchat_channels(pool: &SqlitePool, name: String) -> anyhow::Result<Vec<i64>> {
    let channels = sqlx::query!(
        r#"
        SELECT id FROM globalchat_channels
        WHERE name = ?
        "#,
        name,
    )
    .fetch_all(pool)
    .await?;
    Ok(channels.into_iter().map(|r| r.id).collect())
}

pub async fn delete_globalchat_channel(pool: &SqlitePool, channel_id: i64) -> anyhow::Result<()> {
    sqlx::query!(
        r#"
        DELETE FROM globalchat_channels
        WHERE id = ?
        "#,
        channel_id,
    )
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn delete_globalchat(
    pool: &SqlitePool,
    name: String,
    owner_id: i64,
) -> anyhow::Result<bool> {
    let result = sqlx::query!(
        r#"
        DELETE FROM globalchat
        WHERE name = ? AND created_by = ?
        "#,
        name,
        owner_id,
    )
    .execute(pool)
    .await?;
    Ok(result.rows_affected() > 0)
}
