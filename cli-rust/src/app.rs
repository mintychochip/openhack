use serde::{Deserialize, Serialize};

use crate::deploy_state::DeployState;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Screen {
    Welcome,
    DeployTarget,
    Services,
    Database,
    Storage,
    Email,
    Secrets,
    Review,
    CostEstimate,
    Progress,
    Done,
    Migrate,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DeployTarget {
    DockerCompose,
    AwsLambda,
    GcpCloudRun,
    AzureContainerApps,
    Kubernetes,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DeployStatus {
    Pending,
    Running,
    Success,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeployStep {
    pub name: String,
    pub status: DeployStatus,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ServiceName {
    Auth,
    Core,
    Mail,
    Judging,
    Leaderboard,
    Notify,
    Ai,
    Analytics,
    Sponsors,
    Media,
    Gateway,
}

impl ServiceName {
    pub fn docker_name(self) -> &'static str {
        match self {
            Self::Auth => "auth-svc",
            Self::Core => "core-svc",
            Self::Mail => "mail-svc",
            Self::Judging => "judging-svc",
            Self::Leaderboard => "leaderboard-svc",
            Self::Notify => "notify-svc",
            Self::Ai => "ai-svc",
            Self::Analytics => "analytics-svc",
            Self::Sponsors => "sponsors-svc",
            Self::Media => "media-svc",
            Self::Gateway => "gateway-svc",
        }
    }

    #[allow(clippy::match_same_arms)]
    pub fn port(self) -> u16 {
        match self {
            Self::Auth => 3001,
            Self::Core => 3002,
            Self::Mail => 3005,
            Self::Judging => 3003,
            Self::Leaderboard => 3004,
            Self::Notify => 3006,
            Self::Ai => 3007,
            Self::Analytics => 3008,
            Self::Sponsors => 3009,
            Self::Media => 3010,
            Self::Gateway => 8000,
        }
    }

    pub fn route_path(self) -> &'static str {
        match self {
            Self::Auth => "/api/auth",
            Self::Core => "/api/core",
            Self::Mail => "/api/mail",
            Self::Judging => "/api/judging",
            Self::Leaderboard => "/api/leaderboard",
            Self::Notify => "/api/notify",
            Self::Ai => "/api/ai",
            Self::Analytics => "/api/analytics",
            Self::Sponsors => "/api/sponsors",
            Self::Media => "/api/media",
            Self::Gateway => "/api/events",
        }
    }

    #[allow(clippy::match_same_arms)]
    pub fn rate_limit(self) -> u32 {
        match self {
            Self::Auth | Self::Judging | Self::Analytics => 100,
            Self::Core | Self::Leaderboard | Self::Gateway => 300,
            Self::Mail | Self::Notify | Self::Sponsors | Self::Media => 50,
            Self::Ai => 30,
        }
    }

    #[allow(clippy::match_same_arms)]
    pub fn request_size_mb(self) -> u32 {
        match self {
            Self::Core => 10,
            Self::Mail => 5,
            Self::Media => 100,
            Self::Auth
            | Self::Judging
            | Self::Leaderboard
            | Self::Notify
            | Self::Ai
            | Self::Analytics
            | Self::Sponsors
            | Self::Gateway => 1,
        }
    }

    pub fn is_core_service(self) -> bool {
        matches!(self, Self::Auth | Self::Core | Self::Gateway)
    }

    #[allow(clippy::missing_panics_doc)]
    pub fn all() -> Vec<ServiceName> {
        vec![
            Self::Auth,
            Self::Core,
            Self::Mail,
            Self::Judging,
            Self::Leaderboard,
            Self::Notify,
            Self::Ai,
            Self::Analytics,
            Self::Sponsors,
            Self::Media,
            Self::Gateway,
        ]
    }
}

impl std::fmt::Display for ServiceName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.docker_name())
    }
}

impl std::fmt::Display for DeployTarget {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::DockerCompose => f.write_str("docker-compose"),
            Self::AwsLambda => f.write_str("aws-lambda"),
            Self::GcpCloudRun => f.write_str("gcp-cloud-run"),
            Self::AzureContainerApps => f.write_str("azure-container-apps"),
            Self::Kubernetes => f.write_str("kubernetes"),
        }
    }
}

