use serde::{Deserialize, Serialize};

pub const DEFAULT_TIMEOUT_SECS: u64 = 120;
pub const MAX_TIMEOUT_SECS: u64 = 3600;

const META_PREFIX: &str = "# syscore:";

/// 一個由腳本定義的 task。
///
/// 腳本格式：
/// ```bash
/// # syscore:name=blog
/// # syscore:description=Blog API Service
/// # syscore:working_dir=/srv/blog
/// # syscore:timeout=600
///
/// start() { ... }
/// stop() { ... }
/// ```
/// 每個頂層 function 都是一個可執行的 action，`_` 開頭的 function 視為內部 helper。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub name: String,
    pub description: Option<String>,
    pub working_dir: Option<String>,
    pub timeout_secs: u64,
    pub actions: Vec<String>,
}

impl Task {
    /// 解析腳本內容。沒有 `syscore:name` 時使用 `fallback_name`（通常是檔名）。
    pub fn from_script(content: &str, fallback_name: Option<&str>) -> Result<Self, String> {
        let mut name = None;
        let mut description = None;
        let mut working_dir = None;
        let mut timeout_secs = DEFAULT_TIMEOUT_SECS;
        let mut actions: Vec<String> = Vec::new();

        for (index, line) in content.lines().enumerate() {
            let line_no = index + 1;

            if let Some(meta) = line.trim_start().strip_prefix(META_PREFIX) {
                let (key, value) = meta
                    .split_once('=')
                    .ok_or(format!("第 {line_no} 行：metadata 格式應為 `# syscore:key=value`"))?;
                let value = value.trim();

                match key.trim() {
                    "name" => name = Some(value.to_string()),
                    "description" => description = Some(value.to_string()),
                    "working_dir" => {
                        if !value.starts_with('/') {
                            return Err(format!("第 {line_no} 行：working_dir 必須是絕對路徑"));
                        }
                        working_dir = Some(value.to_string());
                    }
                    "timeout" => {
                        timeout_secs = value
                            .parse()
                            .ok()
                            .filter(|t| (1..=MAX_TIMEOUT_SECS).contains(t))
                            .ok_or(format!(
                                "第 {line_no} 行：timeout 必須是 1 到 {MAX_TIMEOUT_SECS} 之間的秒數"
                            ))?;
                    }
                    other => return Err(format!("第 {line_no} 行：未知的 metadata `{other}`")),
                }
                continue;
            }

            if let Some(function) = parse_function_name(line)
                && !function.starts_with('_')
                && !actions.iter().any(|a| a == function)
            {
                actions.push(function.to_string());
            }
        }

        let name = name
            .or_else(|| fallback_name.map(str::to_string))
            .ok_or("缺少 `# syscore:name=...`")?;

        if !is_valid_identifier(&name) {
            return Err(format!("名稱 `{name}` 只能包含英數字、`-`、`_`"));
        }

        if actions.is_empty() {
            return Err("腳本裡找不到任何 function（action）".into());
        }

        Ok(Self {
            name,
            description,
            working_dir,
            timeout_secs,
            actions,
        })
    }

    pub fn has_action(&self, action: &str) -> bool {
        self.actions.iter().any(|a| a == action)
    }
}

pub fn is_valid_identifier(s: &str) -> bool {
    !s.is_empty()
        && s
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
}

/// 辨識頂層（沒有縮排）的 function 定義：
/// `name() {`、`name ()`、`function name {`、`function name() {`
fn parse_function_name(line: &str) -> Option<&str> {
    if line.starts_with(char::is_whitespace) {
        return None;
    }

    let (rest, has_keyword) = match line.strip_prefix("function ") {
        Some(rest) => (rest.trim_start(), true),
        None => (line, false),
    };

    let end = rest
        .find(|c: char| !(c.is_ascii_alphanumeric() || c == '-' || c == '_'))
        .unwrap_or(rest.len());
    let name = &rest[..end];
    if name.is_empty() {
        return None;
    }

    let after = rest[end..].trim_start();
    let is_function = after.starts_with("()")
        || (has_keyword && (after.is_empty() || after.starts_with('{')));

    is_function.then_some(name)
}

#[cfg(test)]
mod tests {
    use super::*;

    const BLOG: &str = r#"#!/usr/bin/env bash
# syscore:name=blog
# syscore:description=Blog API Service
# syscore:working_dir=/srv/blog

start() {
    sudo systemctl start baihu-blog-api
}

function stop {
    sudo systemctl stop baihu-blog-api
}

function restart() {
    sudo systemctl restart baihu-blog-api
}

_helper() {
    inner() { :; }
}

deploy() {
    git pull --ff-only
    restart
}
"#;

    #[test]
    fn parses_metadata_and_actions() {
        let task = Task::from_script(BLOG, None).unwrap();
        assert_eq!(task.name, "blog");
        assert_eq!(task.description.as_deref(), Some("Blog API Service"));
        assert_eq!(task.working_dir.as_deref(), Some("/srv/blog"));
        assert_eq!(task.timeout_secs, DEFAULT_TIMEOUT_SECS);
        assert_eq!(task.actions, ["start", "stop", "restart", "deploy"]);
    }

    #[test]
    fn falls_back_to_file_name() {
        let task = Task::from_script("start() { :; }", Some("web")).unwrap();
        assert_eq!(task.name, "web");
    }

    #[test]
    fn rejects_invalid_scripts() {
        assert!(Task::from_script("start() { :; }", None).is_err());
        assert!(Task::from_script("# syscore:name=x\necho hi", None).is_err());
        assert!(Task::from_script("# syscore:name=a b\nstart() { :; }", None).is_err());
        assert!(Task::from_script("# syscore:name=x\n# syscore:workdir=/a\nstart() { :; }", None).is_err());
        assert!(Task::from_script("# syscore:name=x\n# syscore:working_dir=srv\nstart() { :; }", None).is_err());
        assert!(Task::from_script("# syscore:name=x\n# syscore:timeout=0\nstart() { :; }", None).is_err());
    }
}
