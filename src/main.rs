use syscore::discord;

#[tokio::main]
async fn main() -> Result<(), discord::Error> {
    dotenvy::dotenv().ok();

    let token =
        std::env::var("DISCORD_TOKEN").map_err(|_| "找不到環境變數 DISCORD_TOKEN，請檢查 .env")?;

    discord::bot::run(token).await
}
