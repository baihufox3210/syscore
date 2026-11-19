use std::path::Path;

use poise::serenity_prelude as serenity;
use tokio::sync::Mutex;
use tracing::{info, warn};

use super::{Data, Error, commands};
use crate::infrastructure::{registry::Registry, storage};

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
        on_error: |error| Box::pin(on_error(error)),
        ..Default::default()
    }
}

/// 指令錯誤只回覆給執行的人，避免在公開頻道洗版
async fn on_error(error: poise::FrameworkError<'_, Data, Error>) {
    match error {
        poise::FrameworkError::Command { error, ctx, .. } => {
            warn!("/{} 執行失敗：{error}", ctx.command().name);

            let reply = poise::CreateReply::default()
                .content(format!("❌ {error}"))
                .ephemeral(true);

            if let Err(e) = ctx.send(reply).await {
                warn!("無法回覆錯誤訊息：{e}");
            }
        }
        other => {
            if let Err(e) = poise::builtins::on_error(other).await {
                warn!("處理錯誤時失敗：{e}");
            }
        }
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

    let registry = Registry::load(Path::new(storage::REGISTRY_PATH))?;
    info!("已載入 {} 個 task", registry.tasks.len());

    Ok(Data {
        registry: Mutex::new(registry),
    })
}
