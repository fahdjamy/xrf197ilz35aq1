use serde::Deserialize;
use serde_aux::field_attributes::deserialize_number_from_string;

#[derive(Deserialize)]
pub struct App {
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub port: u16,
    pub host: String,
}

#[derive(serde::Deserialize)]
pub struct Config {
    pub app: App,
}

pub fn load_config() -> Result<Config, config::ConfigError> {
    let base_path = std::env::current_dir().expect("Could not determine current directory");
    let config_path = base_path.join("config");

    // load app environment. default to dev (local/dev) if no env is specified
    let env: Environment = std::env::var("XRF_ENV")
        .unwrap_or_else(|_| "dev".into())
        .try_into()
        .expect("XRF_ENV env variable is not accepted environment");

    // load config filename for set XRF_ENV environment
    let env_config_file = format!("{}.yml", env.as_str());

    // Initialise the configurations
    let config = config::Config::builder()
        // Add base configuration values from a file named `app.yaml`.
        .add_source(config::File::from(config_path.join("app.yaml")))
        // Add configuration values from the environment specific file
        .add_source(config::File::from(config_path.join(env_config_file)))
        // Add configurations set from the exported environment
        .add_source(
            config::Environment::with_prefix("XRF")
                .prefix_separator("_")
                .separator("-"),
        )
        .build()?;

    // Try converting the configuration values into our Config type
    config.try_deserialize::<Config>()
}

enum Environment {
    Dev,
    Live,
    Staging,
    Production,
}

impl Environment {
    fn as_str(&self) -> &'static str {
        match self {
            Environment::Dev => "dev",
            Environment::Live => "live",
            Environment::Staging => "stg",
            Environment::Production => "prod",
        }
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