#[allow(clippy::struct_excessive_bools)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct App {
    pub name: String,
    pub domain: String,
    pub secret: String,
    pub timezone: String,
    pub environment: String,
    pub log_level: String,
    pub debug: bool,
    pub deploy_target: DeployTarget,
    pub enabled_services: Vec<ServiceName>,

    pub db_username: String,
    pub db_password: String,
    pub db_name: String,
    pub db_host: String,
    pub db_port: u16,
    pub db_pool_min: u32,
    pub db_pool_max: u32,
    pub db_pool_timeout: u32,

    pub redis_url: String,
    pub redis_host: String,
    pub redis_port: u16,
    pub redis_password: String,
    pub redis_pool_min: u32,
    pub redis_pool_max: u32,

    pub jwt_secret: String,
    pub jwt_expiry: String,
    pub refresh_token_expiry: String,
    pub jwt_issuer: String,
    pub password_min_length: u8,
    pub password_require_uppercase: bool,
    pub password_require_lowercase: bool,
    pub password_require_numbers: bool,
    pub password_require_special: bool,
    pub session_timeout: String,
    pub session_cookie_secure: bool,
    pub session_cookie_httponly: bool,
    pub session_cookie_samesite: String,
    pub auth_rate_limit_enabled: bool,
    pub auth_rate_limit_max: u32,
    pub auth_rate_limit_window_ms: u32,

    pub github_enabled: bool,
    pub github_client_id: String,
    pub github_client_secret: String,
    pub google_enabled: bool,
    pub google_client_id: String,
    pub google_client_secret: String,
    pub discord_oauth_enabled: bool,
    pub discord_client_id: String,
    pub discord_client_secret: String,

    pub mfa_totp_enabled: bool,
    pub mfa_totp_issuer: String,
    pub mfa_sms_enabled: bool,
    pub twilio_account_sid: String,
    pub twilio_auth_token: String,
    pub twilio_phone_number: String,

    pub mail_provider: String,
    pub mail_from: String,
    pub mail_from_name: String,
    pub smtp_host: String,
    pub smtp_port: u16,
    pub smtp_user: String,
    pub smtp_password: String,
    pub smtp_tls: bool,
    pub smtp_reject_unauthorized: bool,
    pub sendgrid_api_key: String,
    pub aws_ses_region: String,
    pub mail_rate_limit_enabled: bool,
    pub mail_rate_limit_max_per_hour: u32,

    pub discord_bot_token: String,
    pub discord_guild_id: String,
    pub slack_bot_token: String,
    pub slack_signing_secret: String,
    pub webhook_secret: String,
    pub webhook_retry_count: u32,
    pub webhook_timeout_ms: u32,

    pub ai_provider: String,
    pub ai_enabled: bool,
    pub openai_api_key: String,
    pub openai_model: String,
    pub openai_max_tokens: u32,
    pub openai_temperature: String,
    pub anthropic_api_key: String,
    pub anthropic_model: String,
    pub ollama_enabled: bool,
    pub ollama_base_url: String,
    pub ollama_model: String,
    pub azure_openai_endpoint: String,
    pub azure_openai_api_key: String,
    pub azure_openai_deployment: String,
    pub azure_openai_api_version: String,
    pub ai_feature_chat: bool,
    pub ai_feature_idea_generator: bool,
    pub ai_feature_team_matcher: bool,
    pub ai_feature_code_review: bool,
    pub ai_feature_organizer_insights: bool,
    pub ai_rag_enabled: bool,
    pub ai_rag_top_k: u32,
    pub ai_embedding_model: String,

    pub storage_provider: String,
    pub local_storage_path: String,
    #[allow(clippy::unreadable_literal)]
    pub local_storage_max_size: u64,
    pub minio_endpoint: String,
    pub minio_bucket: String,
    pub minio_access_key: String,
    pub minio_secret_key: String,
    pub minio_use_ssl: bool,
    pub aws_region: String,
    pub aws_s3_bucket: String,
    pub aws_access_key_id: String,
    pub aws_secret_access_key: String,
    pub s3_use_path_style: bool,
    pub gcs_bucket: String,
    pub gcs_credentials: String,
    pub azure_storage_account: String,
    pub azure_storage_key: String,
    pub azure_container: String,
    #[allow(clippy::unreadable_literal)]
    pub max_file_size: u64,
    #[allow(clippy::unreadable_literal)]
    pub max_image_size: u64,
    #[allow(clippy::unreadable_literal)]
    pub max_video_size: u64,

    pub next_public_api_url: String,
    pub next_public_ws_url: String,
    pub next_public_app_name: String,
    pub feature_teams: bool,
    pub feature_projects: bool,
    pub feature_judging: bool,
    pub feature_voting: bool,
    pub feature_sponsors: bool,

    pub gateway_url: String,
    pub gateway_rate_limit_auth: u32,
    pub gateway_rate_limit_core: u32,
    pub gateway_rate_limit_judging: u32,
    pub gateway_rate_limit_mail: u32,
    pub gateway_rate_limit_notify: u32,
    pub gateway_rate_limit_ai: u32,
    pub cors_enabled: bool,
    pub cors_origins: String,
    pub cors_credentials: bool,
    pub cors_max_age: u32,

    pub prometheus_enabled: bool,
    pub prometheus_port: u16,
    pub grafana_enabled: bool,
    pub grafana_admin_password: String,
    pub alertmanager_enabled: bool,
    pub otel_enabled: bool,
    pub otel_endpoint: String,
    pub otel_service_name: String,

    pub https_enabled: bool,
    pub tls_cert_path: String,
    pub tls_key_path: String,
    pub security_hsts_enabled: bool,
    #[allow(clippy::unreadable_literal)]
    pub security_hsts_max_age: u32,
    pub security_frame_options: String,
    pub security_content_type_nosniff: bool,
    pub csp_enabled: bool,
    pub csp_default_src: String,
    pub csp_script_src: String,
    pub csp_style_src: String,

    pub backup_schedule: String,
    pub backup_retention_daily: u32,
    pub backup_retention_weekly: u32,
    pub backup_retention_monthly: u32,
    pub backup_destination: String,
    pub backup_s3_bucket: String,
    pub backup_s3_region: String,

    pub multi_tenancy_enabled: bool,
    pub tenancy_model: String,
    pub default_tenant: String,

    pub dev_hot_reload: bool,
    pub dev_watch_files: bool,
    pub dev_enable_profiling: bool,
    pub dev_enable_query_logging: bool,
    pub dev_enable_swagger: bool,
    pub dev_seed_data: bool,
    pub dev_test_users: bool,

    pub kubernetes_namespace: String,
    pub docker_compose_project_name: String,
    pub health_check_interval: u32,
    pub health_check_timeout: u32,

    pub db_instance_class: String,
    pub db_serverless_min_capacity: f64,
    pub db_serverless_max_capacity: f64,
    pub lambda_memory_size: u32,
    pub lambda_timeout: u32,
    pub ecr_image_tag: String,
    pub domain_name: String,
    pub certificate_arn: String,
    pub vpc_cidr: String,

    pub gcp_project_id: String,
    pub gcp_region: String,

    pub azure_location: String,
    pub azure_resource_group: String,

    pub deploy_steps: Vec<DeployStep>,
    pub deploy_error: Option<String>,
    pub aws_api_gateway_id: Option<String>,

    pub deploy_state: Option<DeployState>,
    pub migrate_selected: usize,

    pub screen: Screen,
    pub selected: usize,
    pub editing: usize,
    pub input_buffer: String,
    pub editing_text: bool,
}

