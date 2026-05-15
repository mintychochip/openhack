use crate::app::{App, DeployStatus};
use crate::deploy::StepUpdate;
use std::path::Path;
use tokio::process::Command;
use tokio::sync::mpsc;

/// Deploy all enabled services to a Kubernetes cluster via Helm.
///
/// Verifies `kubectl` and `helm` are available, runs `helm upgrade --install`,
/// waits for pods to become ready, then runs health checks against the K8s
/// service DNS names.
pub async fn deploy(
    app: &App,
    project_dir: &Path,
    tx: &mpsc::UnboundedSender<StepUpdate>,
) -> Result<(), String> {
    let step = "check-kubectl";
    send_step(tx, step, DeployStatus::Running, None);
    if which::which("kubectl").is_err() {
        let msg = "kubectl is not installed or not on PATH".to_string();
        send_step(tx, step, DeployStatus::Failed, Some(&msg));
        return Err(msg);
    }
    send_step(tx, step, DeployStatus::Success, None);

    let step = "check-helm";
    send_step(tx, step, DeployStatus::Running, None);
    if which::which("helm").is_err() {
        let msg = "helm is not installed or not on PATH".to_string();
        send_step(tx, step, DeployStatus::Failed, Some(&msg));
        return Err(msg);
    }
    send_step(tx, step, DeployStatus::Success, None);

    let step = "check-k8s-context";
    send_step(tx, step, DeployStatus::Running, None);
    let context_result = check_k8s_context().await;
    if let Err(e) = &context_result {
        send_step(tx, step, DeployStatus::Failed, Some(e));
        return Err(e.clone());
    }
    send_step(tx, step, DeployStatus::Success, None);

    let step = "helm-dep-build";
    send_step(tx, step, DeployStatus::Running, None);
    let chart_dir = project_dir.join("deploy").join("helm").join("openhack");
    let dep_result = run_helm_dep_build(&chart_dir).await;
    if let Err(e) = &dep_result {
        send_step(tx, step, DeployStatus::Failed, Some(e));
        return Err(e.clone());
    }
    send_step(tx, step, DeployStatus::Success, None);

    let step = "helm-upgrade";
    send_step(tx, step, DeployStatus::Running, None);
    let upgrade_result = run_helm_upgrade(&chart_dir, &app.kubernetes_namespace).await;
    if let Err(e) = &upgrade_result {
        send_step(tx, step, DeployStatus::Failed, Some(e));
        return Err(e.clone());
    }
    send_step(tx, step, DeployStatus::Success, None);

    let step = "wait-pods-ready";
    send_step(tx, step, DeployStatus::Running, None);
    let wait_result = wait_for_pods(&app.kubernetes_namespace).await;
    if let Err(e) = &wait_result {
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

async fn check_k8s_context() -> Result<(), String> {
    let output = Command::new("kubectl")
        .args(["config", "current-context"])
        .output()
        .await
        .map_err(|e| format!("failed to execute kubectl config current-context: {e}"))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("no active kubectl context: {stderr}"));
    }
    Ok(())
}

async fn run_helm_dep_build(chart_dir: &Path) -> Result<(), String> {
    let output = Command::new("helm")
        .args(["dep", "build"])
        .current_dir(chart_dir)
        .output()
        .await
        .map_err(|e| format!("failed to execute helm dep build: {e}"))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("helm dep build failed: {stderr}"));
    }
    Ok(())
}

async fn run_helm_upgrade(chart_dir: &Path, namespace: &str) -> Result<(), String> {
    let output = Command::new("helm")
        .args([
            "upgrade",
            "--install",
            "openhack",
            ".",
            "--namespace",
            namespace,
            "--create-namespace",
            "--wait",
            "--timeout",
            "5m",
        ])
        .current_dir(chart_dir)
        .output()
        .await
        .map_err(|e| format!("failed to execute helm upgrade: {e}"))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("helm upgrade failed: {stderr}"));
    }
    Ok(())
}

async fn wait_for_pods(namespace: &str) -> Result<(), String> {
    let output = Command::new("kubectl")
        .args([
            "wait",
            "pods",
            "--for=condition=ready",
            "--all",
            "--namespace",
            namespace,
            "--timeout=300s",
        ])
        .output()
        .await
        .map_err(|e| format!("failed to execute kubectl wait: {e}"))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("pods not ready within timeout: {stderr}"));
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
