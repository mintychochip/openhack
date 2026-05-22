use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Audit log entry for tracking security-relevant actions.
///
/// # Expected Behavior
///
/// Each audit log entry captures:
/// - Timestamp (UTC)
/// - Action performed (e.g., "user.registered", "score.submitted")
/// - Actor (user ID, or "anonymous" / "system")
/// - Resource type and ID
/// - Outcome (success/failure)
/// - IP address
/// - Additional context (JSON)
///
/// Entries are serialized as JSON and written to stdout for collection by Loki/Fluentd.
///
/// # Side Effects
///
/// - Writes JSON line to stdout when `log_audit()` is called
/// - No other I/O or state mutation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditLogEntry {
    /// ISO 8601 timestamp in UTC
    pub timestamp: DateTime<Utc>,
    
    /// Action performed (e.g., "user.registered", "auth.login.success")
    pub action: String,
    
    /// User ID who performed the action, or special values:
    /// - "anonymous" for unauthenticated requests
    /// - "system" for automated processes
    pub actor: String,
    
    /// Resource type (e.g., "user", "project", "score")
    pub resource_type: Option<String>,
    
    /// Resource ID (UUID or other identifier)
    pub resource_id: Option<String>,
    
    /// Outcome: "success" or "failure"
    pub outcome: String,
    
    /// Client IP address
    pub ip_address: Option<String>,
    
    /// Additional context as JSON-serializable value
    pub details: Option<serde_json::Value>,
}

impl AuditLogEntry {
    #[must_use]
    pub fn new(action: &str, actor: &str, outcome: &str) -> Self {
        Self {
            timestamp: Utc::now(),
            action: action.to_string(),
            actor: actor.to_string(),
            resource_type: None,
            resource_id: None,
            outcome: outcome.to_string(),
            ip_address: None,
            details: None,
        }
    }

    #[must_use]
    pub fn with_resource_type(mut self, resource_type: &str) -> Self {
        self.resource_type = Some(resource_type.to_string());
        self
    }

    #[must_use]
    pub fn with_resource_id(mut self, resource_id: &str) -> Self {
        self.resource_id = Some(resource_id.to_string());
        self
    }

    #[must_use]
    pub fn with_ip_address(mut self, ip_address: &str) -> Self {
        self.ip_address = Some(ip_address.to_string());
        self
    }

    #[must_use]
    pub fn with_details(mut self, details: serde_json::Value) -> Self {
        self.details = Some(details);
        self
    }
}

/// Log an audit entry to stdout as JSON.
///
/// # Expected Behavior
///
/// Serializes the `AuditLogEntry` as a single-line JSON object and writes
/// to stdout. This format is compatible with Loki/Fluentd/CloudWatch Logs
/// collection agents.
///
/// # Errors
///
/// None. Serialization failures are silently ignored to prevent audit
/// logging from breaking application functionality.
///
/// # Side Effects
///
/// - Writes one JSON line to stdout
/// - May block briefly if stdout buffer is full
pub fn log_audit(entry: &AuditLogEntry) {
    match serde_json::to_string(entry) {
        Ok(json) => println!("{json}"),
        Err(e) => eprintln!("Failed to serialize audit log entry: {e}"),
    }
}

/// Macro for convenient audit logging at the call site.
///
/// # Usage
///
/// ```rust
/// audit_log!(
///     "user.registered",
///     actor = &user_id.to_string(),
///     resource_type = "user",
///     resource_id = &user_id.to_string(),
///     outcome = "success",
///     ip = &client_ip,
///     details = serde_json::json!({"email": &user_email})
/// );
/// ```
#[macro_export]
macro_rules! audit_log {
    ($action:expr, actor = $actor:expr, resource_type = $resource_type:expr, resource_id = $resource_id:expr, outcome = $outcome:expr, ip = $ip:expr, details = $details:expr) => {
        $crate::audit::log_audit(
            &$crate::audit::AuditLogEntry::new($action, $actor, $outcome)
                .with_resource_type($resource_type)
                .with_resource_id($resource_id)
                .with_ip_address($ip)
                .with_details($details)
        )
    };
    ($action:expr, actor = $actor:expr, resource_type = $resource_type:expr, resource_id = $resource_id:expr, outcome = $outcome:expr) => {
        $crate::audit::log_audit(
            &$crate::audit::AuditLogEntry::new($action, $actor, $outcome)
                .with_resource_type($resource_type)
                .with_resource_id($resource_id)
        )
    };
    ($action:expr, actor = $actor:expr, outcome = $outcome:expr) => {
        $crate::audit::log_audit(
            &$crate::audit::AuditLogEntry::new($action, $actor, $outcome)
        )
    };
}

/// Common audit action names for consistency across services.
pub mod actions {
    // Authentication
    pub const AUTH_REGISTER: &str = "auth.register";
    pub const AUTH_LOGIN_SUCCESS: &str = "auth.login.success";
    pub const AUTH_LOGIN_FAILURE: &str = "auth.login.failure";
    pub const AUTH_LOGOUT: &str = "auth.logout";
    pub const AUTH_PASSWORD_RESET: &str = "auth.password.reset";
    pub const AUTH_MFA_ENABLE: &str = "auth.mfa.enable";
    pub const AUTH_MFA_DISABLE: &str = "auth.mfa.disable";
    pub const AUTH_EMAIL_VERIFY: &str = "auth.email.verify";
    
    // User management
    pub const USER_UPDATE: &str = "user.update";
    pub const USER_DELETE: &str = "user.delete";
    pub const USER_ROLE_CHANGE: &str = "user.role.change";
    
    // Core service
    pub const TEAM_CREATE: &str = "team.create";
    pub const TEAM_UPDATE: &str = "team.update";
    pub const TEAM_DELETE: &str = "team.delete";
    pub const PROJECT_SUBMIT: &str = "project.submit";
    pub const PROJECT_UPDATE: &str = "project.update";
    pub const EVENT_RSVP: &str = "event.rsvp";
    
    // Judging service
    pub const SCORE_SUBMIT: &str = "score.submit";
    pub const SCORE_UPDATE: &str = "score.update";
    pub const RUBRIC_CREATE: &str = "rubric.create";
    pub const RUBRIC_UPDATE: &str = "rubric.update";
    pub const RUBRIC_DELETE: &str = "rubric.delete";
    pub const PHASE_OPEN: &str = "phase.open";
    pub const PHASE_CLOSE: &str = "phase.close";
    pub const PHASE_FINALIZE: &str = "phase.finalize";
    
    // Leaderboard
    pub const VOTE_CAST: &str = "vote.cast";
    pub const LEADERBOARD_RECALCULATE: &str = "leaderboard.recalculate";
    
    // Mail service
    pub const EMAIL_SEND: &str = "email.send";
    pub const EMAIL_BROADCAST: &str = "email.broadcast";
    pub const TEMPLATE_CREATE: &str = "template.create";
    
    // Media service
    pub const FILE_UPLOAD: &str = "file.upload";
    pub const FILE_DELETE: &str = "file.delete";
}