impl App {
    #[allow(clippy::too_many_lines)]
    pub fn default_dev() -> Self {
        Self {
            name: "My Hackathon".into(),
            domain: "hackathon.example.com".into(),
            secret: "your-super-secret-key-min-32-chars-long".into(),
            timezone: "America/New_York".into(),
            environment: "development".into(),
            log_level: "info".into(),
            debug: false,
            deploy_target: DeployTarget::DockerCompose,
            enabled_services: vec![ServiceName::Auth, ServiceName::Core, ServiceName::Gateway],

            db_username: "openhack".into(),
            db_password: "openhack_pass".into(),
            db_name: "openhack".into(),
            db_host: "postgres".into(),
            db_port: 5432,
            db_pool_min: 5,
            db_pool_max: 20,
            db_pool_timeout: 30000,

            redis_url: "redis://redis:6379".into(),
            redis_host: "redis".into(),
            redis_port: 6379,
            redis_password: String::new(),
            redis_pool_min: 5,
            redis_pool_max: 20,

            jwt_secret: "${OPENHACK_SECRET}".into(),
            jwt_expiry: "15m".into(),
            refresh_token_expiry: "7d".into(),
            jwt_issuer: "openhack".into(),
            password_min_length: 12,
            password_require_uppercase: true,
            password_require_lowercase: true,
            password_require_numbers: true,
            password_require_special: true,
            session_timeout: "24h".into(),
            session_cookie_secure: true,
            session_cookie_httponly: true,
            session_cookie_samesite: "strict".into(),
            auth_rate_limit_enabled: true,
            auth_rate_limit_max: 100,
            auth_rate_limit_window_ms: 60000,

            github_enabled: true,
            github_client_id: "your-github-client-id".into(),
            github_client_secret: "your-github-client-secret".into(),
            google_enabled: false,
            google_client_id: "your-google-client-id.apps.googleusercontent.com".into(),
            google_client_secret: "your-google-client-secret".into(),
            discord_oauth_enabled: true,
            discord_client_id: "your-discord-client-id".into(),
            discord_client_secret: "your-discord-client-secret".into(),

            mfa_totp_enabled: true,
            mfa_totp_issuer: "OpenHack".into(),
            mfa_sms_enabled: false,
            twilio_account_sid: "your-twilio-account-sid".into(),
            twilio_auth_token: "your-twilio-auth-token".into(),
            twilio_phone_number: "+1234567890".into(),

            mail_provider: "smtp".into(),
            mail_from: "noreply@${OPENHACK_DOMAIN}".into(),
            mail_from_name: "${OPENHACK_NAME}".into(),
            smtp_host: "mail.example.com".into(),
            smtp_port: 587,
            smtp_user: String::new(),
            smtp_password: String::new(),
            smtp_tls: true,
            smtp_reject_unauthorized: true,
            sendgrid_api_key: String::new(),
            aws_ses_region: "us-east-1".into(),
            mail_rate_limit_enabled: true,
            mail_rate_limit_max_per_hour: 1000,

            discord_bot_token: String::new(),
            discord_guild_id: String::new(),
            slack_bot_token: String::new(),
            slack_signing_secret: String::new(),
            webhook_secret: "your-webhook-hmac-secret".into(),
            webhook_retry_count: 3,
            webhook_timeout_ms: 30000,

            ai_provider: "openai".into(),
            ai_enabled: true,
            openai_api_key: "sk-...".into(),
            openai_model: "gpt-4-turbo".into(),
            openai_max_tokens: 1024,
            openai_temperature: "0.7".into(),
            anthropic_api_key: "sk-ant-...".into(),
            anthropic_model: "claude-3-opus-20240229".into(),
            ollama_enabled: false,
            ollama_base_url: "http://localhost:11434".into(),
            ollama_model: "llama2:7b".into(),
            azure_openai_endpoint: String::new(),
            azure_openai_api_key: String::new(),
            azure_openai_deployment: String::new(),
            azure_openai_api_version: "2024-02-15-preview".into(),
            ai_feature_chat: true,
            ai_feature_idea_generator: true,
            ai_feature_team_matcher: true,
            ai_feature_code_review: false,
            ai_feature_organizer_insights: true,
            ai_rag_enabled: true,
            ai_rag_top_k: 5,
            ai_embedding_model: "text-embedding-3-small".into(),

            storage_provider: "local".into(),
            local_storage_path: "/var/lib/openhack/media".into(),
            local_storage_max_size: 52_428_800,
            minio_endpoint: "http://minio:9000".into(),
            minio_bucket: "openhack-media".into(),
            minio_access_key: "minioadmin".into(),
            minio_secret_key: "minioadmin".into(),
            minio_use_ssl: false,
            aws_region: "us-east-1".into(),
            aws_s3_bucket: "openhack-media".into(),
            aws_access_key_id: String::new(),
            aws_secret_access_key: String::new(),
            s3_use_path_style: false,
            gcs_bucket: String::new(),
            gcs_credentials: String::new(),
            azure_storage_account: String::new(),
            azure_storage_key: String::new(),
            azure_container: String::new(),
            max_file_size: 52_428_800,
            max_image_size: 10_485_760,
            max_video_size: 524_288_000,

            next_public_api_url: "/api".into(),
            next_public_ws_url: "ws://${OPENHACK_DOMAIN}".into(),
            next_public_app_name: "${OPENHACK_NAME}".into(),
            feature_teams: true,
            feature_projects: true,
            feature_judging: true,
            feature_voting: true,
            feature_sponsors: true,

            gateway_url: "http://gateway-svc:8000".into(),
            gateway_rate_limit_auth: 100,
            gateway_rate_limit_core: 300,
            gateway_rate_limit_judging: 100,
            gateway_rate_limit_mail: 50,
            gateway_rate_limit_notify: 50,
            gateway_rate_limit_ai: 30,
            cors_enabled: true,
            cors_origins: "http://localhost:3000,http://localhost:3001".into(),
            cors_credentials: true,
            cors_max_age: 3600,

            prometheus_enabled: false,
            prometheus_port: 9090,
            grafana_enabled: false,
            grafana_admin_password: String::new(),
            alertmanager_enabled: false,
            otel_enabled: false,
            otel_endpoint: "http://jaeger:4317".into(),
            otel_service_name: "openhack".into(),

            https_enabled: true,
            tls_cert_path: "/etc/ssl/certs/openhack.crt".into(),
            tls_key_path: "/etc/ssl/private/openhack.key".into(),
            security_hsts_enabled: true,
            security_hsts_max_age: 31_536_000,
            security_frame_options: "DENY".into(),
            security_content_type_nosniff: true,
            csp_enabled: true,
            csp_default_src: "'self'".into(),
            csp_script_src: "'self' 'unsafe-inline'".into(),
            csp_style_src: "'self' 'unsafe-inline'".into(),

            backup_schedule: "0 2 * * *".into(),
            backup_retention_daily: 7,
            backup_retention_weekly: 4,
            backup_retention_monthly: 12,
            backup_destination: "/backups".into(),
            backup_s3_bucket: "openhack-backups".into(),
            backup_s3_region: "us-east-1".into(),

            multi_tenancy_enabled: false,
            tenancy_model: "schema".into(),
            default_tenant: "default".into(),

            dev_hot_reload: true,
            dev_watch_files: true,
            dev_enable_profiling: false,
            dev_enable_query_logging: true,
            dev_enable_swagger: true,
            dev_seed_data: false,
            dev_test_users: false,

            kubernetes_namespace: "openhack".into(),
            docker_compose_project_name: "openhack".into(),
            health_check_interval: 30,
            health_check_timeout: 10,

            db_instance_class: "db.t4g.micro".into(),
            db_serverless_min_capacity: 0.5,
            db_serverless_max_capacity: 2.0,
            lambda_memory_size: 256,
            lambda_timeout: 30,
            ecr_image_tag: "latest".into(),
            domain_name: String::new(),
            certificate_arn: String::new(),
            vpc_cidr: "10.0.0.0/16".into(),

            gcp_project_id: "your-gcp-project".into(),
            gcp_region: "us-central1".into(),

            azure_location: "eastus".into(),
            azure_resource_group: "openhack-rg".into(),

            deploy_steps: Vec::new(),
            deploy_error: None,
            aws_api_gateway_id: None,

            deploy_state: None,
            migrate_selected: 0,

            screen: Screen::Welcome,
            selected: 0,
            editing: 0,
            input_buffer: String::new(),
            editing_text: false,
        }
    }

