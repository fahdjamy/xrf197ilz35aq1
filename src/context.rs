use crate::constant::{INVALID_SERVER_ID, XRF_1_POSTGRES_DB_URL_ENV_KEY, XRF_ENV_KEY};
use std::fmt::{Display, Formatter};
use std::sync::{Mutex, OnceLock};
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ContextError {
    InvalidEnvironment(String),
    MissingEnvironment(String),
    ConflictingEnvironmentVariables,
}

impl std::error::Error for ContextError {}

impl Display for ContextError {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
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

    pub fn is_local(&self) -> bool {
        *self == Environment::Dev
    }

    pub fn is_not_local(&self) -> bool {
        !self.is_local()
    }
}

impl Display for Environment {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
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
    pub fn get_or_load() -> Result<&'static AppContext, ContextError> {
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

                let database_env = std::env::var(XRF_1_POSTGRES_DB_URL_ENV_KEY)
                    .map_err(|_| ContextError::MissingEnvironment(format!("{} environment variable not set", XRF_1_POSTGRES_DB_URL_ENV_KEY)))?;

                // every database environment must contain an environment's name
                if !env.is_local() && (database_env.is_empty() || !database_env.contains(&env_str)) {
                    return Err(ContextError::ConflictingEnvironmentVariables);
                }

                let server_id = Uuid::new_v4().to_string();
                Ok(AppContext { environment: env, server_id })
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

    pub fn environment() -> Option<Environment> {
        if let Some(context) = APP_CONTEXT_CONFIG.get() {
            return Some(context.environment.clone())
        }
        None
    }

    // Reset function, using get_mut (unstable).
    #[cfg(test)]
    fn reset() {
        *LOAD_APP_CONTEXT_ERROR.lock().unwrap() = None; //clear error
        std::env::remove_var(XRF_ENV_KEY); //remove env var
        std::env::remove_var("DATABASE_URL"); //remove env var
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_app_context_success() {
        AppContext::reset(); // Reset before the test
        assert_eq!(AppContext::environment(), None);

        std::env::set_var(XRF_ENV_KEY, "live");
        std::env::set_var(XRF_1_POSTGRES_DB_URL_ENV_KEY, "xrf_live_pg_db");

        let result = AppContext::get_or_load();
        assert!(result.is_ok());
        let context = result.unwrap();
        assert!(!context.server_id.is_empty());
        assert_eq!(context.environment, Environment::Live);

        // Test that subsequent calls return the same instance.
        let result2 = AppContext::get_or_load();
        assert!(result2.is_ok());
        let context2 = result2.unwrap();
        assert!(std::ptr::eq(context, context2)); // Check for pointer equality
    }

    // #[test]
    // fn test_load_missing_xrf_env() {
    //     AppContext::reset();
    //     // Don't set XRF_ENV_KEY
    //     std::env::set_var(XRF_1_POSTGRES_DB_URL_ENV_KEY, "any_db");
    //
    //     let result = AppContext::get_or_load();
    //     assert!(result.is_err());
    //     assert_eq!(
    //         result.unwrap_err(),
    //         ContextError::MissingEnvironment(format!("{} environment variable not set", XRF_ENV_KEY))
    //     );
    //     // Check that subsequent calls return the *same* error.
    //     let result2 = AppContext::get_or_load();
    //     assert!(result2.is_err());
    //     assert_eq!(
    //         result2.unwrap_err(),
    //         ContextError::MissingEnvironment(format!("{} environment variable not set", XRF_ENV_KEY))
    //     );
    // }
}
