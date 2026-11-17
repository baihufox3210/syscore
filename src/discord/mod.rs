pub mod auth;
pub mod bot;
pub mod commands;
pub mod response;

pub struct Data {}

pub type Error = Box<dyn std::error::Error + Send + Sync>;
pub type Context<'a> = poise::Context<'a, Data, Error>;
