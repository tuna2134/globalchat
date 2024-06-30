use reqwest::Client;
use sqlx::SqlitePool;
use std::{env, sync::Arc};
use tokio::task::JoinSet;
use twilight_cache_inmemory::{InMemoryCache, ResourceType};
use twilight_gateway::{Event, Intents, Shard, ShardId};
use twilight_http::Client as HttpClient;
use twilight_model::http::{
    attachment::Attachment,
    interaction::{InteractionResponse, InteractionResponseData, InteractionResponseType},
};
use twilight_model::id::Id;
use vesper::framework::Framework;
use vesper::prelude::*;

mod db;

#[command]
#[description = "グローバルチャットを作成します。"]
#[only_guilds]
#[required_permissions(MANAGE_CHANNELS)]
async fn create(
    ctx: &mut SlashContext<Data>,
    #[description = "名前"] name: String,
) -> anyhow::Result<()> {
    let channel_id = ctx.interaction.clone().channel.map(|c| c.id).unwrap();
    db::create_globalchat(
        &ctx.data.pool,
        name.clone(),
        ctx.interaction.author_id().unwrap().get() as i64,
    )
    .await?;
    db::add_channel_to_globalchat(&ctx.data.pool, name, channel_id.get() as i64).await?;
    ctx.interaction_client
        .create_response(
            ctx.interaction.id,
            &ctx.interaction.token,
            &InteractionResponse {
                kind: InteractionResponseType::ChannelMessageWithSource,
                data: Some(InteractionResponseData {
                    content: Some("作成しました".to_string()),
                    ..Default::default()
                }),
            },
        )
        .await?;
    Ok(())
}

#[command]
#[description = "グローバルチャットに参加します。"]
#[only_guilds]
#[required_permissions(MANAGE_CHANNELS)]
async fn join(
    ctx: &mut SlashContext<Data>,
    #[description = "名前"] name: String,
) -> anyhow::Result<()> {
    let channel_id = ctx.interaction.clone().channel.map(|c| c.id).unwrap();
    db::add_channel_to_globalchat(&ctx.data.pool, name, channel_id.get() as i64).await?;
    ctx.interaction_client
        .create_response(
            ctx.interaction.id,
            &ctx.interaction.token,
            &InteractionResponse {
                kind: InteractionResponseType::ChannelMessageWithSource,
                data: Some(InteractionResponseData {
                    content: Some("参加しました。".to_string()),
                    ..Default::default()
                }),
            },
        )
        .await?;
    Ok(())
}

#[command]
#[description = "グローバルチャットから退出します。"]
#[only_guilds]
#[required_permissions(MANAGE_CHANNELS)]
async fn leave(ctx: &mut SlashContext<Data>) -> anyhow::Result<()> {
    let channel_id = ctx.interaction.clone().channel.map(|c| c.id).unwrap();
    db::delete_globalchat_channel(&ctx.data.pool, channel_id.get() as i64).await?;
    ctx.interaction_client
        .create_response(
            ctx.interaction.id,
            &ctx.interaction.token,
            &InteractionResponse {
                kind: InteractionResponseType::ChannelMessageWithSource,
                data: Some(InteractionResponseData {
                    content: Some("退出しました。".to_string()),
                    ..Default::default()
                }),
            },
        )
        .await?;
    Ok(())
}

#[command]
#[description = "グローバルチャットを削除します"]
#[only_guilds]
#[required_permissions(MANAGE_CHANNELS)]
async fn delete(
    ctx: &mut SlashContext<Data>,
    #[description = "名前"] name: String,
) -> anyhow::Result<()> {
    db::delete_globalchat(
        &ctx.data.pool,
        name,
        ctx.interaction.author().unwrap().id.get() as i64,
    )
    .await?;
    ctx.interaction_client
        .create_response(
            ctx.interaction.id,
            &ctx.interaction.token,
            &InteractionResponse {
                kind: InteractionResponseType::ChannelMessageWithSource,
                data: Some(InteractionResponseData {
                    content: Some("削除しました".to_string()),
                    ..Default::default()
                }),
            },
        )
        .await?;
    Ok(())
}

#[after]
async fn after(
    ctx: &mut SlashContext<Data>,
    command_name: &str,
    error: Option<anyhow::Result<()>>,
) {
    if let Some(Err(err)) = error {
        tracing::error!(
            "Command '{}' executed by user '{}' with result '{:?}'",
            command_name,
            ctx.interaction.author_id().unwrap(),
            err
        );
    };
}

