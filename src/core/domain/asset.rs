use crate::core::domain::error::DomainError;
use crate::core::domain::key::{generate_unique_key, DOMAIN_KEY_SIZE};
use chrono::{DateTime, Utc};

const MIN_NAME_LENGTH: usize = 3;
const MAX_NAME_LENGTH: usize = 32;

pub struct Asset {
    pub id: String,
    pub name: String,
    pub symbol: String,
    pub description: String,
    pub organization: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Asset {
    pub fn new(
        name: String,
        symbol: String,
        description: String,
        organization: String,
    ) -> Result<Self, DomainError> {
        Self::validate_name(&name)?;
        let now = Utc::now();
        let asset_id = generate_unique_key(DOMAIN_KEY_SIZE);
        Ok(Self {
            name,
            symbol,
            description,
            organization,
            id: asset_id,
            created_at: now,
            updated_at: now,
        })
    }

    fn validate_name(name: &str) -> Result<(), DomainError> {
        if name.is_empty() || name.len() < MIN_NAME_LENGTH || name.len() > MAX_NAME_LENGTH {
            let error_msg = format!(
                "name should be between {MIN_NAME_LENGTH} and {MAX_NAME_LENGTH} characters long"
            )
            .to_string();
            return Err(DomainError::InvalidArgument(error_msg));
        }
        Ok(())
    }
}
