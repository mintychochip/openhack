use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::Path;

/// Top-level deploy state tracking file stored at `.openhack/deploy-state.toml`.
///
/// Tracks per-service deploy targets (docker-compose, aws-lambda, kubernetes),
/// endpoints, image tags, and shared infrastructure outputs so that
/// `openhack migrate` knows the current state of every service and can compute
/// the diff needed to move services between targets.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeployState {
    pub version: String,
    pub meta: DeployStateMeta,
    pub deploy_target: DeployStateTarget,
    pub services: BTreeMap<String, ServiceDeployState>,
    pub infrastructure: InfrastructureState,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeployStateMeta {
    pub project_name: String,
    pub environment: String,
    pub last_updated: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeployStateTarget {
    pub current: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceDeployState {
    pub target: String,
    pub endpoint: String,
    pub image_tag: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InfrastructureState {
    pub db_endpoint: String,
    pub redis_url: String,
    pub s3_bucket: String,
    pub api_gateway_url: String,
}

#[derive(Debug, thiserror::Error)]
pub enum DeployStateError {
    #[error("failed to read deploy state file: {0}")]
    Read(#[source] std::io::Error),
    #[error("failed to parse deploy state TOML: {0}")]
    Parse(#[from] toml::de::Error),
    #[error("failed to serialize deploy state: {0}")]
    Serialize(#[from] toml::ser::Error),
    #[error("failed to write deploy state file: {0}")]
    Write(#[source] std::io::Error),
}

/// Ordered list of all service keys matching the canonical TOML schema.
const ALL_SERVICE_KEYS: &[(&str, u16)] = &[
    ("auth", 3001),
    ("core", 3002),
    ("gateway", 8000),
    ("judging", 3003),
    ("leaderboard", 3004),
    ("mail", 3005),
    ("notify", 3006),
    ("ai", 3007),
    ("analytics", 3008),
    ("sponsors", 3009),
    ("media", 3010),
];

/// Deploy targets that services can cycle through when toggled in the Migrate screen.
const TARGET_CYCLE: &[&str] = &["docker-compose", "aws-lambda", "kubernetes"];

impl DeployState {
    /// Load deploy state from a TOML file on disk.
    ///
    /// # Errors
    /// Returns `DeployStateError::Read` if the file cannot be read,
    /// or `DeployStateError::Parse` if the TOML is invalid.
    pub fn load(path: &Path) -> Result<Self, DeployStateError> {
        let content = std::fs::read_to_string(path).map_err(DeployStateError::Read)?;
        toml::from_str(&content).map_err(DeployStateError::Parse)
    }

    /// Save deploy state to a TOML file, creating parent directories as needed.
    ///
    /// # Errors
    /// Returns `DeployStateError::Serialize` if serialization fails,
    /// or `DeployStateError::Write` if directory creation or file write fails.
    pub fn save(&self, path: &Path) -> Result<(), DeployStateError> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(DeployStateError::Write)?;
        }
        let content = toml::to_string_pretty(self).map_err(DeployStateError::Serialize)?;
        std::fs::write(path, content).map_err(DeployStateError::Write)
    }

    /// Canonical path for the deploy-state file within a project directory.
    pub fn path_for_project(project_dir: &Path) -> std::path::PathBuf {
        project_dir.join(".openhack").join("deploy-state.toml")
    }

    /// Build a `DeployState` from the current `App` configuration after an initial deploy.
    ///
    /// Every enabled service gets an entry with the app's deploy target, a
    /// localhost or placeholder endpoint, and the configured image tag.
    pub fn from_app(app: &crate::app::App) -> Self {
        let mut services = BTreeMap::new();
        for svc in &app.enabled_services {
            let endpoint = match app.deploy_target {
                crate::app::DeployTarget::DockerCompose => {
                    format!("http://localhost:{}", svc.port())
                }
                crate::app::DeployTarget::AwsLambda => String::new(),
                crate::app::DeployTarget::Kubernetes => String::new(),
                _ => String::new(),
            };
            services.insert(
                format!("{svc:?}").to_lowercase(),
                ServiceDeployState {
                    target: app.deploy_target.to_string(),
                    endpoint,
                    image_tag: app.ecr_image_tag.clone(),
                },
            );
        }

        Self {
            version: "1.0".into(),
            meta: DeployStateMeta {
                project_name: app.name.clone(),
                environment: app.environment.clone(),
                last_updated: chrono::Utc::now().to_rfc3339(),
            },
            deploy_target: DeployStateTarget {
                current: app.deploy_target.to_string(),
            },
            services,
            infrastructure: InfrastructureState {
                db_endpoint: app.database_url(),
                redis_url: app.redis_url.clone(),
                s3_bucket: app.aws_s3_bucket.clone(),
                api_gateway_url: String::new(),
            },
        }
    }

    /// Create a default deploy state with all 11 services set to `docker-compose`.
    pub fn init_default(project_name: &str, environment: &str) -> Self {
        let mut services = BTreeMap::new();
        for &(key, port) in ALL_SERVICE_KEYS {
            services.insert(
                key.to_string(),
                ServiceDeployState {
                    target: "docker-compose".into(),
                    endpoint: format!("http://localhost:{port}"),
                    image_tag: "latest".into(),
                },
            );
        }

        Self {
            version: "1.0".into(),
            meta: DeployStateMeta {
                project_name: project_name.into(),
                environment: environment.into(),
                last_updated: String::new(),
            },
            deploy_target: DeployStateTarget {
                current: "docker-compose".into(),
            },
            services,
            infrastructure: InfrastructureState {
                db_endpoint: String::new(),
                redis_url: String::new(),
                s3_bucket: String::new(),
                api_gateway_url: String::new(),
            },
        }
    }

    /// Return the target for a given service key, defaulting to the global deploy target.
    #[must_use]
    pub fn service_target(&self, service_key: &str) -> &str {
        self.services
            .get(service_key)
            .map_or(&self.deploy_target.current, |s| s.target.as_str())
    }

    /// Cycle a service's deploy target through: docker-compose → aws-lambda → kubernetes → docker-compose.
    pub fn toggle_service_target(&mut self, service_key: &str) {
        if let Some(svc) = self.services.get_mut(service_key) {
            let next = TARGET_CYCLE
                .iter()
                .position(|&t| t == svc.target)
                .map_or("docker-compose", |i| {
                    TARGET_CYCLE[(i + 1) % TARGET_CYCLE.len()]
                });
            svc.target = next.to_string();
        }
    }

    /// List services whose target differs from the global `deploy_target.current`.
    #[must_use]
    pub fn migrated_services(&self) -> Vec<(&str, &str)> {
        self.services
            .iter()
            .filter(|(_, s)| s.target != self.deploy_target.current)
            .map(|(name, s)| (name.as_str(), s.target.as_str()))
            .collect()
    }

    /// Number of services whose target differs from the global deploy target.
    #[must_use]
    pub fn migration_count(&self) -> usize {
        self.migrated_services().len()
    }

    /// Ordered list of all service keys for iteration in the Migrate screen.
    #[must_use]
    #[allow(clippy::unused_self)]
    pub fn service_keys_ordered(&self) -> Vec<&str> {
        ALL_SERVICE_KEYS.iter().map(|(k, _)| *k).collect()
    }

    /// Get port for a service key from the canonical list.
    #[must_use]
    pub fn port_for_key(key: &str) -> u16 {
        ALL_SERVICE_KEYS
            .iter()
            .find(|(k, _)| *k == key)
            .map_or(3000, |&(_, p)| p)
    }

    /// Update the `last_updated` timestamp to now.
    pub fn touch(&mut self) {
        self.meta.last_updated = chrono::Utc::now().to_rfc3339();
    }

    /// Update endpoint for a service based on its target and available infrastructure.
    pub fn update_endpoint_for_target(&mut self, service_key: &str, namespace: &str) {
        let Some(svc) = self.services.get_mut(service_key) else {
            return;
        };
        let port = Self::port_for_key(service_key);
        svc.endpoint = match svc.target.as_str() {
            "docker-compose" => format!("http://localhost:{port}"),
            "aws-lambda" => {
                if self.infrastructure.api_gateway_url.is_empty() {
                    String::new()
                } else {
                    format!(
                        "{}/api/{}",
                        self.infrastructure.api_gateway_url.trim_end_matches('/'),
                        service_key.replace('-', "-svc")
                    )
                }
            }
            "kubernetes" => {
                format!(
                    "http://{}-svc.{}.svc.cluster.local:{port}",
                    service_key, namespace
                )
            }
            _ => String::new(),
        };
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn make_state() -> DeployState {
        DeployState::init_default("test-project", "dev")
    }

    #[test]
    fn init_default_has_all_11_services() {
        let state = make_state();
        assert_eq!(state.services.len(), 11);
    }

    #[test]
    fn init_default_all_docker_compose() {
        let state = make_state();
        for (key, svc) in &state.services {
            assert_eq!(
                svc.target, "docker-compose",
                "service {key} should be docker-compose"
            );
        }
    }

    #[test]
    fn service_keys_ordered_matches_all_service_keys() {
        let state = make_state();
        let keys = state.service_keys_ordered();
        assert_eq!(keys.len(), 11);
        assert_eq!(keys[0], "auth");
        assert_eq!(keys[2], "gateway");
    }

    #[test]
    fn port_for_key_returns_correct_ports() {
        assert_eq!(DeployState::port_for_key("auth"), 3001);
        assert_eq!(DeployState::port_for_key("gateway"), 8000);
        assert_eq!(DeployState::port_for_key("media"), 3010);
        assert_eq!(DeployState::port_for_key("nonexistent"), 3000);
    }

    #[test]
    fn toggle_cycles_through_targets() {
        let mut state = make_state();
        assert_eq!(state.service_target("auth"), "docker-compose");
        state.toggle_service_target("auth");
        assert_eq!(state.service_target("auth"), "aws-lambda");
        state.toggle_service_target("auth");
        assert_eq!(state.service_target("auth"), "kubernetes");
        state.toggle_service_target("auth");
        assert_eq!(state.service_target("auth"), "docker-compose");
    }

    #[test]
    fn toggle_nonexistent_service_is_noop() {
        let mut state = make_state();
        state.toggle_service_target("does-not-exist");
        assert_eq!(state.migration_count(), 0);
    }

    #[test]
    fn migrated_services_tracks_diff_from_global() {
        let mut state = make_state();
        assert_eq!(state.migration_count(), 0);
        state.toggle_service_target("auth");
        assert_eq!(state.migration_count(), 1);
        let migrated = state.migrated_services();
        assert_eq!(migrated[0].0, "auth");
        assert_eq!(migrated[0].1, "aws-lambda");
    }

    #[test]
    fn service_target_defaults_to_global_for_missing_key() {
        let state = make_state();
        assert_eq!(state.service_target("nonexistent"), "docker-compose");
    }

    #[test]
    fn save_and_load_roundtrip() {
        let dir = std::env::temp_dir().join("openhack-test-deploy-state");
        let path = dir.join("deploy-state.toml");

        let mut state = make_state();
        state.toggle_service_target("auth");
        state.toggle_service_target("core");

        state.save(&path).expect("save should succeed");
        let loaded = DeployState::load(&path).expect("load should succeed");

        assert_eq!(loaded.version, "1.0");
        assert_eq!(loaded.services.len(), 11);
        assert_eq!(loaded.service_target("auth"), "aws-lambda");
        assert_eq!(loaded.service_target("core"), "aws-lambda");
        assert_eq!(loaded.service_target("gateway"), "docker-compose");
        assert_eq!(loaded.deploy_target.current, "docker-compose");

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn load_nonexistent_file_returns_read_error() {
        let result = DeployState::load(Path::new("/nonexistent/path/state.toml"));
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(matches!(err, DeployStateError::Read(_)));
    }

    #[test]
    fn update_endpoint_for_docker_compose() {
        let mut state = make_state();
        state.update_endpoint_for_target("auth", "openhack");
        assert_eq!(state.services["auth"].endpoint, "http://localhost:3001");
    }

    #[test]
    fn update_endpoint_for_kubernetes() {
        let mut state = make_state();
        state.services.get_mut("auth").unwrap().target = "kubernetes".into();
        state.update_endpoint_for_target("auth", "production");
        assert_eq!(
            state.services["auth"].endpoint,
            "http://auth-svc.production.svc.cluster.local:3001"
        );
    }

    #[test]
    fn update_endpoint_for_lambda_with_gateway_url() {
        let mut state = make_state();
        state.services.get_mut("auth").unwrap().target = "aws-lambda".into();
        state.infrastructure.api_gateway_url =
            "https://abc123.execute-api.us-east-1.amazonaws.com".into();
        state.update_endpoint_for_target("auth", "openhack");
        assert_eq!(
            state.services["auth"].endpoint,
            "https://abc123.execute-api.us-east-1.amazonaws.com/api/auth"
        );
    }

    #[test]
    fn update_endpoint_for_lambda_multi_word_service() {
        let mut state = make_state();
        state.services.get_mut("leaderboard").unwrap().target = "aws-lambda".into();
        state.infrastructure.api_gateway_url =
            "https://abc123.execute-api.us-east-1.amazonaws.com".into();
        state.update_endpoint_for_target("leaderboard", "openhack");
        assert_eq!(
            state.services["leaderboard"].endpoint,
            "https://abc123.execute-api.us-east-1.amazonaws.com/api/leaderboard"
        );
    }

    #[test]
    fn update_endpoint_for_lambda_without_gateway_url() {
        let mut state = make_state();
        state.services.get_mut("auth").unwrap().target = "aws-lambda".into();
        state.update_endpoint_for_target("auth", "openhack");
        assert!(state.services["auth"].endpoint.is_empty());
    }

    #[test]
    fn path_for_project() {
        let path = DeployState::path_for_project(Path::new("/my/project"));
        assert_eq!(
            path,
            PathBuf::from("/my/project/.openhack/deploy-state.toml")
        );
    }
}
