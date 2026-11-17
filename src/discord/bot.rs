use poise::serenity_prelude as serenity;

use super::{Data, Error, commands};

pub async fn run(token: String) -> Result<(), Error> {
    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: vec![commands::ping::ping()],
            ..Default::default()
        })
        .setup(|ctx, ready, framework| {
            Box::pin(async move {
                println!("Bot 已登入: {}", ready.user.name);

                poise::builtins::register_globally(ctx, &framework.options().commands).await?;

                println!("Slash Commands 註冊完成");

                Ok(Data {})
            })
        })
        .build();

    let intents = serenity::GatewayIntents::non_privileged();

    let mut client = serenity::ClientBuilder::new(token, intents)
        .framework(framework)
        .await?;

    client.start().await?;
    Ok(())
}
