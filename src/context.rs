use crate::common::XRF_ENV_KEY;
use lazy_static::lazy_static;
use std::sync::OnceLock;
use uuid::Uuid;

#[derive(Clone)]
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
        if let Environment::Dev = self.clone() {
            return true;
        };
        false
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

pub struct AppContext {
    pub server_id: String,
    pub environment: Environment,
}

lazy_static! {
    pub static ref APP_CONTEXT_CONFIG: OnceLock<AppContext> = OnceLock::new();
}

impl AppContext {
    pub fn load() -> &'static AppContext {
        APP_CONTEXT_CONFIG.get_or_init(|| {
            // Determine the environment.  This is where your loading logic goes.
            // For this example, we'll use an environment variable.
            let env: Environment = std::env::var(XRF_ENV_KEY)
                .unwrap_or_else(|_| "dev".into())
                .try_into()
                .expect("XRF_ENV env variable is not accepted environment");

            AppContext { environment: env.clone(), server_id: Uuid::new_v4().to_string() }
        })
    }
}
