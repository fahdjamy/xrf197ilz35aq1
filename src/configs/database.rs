use secrecy::{ExposeSecret, SecretString};
use serde::Deserialize;
use serde_aux::field_attributes::deserialize_number_from_string;
use sqlx::postgres::{PgConnectOptions, PgSslMode};

#[derive(Clone, Debug, Deserialize)]
pub struct DatabaseConfig {
    pub postgres: Postgres,
}

#[derive(Deserialize, Debug, Clone)]
pub struct Postgres {
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub port: u16,
    pub host: String,
    pub name: String,
    pub username: String,
    // determines of db connection needs to be secure or not
    pub require_ssl: bool,
    pub password: SecretString,
}

impl Postgres {
    pub fn connect_to_instance(&self) -> PgConnectOptions {
        let ssl_mode = if self.require_ssl {
            PgSslMode::Require
        } else {
            PgSslMode::Prefer
        };

        PgConnectOptions::new()
            .port(self.port)
            .host(&self.host)
            .ssl_mode(ssl_mode)
            .username(&self.username)
            .password(&self.password.expose_secret())
    }

    pub fn connect_to_database(&self, database_name: &str) -> PgConnectOptions {
        self.connect_to_instance().database(database_name)
    }
}
