use std::{env, sync::Arc};
use tokio::task::JoinSet;
use twilight_gateway::{Event, Intents, Shard, ShardId};
use twilight_http::Client as HttpClient;
use twilight_model::http::interaction::{
    InteractionResponse, InteractionResponseData, InteractionResponseType,
};
use vesper::framework::Framework;
use vesper::prelude::*;

#[command]
#[description = "テスト"]
async fn test(ctx: &mut SlashContext<()>) -> anyhow::Result<()> {
    ctx.interaction_client
        .create_response(
            ctx.interaction.id,
            &ctx.interaction.token,
            &InteractionResponse {
                kind: InteractionResponseType::ChannelMessageWithSource,
                data: Some(InteractionResponseData {
                    content: Some("Hello, World!".to_string()),
                    ..Default::default()
                }),
            },
        )
        .await?;
    Ok(())
}

async fn handle_event(event: Event) -> anyhow::Result<()> {
    match event {
        Event::Ready(_r) => {
            tracing::info!("Bot is ready!");
        }
        _ => {}
    }
    Ok(())
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing::info!("Now booting...");
    tracing_subscriber::fmt::init();
    dotenvy::dotenv().ok();

    let (http, mut shard) = {
        let token = env::var("DISCORD_TOKEN")?;
        let http = HttpClient::new(token.clone());
        let intents = Intents::GUILDS | Intents::MESSAGE_CONTENT;
        let shard = Shard::new(ShardId::ONE, token, intents);
        (http, shard)
    };

    let application_id = {
        let app_info = http.current_user_application().await?.model().await?;
        app_info.id
    };

    let framework = {
        let framework = Framework::builder(http, application_id, ())
            .command(test)
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
        let clone = Arc::clone(&framework);
        set.spawn(async move {
            if let Event::InteractionCreate(inter) = event.clone() {
                tokio::spawn(async move {
                    clone.process(inter.clone().0).await;
                });
            };
            tokio::spawn(handle_event(event));
        });
    }
    Ok(())
}