    pub fn new() -> Self {
        Self::default_dev()
    }

    pub fn next_screen(&mut self) {
        self.selected = 0;
        self.screen = match self.screen {
            Screen::Welcome => Screen::DeployTarget,
            Screen::DeployTarget => Screen::Services,
            Screen::Services => Screen::Database,
            Screen::Database => {
                if self.is_service_enabled(ServiceName::Media) {
                    Screen::Storage
                } else {
                    Screen::Email
                }
            }
            Screen::Storage => Screen::Email,
            Screen::Email => {
                if self.is_service_enabled(ServiceName::Mail) {
                    Screen::Secrets
                } else {
                    Screen::Review
                }
            }
            Screen::Secrets => Screen::CostEstimate,
            Screen::CostEstimate => Screen::Review,
            Screen::Review => Screen::Progress,
            Screen::Progress => Screen::Done,
            Screen::Done => Screen::Done,
            Screen::Migrate => Screen::Migrate,
        };
    }

    pub fn prev_screen(&mut self) {
        self.selected = 0;
        self.screen = match self.screen {
            Screen::Welcome => Screen::Welcome,
            Screen::DeployTarget => Screen::Welcome,
            Screen::Services => Screen::DeployTarget,
            Screen::Database => Screen::Services,
            Screen::Storage => Screen::Database,
            Screen::Email => {
                if self.is_service_enabled(ServiceName::Media) {
                    Screen::Storage
                } else {
                    Screen::Database
                }
            }
            Screen::Secrets => {
                if self.is_service_enabled(ServiceName::Mail) {
                    Screen::Email
                } else {
                    Screen::Storage
                }
            }
            Screen::CostEstimate => Screen::Secrets,
            Screen::Review => Screen::CostEstimate,
            Screen::Progress => Screen::Review,
            Screen::Done => Screen::Done,
            Screen::Migrate => Screen::Done,
        };
    }

