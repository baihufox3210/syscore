use std::{
    io,
    path::Path,
    process::Stdio,
    time::{Duration, Instant},
};

use tokio::{process::Command, time::timeout};

/// 載入使用者腳本並呼叫指定的 function。
/// 所有值都透過環境變數和 argv 傳入，不會被當成 shell 程式碼解析。
const WRAPPER: &str = r#"set -eo pipefail
if [[ -n "${SYSCORE_WORKDIR:-}" ]]; then
    cd -- "$SYSCORE_WORKDIR"
fi
source "$SYSCORE_SCRIPT"
"$SYSCORE_ACTION" "$@"
"#;

pub struct ActionOutput {
    pub exit_code: Option<i32>,
    pub stdout: String,
    pub stderr: String,
    pub elapsed: Duration,
    pub timed_out: bool,
}

impl ActionOutput {
    pub fn success(&self) -> bool {
        !self.timed_out && self.exit_code == Some(0)
    }
}

pub async fn run_action(
    script: &Path,
    working_dir: Option<&str>,
    action: &str,
    args: &[String],
    limit: Duration,
) -> io::Result<ActionOutput> {
    // 腳本可能會 cd 到 working_dir，所以要用絕對路徑
    let script = std::path::absolute(script)?;

    let mut command = Command::new("/bin/bash");
    command
        .arg("-c")
        .arg(WRAPPER)
        .arg("syscore") // 對應腳本裡的 $0
        .args(args)
        .env("SYSCORE_SCRIPT", &script)
        .env("SYSCORE_WORKDIR", working_dir.unwrap_or_default())
        .env("SYSCORE_ACTION", action)
        // stdin 不接任何東西，sudo 需要密碼時會直接失敗，而不是卡住
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        // 獨立的 process group，逾時可以把 npm、docker 等子行程一起終止
        .process_group(0)
        .kill_on_drop(true);

    let started = Instant::now();
    let child = command.spawn()?;
    let pid = child.id();

    match timeout(limit, child.wait_with_output()).await {
        Ok(output) => {
            let output = output?;
            Ok(ActionOutput {
                exit_code: output.status.code(),
                stdout: String::from_utf8_lossy(&output.stdout).into_owned(),
                stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
                elapsed: started.elapsed(),
                timed_out: false,
            })
        }
        Err(_) => {
            if let Some(pid) = pid {
                let _ = Command::new("kill")
                    .arg("-KILL")
                    .arg(format!("-{pid}"))
                    .status()
                    .await;
            }
            Ok(ActionOutput {
                exit_code: None,
                stdout: String::new(),
                stderr: format!("超過 {} 秒，已終止", limit.as_secs()),
                elapsed: started.elapsed(),
                timed_out: true,
            })
        }
    }
}
