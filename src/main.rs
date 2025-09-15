mod cmd;
mod config;
mod util;

use std::sync::Arc;

use config::Config;
use poise::serenity_prelude as serenity;
use util::logger;

type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, Data, Error>;

#[derive(Clone)]
pub struct Data {
    pub config: Arc<Config>,
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
    _ctx: &serenity::Context,
    ready: &serenity::Ready,
    _framework: &poise::Framework<Data, Error>,
    _data: &Data,
) -> Result<(), Error> {
    tracing::info!("{} is connected!", ready.user.name);
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    logger::init();

    let config = Arc::new(Config::load()?);

    if config.discord.token.is_empty() {
        panic!("Discord token not set in configuration");
    }

    tracing::info!("Starting foxbot...");

    let data = Data {
        config: config.clone(),
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