    pub fn screen_index(&self) -> usize {
        match self.screen {
            Screen::Welcome => 1,
            Screen::DeployTarget => 2,
            Screen::Services => 3,
            Screen::Database => 4,
            Screen::Storage => 5,
            Screen::Email => 6,
            Screen::Secrets => 7,
            Screen::CostEstimate => 8,
            Screen::Review => 9,
            Screen::Progress => 10,
            Screen::Done => 11,
            Screen::Migrate => 0,
        }
    }

    pub fn total_screen_count() -> usize {
        11
    }

    pub fn toggle_service(&mut self, idx: usize) {
        let all = ServiceName::all();
        if let Some(svc) = all.get(idx) {
            if svc.is_core_service() {
                return;
            }
            if self.is_service_enabled(*svc) {
                self.enabled_services.retain(|s| s != svc);
            } else {
                self.enabled_services.push(*svc);
            }
        }
    }

    pub fn total_memory_mb(&self) -> u32 {
        let mems: [(ServiceName, u32); 11] = [
            (ServiceName::Auth, 64),
            (ServiceName::Core, 64),
            (ServiceName::Gateway, 48),
            (ServiceName::Judging, 96),
            (ServiceName::Leaderboard, 48),
            (ServiceName::Mail, 64),
            (ServiceName::Notify, 64),
            (ServiceName::Ai, 64),
            (ServiceName::Analytics, 64),
            (ServiceName::Sponsors, 64),
            (ServiceName::Media, 96),
        ];
        mems.iter()
            .filter(|(svc, _)| self.is_service_enabled(*svc))
            .map(|(_, m)| m)
            .sum()
    }

