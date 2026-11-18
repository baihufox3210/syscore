use tokio::process::Command;

use crate::domain::action::ServiceAction;

pub async fn execute(unit: &str, action: ServiceAction) -> Result<String, std::io::Error> {
    let operation = match action {
        ServiceAction::Start => "start",
        ServiceAction::Stop => "stop",
        ServiceAction::Restart => "restart",
        ServiceAction::Status => "is-active",
    };

    let output = Command::new("systemctl")
        .arg(operation)
        .arg(unit)
        .output()
        .await?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    Ok(format!("Exit: {}\n{}\n{}", output.status, stdout, stderr))
}
