mod cli;

use std::process::ExitCode;

use syscore::discord;
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();

    if args.first().map(String::as_str) == Some("check") {
        return if cli::check(&args[1..]) {
            ExitCode::SUCCESS
        } else {
            ExitCode::FAILURE
        };
    }

    match run_bot().await {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("Error: {e}");
            ExitCode::FAILURE
        }
    }
}

async fn run_bot() -> Result<(), discord::Error> {
    dotenvy::dotenv().ok();

    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()))
        .init();

    let token =
        std::env::var("DISCORD_TOKEN").map_err(|_| "找不到環境變數 DISCORD_TOKEN，請檢查 .env")?;

    discord::bot::run(token).await
}
