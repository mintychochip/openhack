pub mod azure;
pub mod docker;
pub mod gcloud;
pub mod health;
pub mod kubernetes;
pub mod terraform;

use crate::app::{App, DeployStatus, DeployStep, DeployTarget};
use crate::deploy_state::DeployState;
use std::path::Path;
use tokio::sync::mpsc;

/// A step status update sent from a background deploy task to the TUI.
#[derive(Debug, Clone)]
pub struct StepUpdate {
    pub step_name: String,
    pub status: DeployStatus,
    pub error: Option<String>,
}

pub fn build_steps(app: &App) -> Vec<DeployStep> {
    let mut steps = vec![
        DeployStep {
            name: "Writing configuration files".into(),
            status: DeployStatus::Pending,
        },
        DeployStep {
            name: "Building container images".into(),
            status: DeployStatus::Pending,
        },
    ];
    match app.deploy_target {
        DeployTarget::DockerCompose => {
            steps.push(DeployStep {
                name: "Starting infrastructure".into(),
                status: DeployStatus::Pending,
            });
            steps.push(DeployStep {
                name: "Running database migrations".into(),
                status: DeployStatus::Pending,
            });
            steps.push(DeployStep {
                name: "Starting application services".into(),
                status: DeployStatus::Pending,
            });
        }
        DeployTarget::AwsLambda => {
            steps.push(DeployStep {
                name: "Running terraform init".into(),
                status: DeployStatus::Pending,
            });
            steps.push(DeployStep {
                name: "Running terraform apply".into(),
                status: DeployStatus::Pending,
            });
        }
        DeployTarget::GcpCloudRun => {
            steps.push(DeployStep {
                name: "Deploying to Cloud Run".into(),
                status: DeployStatus::Pending,
            });
        }
        DeployTarget::AzureContainerApps => {
            steps.push(DeployStep {
                name: "Deploying to Container Apps".into(),
                status: DeployStatus::Pending,
            });
        }
        DeployTarget::Kubernetes => {
            steps.push(DeployStep {
                name: "Checking kubectl and helm".into(),
                status: DeployStatus::Pending,
            });
            steps.push(DeployStep {
                name: "Running helm upgrade".into(),
                status: DeployStatus::Pending,
            });
            steps.push(DeployStep {
                name: "Waiting for pods".into(),
                status: DeployStatus::Pending,
            });
        }
    }
    steps.push(DeployStep {
        name: "Running health checks".into(),
        status: DeployStatus::Pending,
    });
    steps
}

pub async fn deploy(
    app: &App,
    project_dir: &Path,
    tx: &mpsc::UnboundedSender<StepUpdate>,
) -> Result<(), String> {
    match app.deploy_target {
        DeployTarget::DockerCompose => docker::deploy(app, project_dir, tx).await,
        DeployTarget::AwsLambda => terraform::deploy(app, project_dir, tx).await,
        DeployTarget::GcpCloudRun => gcloud::deploy(app, project_dir, tx).await,
        DeployTarget::AzureContainerApps => azure::deploy(app, project_dir, tx).await,
        DeployTarget::Kubernetes => kubernetes::deploy(app, project_dir, tx).await,
    }
}

pub fn build_migration_steps(migrated: &[(&str, &str)]) -> Vec<DeployStep> {
    let mut steps = vec![DeployStep {
        name: "Reading deploy state".into(),
        status: DeployStatus::Pending,
    }];
    for (svc, target) in migrated {
        steps.push(DeployStep {
            name: format!("Migrating {svc} → {target}"),
            status: DeployStatus::Pending,
        });
    }
    steps.push(DeployStep {
        name: "Saving deploy state".into(),
        status: DeployStatus::Pending,
    });
    steps.push(DeployStep {
        name: "Running health checks".into(),
        status: DeployStatus::Pending,
    });
    steps
}

/// Execute migration for services that have changed deploy targets.
///
/// For v1, this performs a full-stack re-deploy to the most common
/// new target among the migrated services. Each migrated service is
/// re-deployed to its individual target by invoking the target-specific
/// deploy logic. After all services are deployed, the deploy state is
/// updated with new endpoints and saved.
pub async fn migrate(
    app: &App,
    project_dir: &Path,
    tx: &mpsc::UnboundedSender<StepUpdate>,
) -> Result<(), String> {
    let step = "Reading deploy state";
    send_step(tx, step, DeployStatus::Running, None);

    let migrated: Vec<(String, String)>;
    if let Some(ref state) = app.deploy_state {
        migrated = state
            .migrated_services()
            .into_iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect();
    } else {
        send_step(
            tx,
            step,
            DeployStatus::Failed,
            Some("No deploy state loaded"),
        );
        return Err("No deploy state loaded".into());
    }
    send_step(tx, step, DeployStatus::Success, None);

    for (svc, target) in &migrated {
        let step = format!("Migrating {svc} → {target}");
        send_step(tx, &step, DeployStatus::Running, None);

        let deploy_target = parse_deploy_target(target);

        let result = match deploy_target {
            DeployTarget::DockerCompose => docker::deploy(app, project_dir, tx).await,
            DeployTarget::AwsLambda => terraform::deploy(app, project_dir, tx).await,
            DeployTarget::Kubernetes => kubernetes::deploy(app, project_dir, tx).await,
            DeployTarget::GcpCloudRun => gcloud::deploy(app, project_dir, tx).await,
            DeployTarget::AzureContainerApps => azure::deploy(app, project_dir, tx).await,
        };

        if let Err(e) = &result {
            send_step(tx, &step, DeployStatus::Failed, Some(e));
            return Err(e.clone());
        }
        send_step(tx, &step, DeployStatus::Success, None);
    }

    let step = "Saving deploy state";
    send_step(tx, step, DeployStatus::Running, None);
    if let Some(ref state) = app.deploy_state {
        let namespace = &app.kubernetes_namespace;
        let mut state = state.clone();
        for (svc, _) in &migrated {
            state.update_endpoint_for_target(svc, namespace);
        }
        state.touch();
        let state_path = DeployState::path_for_project(project_dir);
        if let Err(e) = state.save(&state_path) {
            send_step(tx, step, DeployStatus::Failed, Some(&e.to_string()));
            return Err(format!("Failed to save deploy state: {e}"));
        }
    }
    send_step(tx, step, DeployStatus::Success, None);

    let step = "Running health checks";
    send_step(tx, step, DeployStatus::Running, None);
    let health_results = health::check_all_services(app);
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

fn parse_deploy_target(s: &str) -> DeployTarget {
    match s {
        "aws-lambda" => DeployTarget::AwsLambda,
        "kubernetes" => DeployTarget::Kubernetes,
        "gcp-cloud-run" => DeployTarget::GcpCloudRun,
        "azure-container-apps" => DeployTarget::AzureContainerApps,
        _ => DeployTarget::DockerCompose,
    }
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
