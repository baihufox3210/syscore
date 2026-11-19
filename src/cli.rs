use std::{path::Path, process::Command};

use syscore::domain::task::Task;

pub fn check(paths: &[String]) -> bool {
    if paths.is_empty() {
        eprintln!("用法：syscore check <script.sh>...");
        return false;
    }

    let mut all_ok = true;
    for path in paths {
        all_ok &= check_one(Path::new(path));
    }
    all_ok
}

fn check_one(path: &Path) -> bool {
    let display = path.display();

    let content = match std::fs::read_to_string(path) {
        Ok(content) => content.replace("\r\n", "\n"),
        Err(e) => {
            println!("error {display}: {e}");
            return false;
        }
    };

    let fallback = path.file_stem().and_then(|s| s.to_str());
    let task = match Task::from_script(&content, fallback) {
        Ok(task) => task,
        Err(e) => {
            println!("error {display}: {e}");
            return false;
        }
    };

    match Command::new("bash").arg("-n").arg(path).output() {
        Ok(output) if !output.status.success() => {
            println!("error {display}: bash 語法錯誤");
            print!("{}", String::from_utf8_lossy(&output.stderr));
            return false;
        }
        Err(e) => println!("warn  {display}: 無法執行 bash -n：{e}"),
        Ok(_) => {}
    }

    println!("ok    {display} → {}", task.name);
    if let Some(description) = &task.description {
        println!("      description: {description}");
    }
    if let Some(dir) = &task.working_dir {
        let note = if Path::new(dir).is_dir() { "" } else { "（這台機器上不存在）" };
        println!("      working_dir: {dir}{note}");
    }
    println!("      timeout:     {}s", task.timeout_secs);
    println!("      actions:     {}", task.actions.join(" "));

    for (line_no, warning) in lint(&content) {
        println!("warn  {display}:{line_no}: {warning}");
    }

    true
}

fn lint(content: &str) -> Vec<(usize, &'static str)> {
    let mut warnings = Vec::new();

    for (index, line) in content.lines().enumerate() {
        let line_no = index + 1;
        let code = line.split(" #").next().unwrap_or(line);
        let trimmed = code.trim();

        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }

        if code.contains("BASH_SOURCE") {
            warnings.push((line_no, "上傳後腳本存放在 data/scripts/，BASH_SOURCE 不會指向原本的位置，請改用 working_dir"));
        }
        if has_word(code, "sudo") && !code.contains("sudo -n") {
            warnings.push((line_no, "sudo 建議加上 -n：沒有設定免密碼時會直接失敗並顯示清楚的錯誤"));
        }
        if code.contains(" -it ") || code.contains(" -ti ") {
            warnings.push((line_no, "執行時沒有 TTY，-it / -ti 會失敗，請拿掉 -t"));
        }
        if !line.starts_with(char::is_whitespace) && is_top_level_command(trimmed) {
            warnings.push((line_no, "頂層指令在每次執行任何 action 時都會跑，請移到 function 裡"));
        }
    }

    warnings
}

fn has_word(code: &str, word: &str) -> bool {
    code.split(|c: char| !(c.is_ascii_alphanumeric() || c == '_' || c == '-'))
        .any(|token| token == word)
}

fn is_top_level_command(line: &str) -> bool {
    if line.starts_with('}') || line.contains("()") || line.starts_with("function ") {
        return false;
    }

    let first = line.split_whitespace().next().unwrap_or_default();
    if matches!(
        first,
        "set" | "export" | "readonly" | "declare" | "shopt" | "umask" | "then" | "else" | "fi" | "do" | "done" | "esac"
    ) {
        return false;
    }

    // NAME=value
    let is_assignment = first
        .split_once('=')
        .is_some_and(|(name, _)| !name.is_empty() && name.chars().all(|c| c.is_ascii_alphanumeric() || c == '_'));

    !is_assignment
}
