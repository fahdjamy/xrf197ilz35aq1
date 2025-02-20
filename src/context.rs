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
