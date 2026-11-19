pub mod ping;
pub mod task;

use super::{Data, Error};

pub fn all() -> Vec<poise::Command<Data, Error>> {
    vec![
        ping::ping(),
        task::task_add(),
        task::task_list(),
        task::task_info(),
        task::task_remove(),
        task::task_run(),
    ]
}
