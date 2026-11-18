use syscore::discord;
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> Result<(), discord::Error> {
    dotenvy::dotenv().ok();

    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()))
        .init();

    let token =
        std::env::var("DISCORD_TOKEN").map_err(|_| "找不到環境變數 DISCORD_TOKEN，請檢查 .env")?;

    discord::bot::run(token).await
}