async fn handle_event(
    event: Event,
    http: Arc<HttpClient>,
    cache: Arc<InMemoryCache>,
    pool: Arc<SqlitePool>,
) -> anyhow::Result<()> {
    match event {
        Event::Ready(_r) => {
            tracing::info!("Bot is ready!");
        }
        Event::MessageCreate(msg) => {
            if msg.author.bot {
                return Ok(());
            }
            let name =
                db::get_globalchat_name_by_channel_id(&pool, msg.channel_id.get() as i64).await?;
            if let Some(name) = name {
                tracing::debug!("Global chat: {}", name);
                let channels = db::get_globalchat_channels(&pool, name).await?;
                let attachments = {
                    let mut attachments: Vec<Attachment> = Vec::new();
                    let client = Client::new();
                    for (index, attachment) in msg.attachments.iter().enumerate() {
                        let data = client.get(&attachment.url).send().await?.bytes().await?;
                        attachments.push(Attachment::from_bytes(
                            attachment.filename.clone(),
                            data.to_vec(),
                            index as u64,
                        ))
                    }
                    attachments
                };
                for channel in channels {
                    if channel == msg.channel_id.get() as i64 {
                        continue;
                    }
                    let channel_id = Id::new(channel as u64);
                    let bot = cache.current_user().unwrap();
                    let webhooks = http.channel_webhooks(channel_id).await?.model().await?;
                    let webhooks = webhooks
                        .iter()
                        .filter(|webhook| {
                            if let Some(webhook_user) = &webhook.user {
                                webhook_user.id == bot.id
                            } else {
                                false
                            }
                        })
                        .collect::<Vec<_>>();
                    let webhook = if webhooks.is_empty() {
                        drop(webhooks);
                        http.create_webhook(channel_id, "globalchat")?
                            .await?
                            .model()
                            .await?
                    } else {
                        webhooks[0].clone()
                    };
                    let avatar_hash = if let Some(avatar) = msg.author.avatar {
                        avatar.to_string()
                    } else {
                        let result = if msg.author.discriminator == 0 {
                            (msg.author.id.get() >> 22) % 5
                        } else {
                            (msg.author.discriminator % 5).into()
                        };
                        result.to_string()
                    };
                    let avatar_url = format!(
                        "https://cdn.discordapp.com/avatars/{}/{}.png",
                        msg.author.id, avatar_hash
                    );
                    http.execute_webhook(webhook.id, &webhook.token.unwrap())
                        .content(&msg.content)?
                        .avatar_url(&avatar_url)
                        .username(&msg.author.name)?
                        .attachments(&attachments)?
                        .await?;
                }
            }
        }
        _ => {}
    }
    Ok(())
}

struct Data {
    pool: Arc<SqlitePool>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing::info!("Now booting...");
    tracing_subscriber::fmt::init();
    dotenvy::dotenv().ok();

    let (http, mut shard, cache) = {
        let token = env::var("DISCORD_TOKEN")?;
        let http = HttpClient::new(token.clone());
        let intents = Intents::GUILDS | Intents::MESSAGE_CONTENT | Intents::GUILD_MESSAGES;
        let shard = Shard::new(ShardId::ONE, token, intents);
        let cache = InMemoryCache::builder()
            .resource_types(ResourceType::USER_CURRENT)
            .build();
        (Arc::new(http), shard, Arc::new(cache))
    };

    let application_id = {
        let app_info = http.current_user_application().await?.model().await?;
        app_info.id
    };

    let pool = {
        let pool = SqlitePool::connect(&env::var("DATABASE_URL")?).await?;
        sqlx::migrate!().run(&pool).await?;
        Arc::new(pool)
    };

    let framework = {
        let data = Data {
            pool: Arc::clone(&pool),
        };
        let framework = Framework::builder(http.clone(), application_id, data)
            .command(create)
            .command(join)
            .command(leave)
            .command(delete)
            .after(after)
            .build();
        let content = serde_json::to_string_pretty(&framework.twilight_commands())?;
        let path = concat!(env!("CARGO_MANIFEST_DIR"), "/commands.locks.json");
        std::fs::write(path, content)?;
        Arc::new(framework)
    };

    let mut set = JoinSet::new();

    loop {
        let event = shard.next_event().await;
        let event = match event {
            Err(err) => {
                tracing::error!("Error receiving event: {:?}", err);
                if err.is_fatal() {
                    break;
                }
                continue;
            }
            Ok(event) => event,
        };
        cache.update(&event.clone());
        let clone = Arc::clone(&framework);
        let http = Arc::clone(&http);
        let pool = Arc::clone(&pool);
        let cache = Arc::clone(&cache);
        set.spawn(async move {
            if let Event::InteractionCreate(inter) = event.clone() {
                tokio::spawn(async move {
                    clone.process(inter.clone().0).await;
                });
            };
            tokio::spawn(handle_event(
                event,
                Arc::clone(&http),
                Arc::clone(&cache),
                Arc::clone(&pool),
            ));
        });
    }
    Ok(())
}
