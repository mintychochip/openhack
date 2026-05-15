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
    let step = "check-docker";
    send_step(tx, step, DeployStatus::Running, None);

    if which::which("docker").is_err() {
        let msg = "docker is not installed or not on PATH".to_string();
        send_step(tx, step, DeployStatus::Failed, Some(&msg));
        return Err(msg);
    }
    send_step(tx, step, DeployStatus::Success, None);

    let step = "docker-compose-build";
    send_step(tx, step, DeployStatus::Running, None);
    let build_result = run_docker_compose_build(project_dir).await;
    if let Err(e) = &build_result {
        send_step(tx, step, DeployStatus::Failed, Some(e));
        return Err(e.clone());
    }
    send_step(tx, step, DeployStatus::Success, None);

    let step = "docker-compose-up";
    send_step(tx, step, DeployStatus::Running, None);
    let up_result = run_docker_compose_up(project_dir).await;
    if let Err(e) = &up_result {
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

async fn run_docker_compose_build(project_dir: &Path) -> Result<(), String> {
    let output = Command::new("docker")
        .args(["compose", "build"])
        .current_dir(project_dir)
        .output()
        .await
        .map_err(|e| format!("failed to execute docker compose build: {e}"))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("docker compose build failed: {stderr}"));
    }
    Ok(())
}

async fn run_docker_compose_up(project_dir: &Path) -> Result<(), String> {
    let args = vec!["compose", "up", "-d", "--profile", "full"];

    let output = Command::new("docker")
        .args(&args)
        .current_dir(project_dir)
        .output()
        .await
        .map_err(|e| format!("failed to execute docker compose up: {e}"))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("docker compose up failed: {stderr}"));
    }
    Ok(())
}

#[allow(dead_code)]
fn build_docker_profiles(app: &App) -> Vec<String> {
    let core_profiles = ["redis"];
    let full_profiles = ["full", "redis"];

    let profiles: &[&str] = if app.enabled_services.iter().any(|s| !s.is_core_service()) {
        &full_profiles
    } else {
        &core_profiles
    };

    profiles.iter().map(|p| format!("--profile={p}")).collect()
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
