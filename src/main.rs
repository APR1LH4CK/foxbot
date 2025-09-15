mod cmd;
mod config;
mod util;

use std::{collections::HashMap, path::Path, sync::Arc};

use config::Config;
use poise::serenity_prelude as serenity;
use util::logger;

type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, Data, Error>;

#[derive(Clone)]
pub struct Data {
    pub config: Arc<Config>,
    pub fox_images: Arc<Vec<String>>,
    pub fox_image_data: Arc<HashMap<String, Vec<u8>>>,
    pub facts: Arc<Vec<serde_json::Value>>,
}

async fn on_error(error: poise::FrameworkError<'_, Data, Error>) {
    match error {
        poise::FrameworkError::Setup { error, .. } => {
            panic!("Failed to start bot: {:?}", error)
        }
        poise::FrameworkError::Command { error, ctx, .. } => {
            let error_msg = format!("Error: {}", error);
            let embed = util::embed::create_error_embed("Command Error", &error_msg);

            if let Err(e) = ctx
                .send(poise::CreateReply::default().embed(embed).ephemeral(true))
                .await
            {
                tracing::error!("Error sending error response: {:?}", e);
            }

            tracing::error!("Error in command `{}`: {:?}", ctx.command().name, error);
        }
        error => {
            if let Err(e) = poise::builtins::on_error(error).await {
                tracing::error!("Error while handling error: {}", e);
            }
        }
    }
}

async fn on_ready(
    ctx: &serenity::Context,
    ready: &serenity::Ready,
    _framework: &poise::Framework<Data, Error>,
    _data: &Data,
) -> Result<(), Error> {
    tracing::info!("{} is connected!", ready.user.name);

    ctx.set_presence(
        Some(serenity::ActivityData::watching("videos of foxes")),
        serenity::OnlineStatus::Idle,
    );

    Ok(())
}

async fn load_fox_images() -> Result<(Vec<String>, HashMap<String, Vec<u8>>), Error> {
    let images_dir = Path::new("images/foxes");

    if !images_dir.exists() {
        tracing::warn!("Fox images directory not found!");
        return Ok((Vec::new(), HashMap::new()));
    }

    let mut entries = tokio::fs::read_dir(images_dir).await?;
    let mut fox_images = Vec::new();
    let mut fox_image_data = HashMap::new();

    while let Some(entry) = entries.next_entry().await? {
        let path = entry.path();
        if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("jpg") {
            if let Some(file_name) = path.file_name().and_then(|s| s.to_str()) {
                match tokio::fs::read(&path).await {
                    Ok(data) => {
                        fox_images.push(file_name.to_string());
                        fox_image_data.insert(file_name.to_string(), data);
                        tracing::debug!("Loaded image: {}", file_name);
                    }
                    Err(e) => {
                        tracing::warn!("Failed to load image {}: {}", file_name, e);
                    }
                }
            }
        }
    }

    tracing::info!("Loaded {} fox images into memory", fox_images.len());
    Ok((fox_images, fox_image_data))
}

async fn load_facts() -> Result<Vec<serde_json::Value>, Error> {
    tracing::info!("Loading fox facts...");
    match tokio::fs::read_to_string("facts.json").await {
        Ok(data) => match serde_json::from_str::<Vec<serde_json::Value>>(&data) {
            Ok(facts) => {
                tracing::info!("Loaded {} fox facts into memory", facts.len());
                Ok(facts)
            }
            Err(e) => {
                tracing::warn!("Failed to parse facts.json: {}", e);
                Ok(Vec::new())
            }
        },
        Err(e) => {
            tracing::warn!("Failed to load facts.json: {}", e);
            Ok(Vec::new())
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    logger::init();

    let config = Arc::new(Config::load()?);

    if config.discord.token.is_empty() {
        panic!("Discord token not set in configuration");
    }

    tracing::info!("Starting foxbot...");

    let (fox_images_list, fox_image_data_map) = load_fox_images().await?;
    let fox_images = Arc::new(fox_images_list);
    let fox_image_data = Arc::new(fox_image_data_map);
    let facts = Arc::new(load_facts().await?);

    let data = Data {
        config: config.clone(),
        fox_images,
        fox_image_data,
        facts,
    };

    let intents =
        serenity::GatewayIntents::non_privileged() | serenity::GatewayIntents::MESSAGE_CONTENT;

    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: vec![cmd::fun::fox::fox(), cmd::fun::fact::fact()],
            prefix_options: poise::PrefixFrameworkOptions {
                prefix: None,
                ..Default::default()
            },
            on_error: |error| Box::pin(on_error(error)),
            pre_command: |ctx| {
                Box::pin(async move {
                    tracing::info!("Executing command {}...", ctx.command().qualified_name);
                })
            },
            post_command: |ctx| {
                Box::pin(async move {
                    tracing::info!("Executed command {}!", ctx.command().qualified_name);
                })
            },
            command_check: Some(|ctx| {
                Box::pin(async move {
                    if ctx.author().bot {
                        return Ok(false);
                    }
                    Ok(true)
                })
            }),
            ..Default::default()
        })
        .setup(move |ctx, ready, framework| {
            Box::pin(async move {
                let commands =
                    poise::builtins::create_application_commands(&framework.options().commands);

                let user_installable_commands: Vec<_> = commands
                    .into_iter()
                    .map(|command| {
                        command
                            .integration_types(vec![
                                serenity::InstallationContext::Guild,
                                serenity::InstallationContext::User,
                            ])
                            .contexts(vec![
                                serenity::InteractionContext::Guild,
                                serenity::InteractionContext::BotDm,
                                serenity::InteractionContext::PrivateChannel,
                            ])
                    })
                    .collect();

                ctx.http
                    .create_global_commands(&user_installable_commands)
                    .await?;

                tracing::info!(
                    "Registered {} user-installable commands",
                    user_installable_commands.len()
                );

                on_ready(ctx, ready, framework, &data).await?;
                Ok(data)
            })
        })
        .build();

    let client = serenity::ClientBuilder::new(&config.discord.token, intents)
        .framework(framework)
        .await;

    client?.start().await?;
    Ok(())
}
