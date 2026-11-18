pub mod ping;
pub mod task;

use super::{Data, Error};

pub fn all() -> Vec<poise::Command<Data, Error>> {
    vec![ping::ping()]
}
