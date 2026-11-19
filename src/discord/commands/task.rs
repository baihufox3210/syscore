use std::fmt::Write as _;

use poise::serenity_prelude as serenity;

use crate::{
    application::task_manager,
    discord::{
        Context, Error, auth,
        response::{MAX_BLOCK_CHARS, code_block},
    },
    infrastructure::storage,
};

const MAX_SCRIPT_BYTES: u32 = 64 * 1024;
const MAX_CHOICES: usize = 25;

#[poise::command(slash_command, ephemeral)]
pub async fn task_add(
    ctx: Context<'_>,
    #[description = "Bash 腳本（.sh），用 # syscore:name=... 定義名稱，每個 function 是一個 action"]
    file: serenity::Attachment,
) -> Result<(), Error> {
    auth::authorize(ctx).await?;

    if file.size > MAX_SCRIPT_BYTES {
        return Err(format!("腳本太大（上限 {} KB）", MAX_SCRIPT_BYTES / 1024).into());
    }

    let bytes = file.download().await?;
    let content = String::from_utf8(bytes).map_err(|_| "腳本必須是 UTF-8 文字檔")?;

    let fallback_name = file
        .filename
        .strip_suffix(".sh")
        .unwrap_or(&file.filename)
        .to_string();

    let registered = {
        let mut registry = ctx.data().registry.lock().await;
        task_manager::register(&mut registry, &content, Some(&fallback_name))?
    };

    let task = &registered.task;
    let verb = if registered.replaced {
        "已更新"
    } else {
        "已新增"
    };

    let mut reply = format!("{verb} task `{}`\n", task.name);
    if let Some(description) = &task.description {
        writeln!(reply, "說明：{description}")?;
    }
    if let Some(dir) = &task.working_dir {
        writeln!(reply, "工作目錄：`{dir}`")?;
    }
    writeln!(reply, "逾時：{} 秒", task.timeout_secs)?;
    writeln!(reply, "Actions：{}", format_actions(&task.actions))?;

    ctx.say(reply).await?;
    Ok(())
}

#[poise::command(slash_command)]
pub async fn task_list(ctx: Context<'_>) -> Result<(), Error> {
    auth::authorize(ctx).await?;

    let registry = ctx.data().registry.lock().await;

    if registry.tasks.is_empty() {
        ctx.say("還沒有任何 task，用 `/task_add` 上傳腳本。")
            .await?;
        return Ok(());
    }

    let mut tasks: Vec<_> = registry.tasks.values().collect();
    tasks.sort_by(|a, b| a.name.cmp(&b.name));

    let mut reply = String::new();
    for task in tasks {
        write!(reply, "**{}**", task.name)?;
        if let Some(description) = &task.description {
            write!(reply, " — {description}")?;
        }
        writeln!(reply, "\n　{}", format_actions(&task.actions))?;
    }

    ctx.say(reply).await?;
    Ok(())
}

#[poise::command(slash_command, ephemeral)]
pub async fn task_info(
    ctx: Context<'_>,
    #[autocomplete = "autocomplete_task"] name: String,
) -> Result<(), Error> {
    auth::authorize(ctx).await?;

    let task = {
        let registry = ctx.data().registry.lock().await;
        registry
            .tasks
            .get(&name)
            .cloned()
            .ok_or_else(|| format!("找不到 task `{name}`"))?
    };

    let script = storage::read_script(&task.name)?;

    let mut reply = format!("**{}**\n", task.name);
    if let Some(description) = &task.description {
        writeln!(reply, "說明：{description}")?;
    }
    writeln!(
        reply,
        "工作目錄：`{}`",
        task.working_dir.as_deref().unwrap_or("（bot 目前目錄）")
    )?;
    writeln!(reply, "逾時：{} 秒", task.timeout_secs)?;
    writeln!(reply, "Actions：{}", format_actions(&task.actions))?;
    reply.push_str(&code_block(
        &script,
        MAX_BLOCK_CHARS - reply.chars().count(),
    ));

    ctx.say(reply).await?;
    Ok(())
}

#[poise::command(slash_command, ephemeral)]
pub async fn task_remove(
    ctx: Context<'_>,
    #[autocomplete = "autocomplete_task"] name: String,
) -> Result<(), Error> {
    auth::authorize(ctx).await?;

    let removed = {
        let mut registry = ctx.data().registry.lock().await;
        task_manager::remove(&mut registry, &name)?
    };

    let reply = if removed {
        format!("已刪除 task `{name}`")
    } else {
        format!("找不到 task `{name}`")
    };

    ctx.say(reply).await?;
    Ok(())
}

#[poise::command(slash_command)]
pub async fn task_run(
    ctx: Context<'_>,
    #[autocomplete = "autocomplete_task"] name: String,
    #[autocomplete = "autocomplete_action"] action: String,
    #[description = "傳給 function 的參數，以空白分隔"] args: Option<String>,
) -> Result<(), Error> {
    auth::authorize(ctx).await?;

    let task = {
        let registry = ctx.data().registry.lock().await;
        task_manager::find_action(&registry, &name, &action)?
    };

    let args: Vec<String> = args
        .as_deref()
        .unwrap_or_default()
        .split_whitespace()
        .map(str::to_string)
        .collect();

    ctx.defer().await?;

    let output = task_manager::run(&task, &action, &args).await?;

    let icon = if output.success() { "✅" } else { "❌" };
    let status = if output.timed_out {
        "逾時".to_string()
    } else {
        match output.exit_code {
            Some(code) => format!("exit {code}"),
            None => "被 signal 終止".to_string(),
        }
    };

    let mut log = output.stdout;
    if !output.stderr.trim().is_empty() {
        if !log.trim().is_empty() {
            log.push_str("\n--- stderr ---\n");
        }
        log.push_str(&output.stderr);
    }
    if log.trim().is_empty() {
        log = "（沒有輸出）".into();
    }

    let reply = format!(
        "{icon} `{name}` → `{action}`（{status}，{:.1} 秒）\n{}",
        output.elapsed.as_secs_f64(),
        code_block(&log, MAX_BLOCK_CHARS)
    );

    ctx.say(reply).await?;
    Ok(())
}

fn format_actions(actions: &[String]) -> String {
    actions
        .iter()
        .map(|a| format!("`{a}`"))
        .collect::<Vec<_>>()
        .join(" ")
}

async fn autocomplete_task(ctx: Context<'_>, partial: &str) -> Vec<String> {
    let registry = ctx.data().registry.lock().await;

    let mut names: Vec<String> = registry
        .tasks
        .keys()
        .filter(|name| name.starts_with(partial))
        .cloned()
        .collect();
    names.sort();
    names.truncate(MAX_CHOICES);
    names
}

async fn autocomplete_action(ctx: Context<'_>, partial: &str) -> Vec<String> {
    let poise::Context::Application(app) = ctx else {
        return Vec::new();
    };

    let name = app.interaction.data.options.iter().find_map(|option| {
        match (&*option.name, &option.value) {
            ("name", serenity::CommandDataOptionValue::String(value)) => Some(value.clone()),
            _ => None,
        }
    });

    let Some(name) = name else {
        return Vec::new();
    };

    let registry = ctx.data().registry.lock().await;

    registry
        .tasks
        .get(&name)
        .map(|task| {
            task.actions
                .iter()
                .filter(|action| action.starts_with(partial))
                .take(MAX_CHOICES)
                .cloned()
                .collect()
        })
        .unwrap_or_default()
}
