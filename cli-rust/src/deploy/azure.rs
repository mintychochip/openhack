use crate::app::{App, DeployStatus};
use crate::deploy::StepUpdate;
use std::path::Path;
use tokio::process::Command;
use tokio::sync::mpsc;

pub async fn deploy(
    app: &App,
    project_dir: &Path,
    tx: &mpsc::UnboundedSender<StepUpdate>,
) -> Result<(), String> {
    let step = "check-az";
    send_step(tx, step, DeployStatus::Running, None);

    if which::which("az").is_err() {
        let msg = "az CLI is not installed or not on PATH".to_string();
        send_step(tx, step, DeployStatus::Failed, Some(&msg));
        return Err(msg);
    }
    send_step(tx, step, DeployStatus::Success, None);

    let step = "azure-deploy";
    send_step(tx, step, DeployStatus::Running, None);
    let deploy_result = run_deploy_script(project_dir).await;
    if let Err(e) = &deploy_result {
        send_step(tx, step, DeployStatus::Failed, Some(e));
        return Err(e.clone());
    }
    send_step(tx, step, DeployStatus::Success, None);

    let step = "health-check";
    send_step(tx, step, DeployStatus::Running, None);
    let health_results = crate::deploy::health::check_all_services(app);
    let failed: Vec<&str> = health_results
        .iter()
        .filter(|(_, ok)| !ok)
        .map(|(name, _)| name.as_str())
        .collect();
    if failed.is_empty() {
        send_step(tx, step, DeployStatus::Success, None);
    } else {
        let msg = format!("unhealthy services: {}", failed.join(", "));
        send_step(tx, step, DeployStatus::Failed, Some(&msg));
        return Err(msg);
    }

    Ok(())
}

async fn run_deploy_script(project_dir: &Path) -> Result<(), String> {
    let script_path = project_dir.join("deploy").join("azure").join("deploy.sh");

    let output = Command::new("bash")
        .arg(&script_path)
        .current_dir(project_dir)
        .output()
        .await
        .map_err(|e| format!("failed to execute azure deploy script: {e}"))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("azure deploy.sh failed: {stderr}"));
    }
    Ok(())
}

fn send_step(
    tx: &mpsc::UnboundedSender<StepUpdate>,
    step_name: &str,
    status: DeployStatus,
    error: Option<&str>,
) {
    let _ = tx.send(StepUpdate {
        step_name: step_name.to_string(),
        status,
        error: error.map(String::from),
    });
}