    pub fn generate_jwt_secret() -> String {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        let mut s = String::with_capacity(64);
        for _ in 0..32 {
            use std::fmt::Write;
            let _ = write!(s, "{:02x}", rng.gen::<u8>());
        }
        s
    }

    pub fn generate_db_password() -> String {
        use rand::Rng;
        const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";
        let mut rng = rand::thread_rng();
        (0..24)
            .map(|_| CHARSET[rng.gen_range(0..CHARSET.len())] as char)
            .collect()
    }

    pub fn field_count(&self) -> usize {
        match self.screen {
            Screen::Database => 2,
            Screen::Secrets => 6,
            _ => 1,
        }
    }

    pub fn is_service_enabled(&self, svc: ServiceName) -> bool {
        self.enabled_services.contains(&svc)
    }

    pub fn uses_redis(&self) -> bool {
        !self.redis_url.is_empty()
    }

    pub fn uses_minio(&self) -> bool {
        self.storage_provider == "minio"
    }

    pub fn uses_s3(&self) -> bool {
        self.storage_provider == "s3"
    }

    pub fn enter_migrate_screen(&mut self, project_dir: &std::path::Path) {
        let state_path = DeployState::path_for_project(project_dir);
        if state_path.exists() {
            self.deploy_state = DeployState::load(&state_path).ok();
        }
        if self.deploy_state.is_none() {
            self.deploy_state = Some(DeployState::from_app(self));
        }
        self.migrate_selected = 0;
        self.selected = 0;
        self.screen = Screen::Migrate;
    }

