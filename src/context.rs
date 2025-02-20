use crate::constant::{INVALID_SERVER_ID, XRF_ENV_KEY};
use std::sync::{Mutex, OnceLock};
use uuid::Uuid;

#[derive(Debug, Clone)]
pub enum ContextError {
    InvalidEnvironment(String),
    MissingEnvironment(String),
    ConflictingEnvironmentVariables,
}

impl std::error::Error for ContextError {}

impl std::fmt::Display for ContextError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            ContextError::ConflictingEnvironmentVariables => write!(f, "conflicting environment variables"),
            ContextError::MissingEnvironment(err) => write!(f, "missing environment variable: {}", err),
            ContextError::InvalidEnvironment(err) => write!(f, "invalid environment variable: {}", err),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Environment {
    Dev,
    Live,
    Staging,
    Production,
}

impl Environment {
    pub fn as_str(&self) -> &'static str {
        match self {
            Environment::Dev => "dev",
            Environment::Live => "live",
            Environment::Staging => "stg",
            Environment::Production => "prod",
        }
    }

    fn is_local(&self) -> bool {
        *self == Environment::Dev
    }

    fn is_not_local(&self) -> bool {
        !self.is_local()
    }
}

impl TryFrom<String> for Environment {
    type Error = String;
    fn try_from(env: String) -> Result<Self, Self::Error> {
        match env.to_lowercase().as_str() {
            "live" => Ok(Environment::Live),
            "stg" => Ok(Environment::Staging),
            "prod" => Ok(Environment::Production),
            "dev" | "local" => Ok(Environment::Dev),
            _ => Err(format!("Unknown environment: {}", env)),
        }
    }
}

#[derive(Debug, Clone)]
pub struct AppContext {
    pub server_id: String,
    pub environment: Environment,
}

static APP_CONTEXT_CONFIG: OnceLock<AppContext> = OnceLock::new();
static LOAD_APP_CONTEXT_ERROR: Mutex<Option<ContextError>> = Mutex::new(None);

impl AppContext {
    // singleton, load application one
    pub fn load() -> Result<&'static AppContext, ContextError> {
        // 1. Check if an error occurred during initialization.
        let error_guard = LOAD_APP_CONTEXT_ERROR.lock().unwrap();
        if let Some(error) = error_guard.as_ref() {
            return Err(error.clone()) // Return the stored error
        };
        drop(error_guard); // Release the lock early

        // Use get_or_init to initialize AppContext if it hasn't been initialized yet.
        let app_context = APP_CONTEXT_CONFIG.get_or_init(|| {
            // Perform initialization, capturing any errors.

            // Use an immediately invoked closure for error handling
            let result = (|| {
                let env_str = std::env::var(XRF_ENV_KEY)
                    .map_err(|_| ContextError::MissingEnvironment(format!("{} environment variable not set", XRF_ENV_KEY)))?;

                let env: Environment = env_str.clone().try_into()
                    .map_err(|_| ContextError::InvalidEnvironment(format!("invalid '{}' environment", XRF_ENV_KEY)))?;

                let database_env = std::env::var("DATABASE_ENV")
                    .map_err(|_| ContextError::MissingEnvironment("DATABASE_ENV environment variable not set".to_string()))?;

                // every database environment must contain an environment's name
                if !env.is_local() && (database_env.is_empty() || !database_env.contains(&env_str)) {
                    return Err(ContextError::ConflictingEnvironmentVariables);
                }

                Ok(AppContext {
                    environment: env,
                    server_id: Uuid::new_v4().to_string(),
                })
            })();

            result.unwrap_or_else(|err| {
                *LOAD_APP_CONTEXT_ERROR.lock().unwrap() = Some(err);
                // This is important:  We *must* return a valid AppContext
                // from get_or_init, even if an error occurred. We provide a
                // *dummy* AppContext, as load() will return the error anyway
                // Because we checked for it before.
                AppContext {
                    environment: Environment::Dev,
                    server_id: INVALID_SERVER_ID.to_string(),
                }
            })
        });

        // 5. Check if an error occurred during initialization.
        let error_guard = LOAD_APP_CONTEXT_ERROR.lock().unwrap();
        if let Some(error) = error_guard.as_ref() {
            return Err(error.clone()) // Return the stored error
        }
        // 6. return app context
        Ok(app_context)
    }
}
