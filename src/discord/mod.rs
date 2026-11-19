pub mod auth;
pub mod bot;
pub mod commands;
pub mod modal;
pub mod response;

use tokio::sync::Mutex;

use crate::infrastructure::registry::Registry;

pub struct Data {
    pub registry: Mutex<Registry>,
}

pub type Error = Box<dyn std::error::Error + Send + Sync>;
pub type Context<'a> = poise::Context<'a, Data, Error>;
