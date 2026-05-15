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
    let step = "check-terraform";
    send_step(tx, step, DeployStatus::Running, None);

    if which::which("terraform").is_err() {
        let msg = "terraform is not installed or not on PATH".to_string();
        send_step(tx, step, DeployStatus::Failed, Some(&msg));
        return Err(msg);
    }
    send_step(tx, step, DeployStatus::Success, None);

    let tf_dir = project_dir.join("infra").join("terraform");

    let step = "terraform-init";
    send_step(tx, step, DeployStatus::Running, None);
    let init_result = run_terraform_init(&tf_dir).await;
    if let Err(e) = &init_result {
        send_step(tx, step, DeployStatus::Failed, Some(e));
        return Err(e.clone());
    }
    send_step(tx, step, DeployStatus::Success, None);

    let step = "terraform-apply";
    send_step(tx, step, DeployStatus::Running, None);
    let apply_result = run_terraform_apply(&tf_dir, &app.environment, &app.aws_region).await;
    if let Err(e) = &apply_result {
        send_step(tx, step, DeployStatus::Failed, Some(e));
        return Err(e.clone());
    }
    send_step(tx, step, DeployStatus::Success, None);

    let step = "parse-outputs";
    send_step(tx, step, DeployStatus::Running, None);
    let output_result = parse_terraform_outputs(&tf_dir).await;
    match &output_result {
        Ok(_gateway_url) => {
            send_step(tx, step, DeployStatus::Success, None);
        }
        Err(e) => {
            send_step(tx, step, DeployStatus::Failed, Some(e));
            return Err(e.clone());
        }
    }

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

async fn run_terraform_init(tf_dir: &Path) -> Result<(), String> {
    let output = Command::new("terraform")
        .arg("init")
        .current_dir(tf_dir)
        .output()
        .await
        .map_err(|e| format!("failed to execute terraform init: {e}"))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("terraform init failed: {stderr}"));
    }
    Ok(())
}

async fn run_terraform_apply(tf_dir: &Path, env: &str, region: &str) -> Result<(), String> {
    let var_file = format!("envs/{env}-{region}.tfvars");

    let output = Command::new("terraform")
        .args(["apply", "-var-file", &var_file, "-auto-approve"])
        .current_dir(tf_dir)
        .output()
        .await
        .map_err(|e| format!("failed to execute terraform apply: {e}"))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("terraform apply failed: {stderr}"));
    }
    Ok(())
}

async fn parse_terraform_outputs(tf_dir: &Path) -> Result<String, String> {
    let output = Command::new("terraform")
        .args(["output", "-raw", "api_gateway_url"])
        .current_dir(tf_dir)
        .output()
        .await
        .map_err(|e| format!("failed to execute terraform output: {e}"))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("terraform output failed: {stderr}"));
    }

    let url = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if url.is_empty() {
        return Err("terraform output returned empty api_gateway_url".into());
    }
    Ok(url)
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