    pub fn migrate_service_keys(&self) -> Vec<&str> {
        self.deploy_state
            .as_ref()
            .map(|ds| ds.service_keys_ordered())
            .unwrap_or_default()
    }

    pub fn database_url(&self) -> String {
        format!(
            "postgresql://{}:{}@{}:{}/{}",
            self.db_username, self.db_password, self.db_host, self.db_port, self.db_name
        )
    }

    /// Estimate monthly cost in USD for the current deploy target and enabled services.
    ///
    /// Returns a tuple of (`monthly_cost`, `breakdown_lines`) where each line is
    /// (`item_name`, `estimated_cost_usd`).
    #[must_use]
    pub fn estimate_monthly_cost(&self) -> (f64, Vec<(&'static str, f64)>) {
        let svc_count = self.enabled_services.len() as f64;
        let mut lines = Vec::new();

        match self.deploy_target {
            DeployTarget::DockerCompose => {
                let mem_gb = f64::from(self.total_memory_mb()) / 1024.0;
                let vps = 5.0 + (mem_gb * 2.0).max(0.0);
                lines.push(("VPS (2GB+ RAM)", vps));
                lines.push(("Domain (optional)", 1.0));
                (vps + 1.0, lines)
            }
            DeployTarget::AwsLambda => {
                let lambda_requests = 1_000_000.0;
                let lambda_gb_sec = svc_count * 100_000.0;
                let lambda_cost = (lambda_requests / 1_000_000.0) * 0.20
                    + (lambda_gb_sec / 1_000_000.0) * 0.0000166667 * 256.0;
                lines.push(("Lambda (1M req, 256MB)", lambda_cost));
                lines.push(("API Gateway (1M req)", 1.00));
                lines.push(("Aurora Serverless v2 (0.5-2 ACU)", 12.60));
                lines.push(("ElastiCache Redis (t4g.micro)", 5.00));
                lines.push(("S3 storage (10GB)", 0.23));
                lines.push(("CloudWatch logs", 0.50));
                let total: f64 = lines.iter().map(|(_, c)| c).sum();
                (total, lines)
            }
            DeployTarget::GcpCloudRun => {
                lines.push(("Cloud Run (2 vCPU, 512Mi, 1M req)", 2.50));
                lines.push(("Cloud SQL (db-f1-micro)", 7.50));
                lines.push(("Memorystore Redis (1GB)", 15.00));
                lines.push(("Cloud Storage (10GB)", 0.20));
                lines.push(("Cloud Logging", 0.50));
                let total: f64 = lines.iter().map(|(_, c)| c).sum();
                (total, lines)
            }
            DeployTarget::AzureContainerApps => {
                lines.push(("Container Apps (0.5 vCPU, 1GB, 1M req)", 4.00));
                lines.push(("Azure DB PostgreSQL (Burstable B1ms)", 6.70));
                lines.push(("Azure Cache Redis (C0)", 16.00));
                lines.push(("Blob Storage (10GB)", 0.10));
                lines.push(("Log Analytics", 1.00));
                let total: f64 = lines.iter().map(|(_, c)| c).sum();
                (total, lines)
            }
            DeployTarget::Kubernetes => {
                lines.push(("K8s cluster (3x n2d-standard-2)", 150.00));
                lines.push(("Cloud SQL (db-custom-1-3840)", 30.00));
                lines.push(("Redis (2GB)", 35.00));
                lines.push(("Storage + networking", 5.00));
                lines.push(("Monitoring (Prometheus)", 0.0));
                let total: f64 = lines.iter().map(|(_, c)| c).sum();
                (total, lines)
            }
        }
    }
}
