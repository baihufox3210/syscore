use std::{path::Path, time::Duration};

use crate::{
    domain::task::{Task, is_valid_identifier},
    infrastructure::{
        executor::{self, ActionOutput},
        registry::Registry,
        storage,
    },
};

pub type Error = Box<dyn std::error::Error + Send + Sync>;

pub struct Registered {
    pub task: Task,
    pub replaced: bool,
}

/// 解析並註冊腳本。同名 task 已存在時會覆蓋。
pub fn register(
    registry: &mut Registry,
    content: &str,
    fallback_name: Option<&str>,
) -> Result<Registered, Error> {
    // 從 Windows 上傳的檔案是 CRLF，bash 會把 \r 當成指令的一部分
    let content = content.replace("\r\n", "\n");

    let task = Task::from_script(&content, fallback_name)?;

    storage::save_script(&task.name, &content)?;
    let replaced = registry
        .tasks
        .insert(task.name.clone(), task.clone())
        .is_some();
    registry.save(Path::new(storage::REGISTRY_PATH))?;

    Ok(Registered { task, replaced })
}

pub fn remove(registry: &mut Registry, name: &str) -> Result<bool, Error> {
    if registry.tasks.remove(name).is_none() {
        return Ok(false);
    }

    storage::remove_script(name)?;
    registry.save(Path::new(storage::REGISTRY_PATH))?;

    Ok(true)
}

pub fn find_action(registry: &Registry, name: &str, action: &str) -> Result<Task, Error> {
    let task = registry
        .tasks
        .get(name)
        .ok_or_else(|| format!("找不到 task `{name}`"))?;

    if !task.has_action(action) {
        return Err(format!(
            "`{name}` 沒有 action `{action}`，可用的有：{}",
            task.actions.join(", ")
        )
        .into());
    }

    Ok(task.clone())
}

pub async fn run(task: &Task, action: &str, args: &[String]) -> Result<ActionOutput, Error> {
    debug_assert!(is_valid_identifier(action));

    let output = executor::run_action(
        &storage::script_path(&task.name),
        task.working_dir.as_deref(),
        action,
        args,
        Duration::from_secs(task.timeout_secs),
    )
    .await?;

    Ok(output)
}
