use crate::core::domain::key::{generate_unique_key, DOMAIN_KEY_SIZE};
use chrono::{DateTime, Utc};

pub struct Asset {
    pub id: String,
    pub name: String,
    pub description: String,
    pub organization: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Asset {
    pub fn new(name: String, description: String, organization: String) -> Self {
        let now = Utc::now();
        Asset {
            name,
            description,
            organization,
            created_at: now,
            updated_at: now,
            id: generate_unique_key(DOMAIN_KEY_SIZE),
        }
    }
}
