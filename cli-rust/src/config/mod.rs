pub mod azure;
pub mod docker_compose;
pub mod env_file;
pub mod gcp;
pub mod kong;
pub mod kubernetes;
pub mod terraform;

use crate::app::{App, DeployTarget};
use std::path::Path;

pub fn generate_all(app: &App, output_dir: &Path) -> Result<(), Box<dyn std::error::Error>> {
    env_file::generate(app, output_dir)?;

    match app.deploy_target {
        DeployTarget::DockerCompose => {
            docker_compose::generate_override(app, output_dir)?;
            kong::generate(app, output_dir)?;
        }
        DeployTarget::AwsLambda => {
            terraform::generate(app, output_dir)?;
        }
        DeployTarget::GcpCloudRun => {
            gcp::generate(app, output_dir)?;
        }
        DeployTarget::AzureContainerApps => {
            azure::generate(app, output_dir)?;
        }
        DeployTarget::Kubernetes => {
            kubernetes::generate(app, output_dir)?;
        }
    }
    Ok(())
}
