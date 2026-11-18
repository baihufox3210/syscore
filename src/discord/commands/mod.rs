pub mod docker;
pub mod npm;
pub mod ping;
pub mod service;

use super::{Data, Error};

pub fn all() -> Vec<poise::Command<Data, Error>> {
    vec![ping::ping()]
}
