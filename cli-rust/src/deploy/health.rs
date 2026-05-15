use crate::app::{App, DeployTarget};

#[allow(clippy::module_name_repetitions)]
#[must_use]
pub fn check_all_services(app: &App) -> Vec<(String, bool)> {
    let urls = build_health_urls(app);
    let mut results = Vec::with_capacity(urls.len());

    for (name, url) in urls {
        let ok = check_health_sync(&url);
        results.push((name, ok));
    }

    results
}

#[must_use]
pub fn check_health(url: &str) -> bool {
    check_health_sync(url)
}

fn check_health_sync(url: &str) -> bool {
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_io()
        .build();

    match rt {
        Ok(runtime) => runtime.block_on(async {
            match reqwest::Client::new()
                .get(format!("{url}/health"))
                .timeout(std::time::Duration::from_secs(10))
                .send()
                .await
            {
                Ok(resp) => resp.status().is_success(),
                Err(_) => false,
            }
        }),
        Err(_) => false,
    }
}

fn build_health_urls(app: &App) -> Vec<(String, String)> {
    match app.deploy_target {
        DeployTarget::DockerCompose => build_docker_urls(app),
        DeployTarget::AwsLambda => build_aws_urls(app),
        DeployTarget::GcpCloudRun => build_gcp_urls(app),
        DeployTarget::AzureContainerApps => build_azure_urls(app),
        DeployTarget::Kubernetes => build_k8s_urls(app),
    }
}

fn build_docker_urls(app: &App) -> Vec<(String, String)> {
    app.enabled_services
        .iter()
        .map(|svc| {
            let url = format!("http://localhost:{}", svc.port());
            (svc.docker_name().to_string(), url)
        })
        .collect()
}

fn build_aws_urls(app: &App) -> Vec<(String, String)> {
    let api_id = match &app.aws_api_gateway_id {
        Some(id) => id.clone(),
        None => return Vec::new(),
    };
    let region = &app.aws_region;

    app.enabled_services
        .iter()
        .map(|svc| {
            let url = format!(
                "https://{api_id}.execute-api.{region}.amazonaws.com/api/{svc}",
                svc = svc.docker_name()
            );
            (svc.docker_name().to_string(), url)
        })
        .collect()
}

fn build_gcp_urls(app: &App) -> Vec<(String, String)> {
    let project = &app.gcp_project_id;
    if project.is_empty() {
        return Vec::new();
    }

    app.enabled_services
        .iter()
        .map(|svc| {
            let url = format!("https://{svc}-{project}.run.app", svc = svc.docker_name());
            (svc.docker_name().to_string(), url)
        })
        .collect()
}

fn build_azure_urls(app: &App) -> Vec<(String, String)> {
    if app.azure_resource_group.is_empty() {
        return Vec::new();
    }
    let env_name = &app.environment;
    let region = &app.azure_location;

    app.enabled_services
        .iter()
        .map(|svc| {
            let url = format!("https://{env_name}.{region}.azurecontainerapps.io");
            (svc.docker_name().to_string(), url)
        })
        .collect()
}

fn build_k8s_urls(app: &App) -> Vec<(String, String)> {
    let namespace = &app.kubernetes_namespace;

    app.enabled_services
        .iter()
        .map(|svc| {
            let url = format!(
                "http://{}.{}.svc.cluster.local:{}",
                svc.docker_name(),
                namespace,
                svc.port()
            );
            (svc.docker_name().to_string(), url)
        })
        .collect()
}
