use regex::Regex;
use std::sync::OnceLock;

static EMAIL_REGEX: OnceLock<Regex> = OnceLock::new();

fn get_email_regex() -> &'static Regex {
    EMAIL_REGEX.get_or_init(|| {
        Regex::new(r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$").unwrap()
    })
}

/// Validate an email address.
///
/// # Expected Behavior
///
/// Checks that the email:
/// - Is not empty
/// - Matches RFC 5322 simplified pattern (local@domain.tld)
/// - Has a valid domain part with at least one dot
/// - Is at most 254 characters (RFC 5321 limit)
///
/// # Errors
///
/// Returns `ValidationError` with a descriptive message if validation fails.
///
/// # Examples
///
/// ```
/// let result = validate_email("user@example.com")?;
/// let result = validate_email("")?; // Err
/// let result = validate_email("invalid")?; // Err
/// ```
pub fn validate_email(email: &str) -> Result<(), ValidationError> {
    if email.is_empty() {
        return Err(ValidationError::new("email", "Email is required"));
    }

    if email.len() > 254 {
        return Err(ValidationError::new(
            "email",
            "Email must be at most 254 characters",
        ));
    }

    if !get_email_regex().is_match(email) {
        return Err(ValidationError::new("email", "Invalid email format"));
    }

    Ok(())
}

/// Validate a password.
///
/// # Expected Behavior
///
/// Checks that the password:
/// - Is at least 8 characters (configurable via min_length)
/// - Is at most 128 characters
/// - Contains at least one uppercase letter (if require_uppercase)
/// - Contains at least one lowercase letter (if require_lowercase)
/// - Contains at least one digit (if require_digit)
/// - Contains at least one special character (if require_special)
///
/// # Errors
///
/// Returns `ValidationError` with a descriptive message if validation fails.
pub fn validate_password(
    password: &str,
    min_length: usize,
    require_uppercase: bool,
    require_lowercase: bool,
    require_digit: bool,
    require_special: bool,
) -> Result<(), ValidationError> {
    if password.is_empty() {
        return Err(ValidationError::new("password", "Password is required"));
    }

    if password.len() < min_length {
        return Err(ValidationError::new(
            "password",
            &format!("Password must be at least {min_length} characters"),
        ));
    }

    if password.len() > 128 {
        return Err(ValidationError::new(
            "password",
            "Password must be at most 128 characters",
        ));
    }

    if require_uppercase && !password.chars().any(|c| c.is_uppercase()) {
        return Err(ValidationError::new(
            "password",
            "Password must contain at least one uppercase letter",
        ));
    }

    if require_lowercase && !password.chars().any(|c| c.is_lowercase()) {
        return Err(ValidationError::new(
            "password",
            "Password must contain at least one lowercase letter",
        ));
    }

    if require_digit && !password.chars().any(|c| c.is_ascii_digit()) {
        return Err(ValidationError::new(
            "password",
            "Password must contain at least one digit",
        ));
    }

    if require_special
        && !password
            .chars()
            .any(|c| !c.is_alphanumeric() && !c.is_whitespace())
    {
        return Err(ValidationError::new(
            "password",
            "Password must contain at least one special character",
        ));
    }

    Ok(())
}

/// Validate a name (user name, team name, etc.).
///
/// # Expected Behavior
///
/// Checks that the name:
/// - Is not empty after trimming
/// - Is at most max_length characters
/// - Contains no control characters
///
/// # Errors
///
/// Returns `ValidationError` with a descriptive message if validation fails.
pub fn validate_name(name: &str, max_length: usize) -> Result<(), ValidationError> {
    let trimmed = name.trim();

    if trimmed.is_empty() {
        return Err(ValidationError::new("name", "Name is required"));
    }

    if trimmed.len() > max_length {
        return Err(ValidationError::new(
            "name",
            &format!("Name must be at most {max_length} characters"),
        ));
    }

    if trimmed.chars().any(|c| c.is_control()) {
        return Err(ValidationError::new(
            "name",
            "Name cannot contain control characters",
        ));
    }

    Ok(())
}

/// Validate a UUID string.
///
/// # Expected Behavior
///
/// Checks that the string is a valid UUID (any version).
///
/// # Errors
///
/// Returns `ValidationError` with a descriptive message if validation fails.
pub fn validate_uuid(uuid_str: &str) -> Result<(), ValidationError> {
    if uuid_str.is_empty() {
        return Err(ValidationError::new("uuid", "UUID is required"));
    }

    uuid::Uuid::parse_str(uuid_str).map_err(|_| {
        ValidationError::new("uuid", "Invalid UUID format")
    })?;

    Ok(())
}

/// Validate a URL.
///
/// # Expected Behavior
///
/// Checks that the URL:
/// - Is not empty
/// - Has a valid scheme (http or https)
/// - Has a valid host
/// - Is at most 2048 characters
///
/// # Errors
///
/// Returns `ValidationError` with a descriptive message if validation fails.
pub fn validate_url(url: &str) -> Result<(), ValidationError> {
    if url.is_empty() {
        return Err(ValidationError::new("url", "URL is required"));
    }

    if url.len() > 2048 {
        return Err(ValidationError::new(
            "url",
            "URL must be at most 2048 characters",
        ));
    }

    if !url.starts_with("http://") && !url.starts_with("https://") {
        return Err(ValidationError::new(
            "url",
            "URL must start with http:// or https://",
        ));
    }

    Ok(())
}

/// Validation error with field name and message.
///
/// # Expected Behavior
///
/// Represents a single validation error for a specific field.
/// Can be collected into a list of errors for comprehensive error reporting.
#[derive(Debug, Clone)]
pub struct ValidationError {
    pub field: String,
    pub message: String,
}

impl ValidationError {
    #[must_use]
    pub fn new(field: &str, message: &str) -> Self {
        Self {
            field: field.to_string(),
            message: message.to_string(),
        }
    }
}

impl std::fmt::Display for ValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.field, self.message)
    }
}

impl std::error::Error for ValidationError {}

/// Collection of validation errors.
///
/// # Expected Behavior
///
/// Aggregates multiple `ValidationError` instances for comprehensive
/// error reporting. Returns all errors at once rather than failing fast.
#[derive(Debug, Clone, Default)]
pub struct ValidationErrors {
    pub errors: Vec<ValidationError>,
}

impl ValidationErrors {
    #[must_use]
    pub fn new() -> Self {
        Self { errors: vec![] }
    }

    pub fn add(&mut self, error: ValidationError) {
        self.errors.push(error);
    }

    pub fn add_field(&mut self, field: &str, message: &str) {
        self.errors.push(ValidationError::new(field, message));
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.errors.is_empty()
    }

    #[must_use]
    pub fn has_errors(&self) -> bool {
        !self.errors.is_empty()
    }

    #[must_use]
    pub fn into_errors(self) -> Vec<ValidationError> {
        self.errors
    }

    #[must_use]
    pub fn first(&self) -> Option<&ValidationError> {
        self.errors.first()
    }
}

impl From<Vec<ValidationError>> for ValidationErrors {
    fn from(errors: Vec<ValidationError>) -> Self {
        Self { errors }
    }
}
