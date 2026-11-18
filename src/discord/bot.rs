use poise::serenity_prelude as serenity;
use tracing::info;

use super::{Data, Error, commands};

type Framework = poise::Framework<Data, Error>;

pub async fn run(token: String) -> Result<(), Error> {
    let framework = Framework::builder()
        .options(framework_options())
        .setup(|ctx, ready, framework| Box::pin(on_ready(ctx, ready, framework)))
        .build();

    let intents = serenity::GatewayIntents::non_privileged();

    let mut client = serenity::ClientBuilder::new(token, intents)
        .framework(framework)
        .await?;

    client.start().await?;
    Ok(())
}

fn framework_options() -> poise::FrameworkOptions<Data, Error> {
    poise::FrameworkOptions {
        commands: commands::all(),
        ..Default::default()
    }
}

async fn on_ready(
    ctx: &serenity::Context,
    ready: &serenity::Ready,
    framework: &Framework,
) -> Result<Data, Error> {
    info!("Bot 已登入: {}", ready.user.name);

    poise::builtins::register_globally(ctx, &framework.options().commands).await?;
    info!("Slash commands 註冊完成");

    Ok(Data {})
}
