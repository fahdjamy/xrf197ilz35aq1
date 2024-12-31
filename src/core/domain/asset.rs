use crate::core::domain::error::DomainError;
use crate::core::domain::key::{generate_unique_key, DOMAIN_KEY_SIZE};
use chrono::{DateTime, Utc};
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct Asset {
    pub id: String,
    pub name: String,
    pub symbol: String,
    pub tradable: bool,
    pub listable: bool, // defines if an asset should be listed in a list of assets, may or may not be tradable
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
        Self::validate_symbol(&symbol)?;
        Self::validate_organization(&organization)?;
        let now = Utc::now();
        let asset_id = generate_unique_key(DOMAIN_KEY_SIZE);
        Ok(Self {
            name,
            description,
            organization,
            id: asset_id,
            listable: true,
            tradable: false,
            created_at: now,
            updated_at: now,
            symbol: symbol.to_uppercase(),
        })
    }

    fn validate_name(name: &str) -> Result<(), DomainError> {
        const MIN_LENGTH: usize = 3;
        const MAX_LENGTH: usize = 32;

        if name.is_empty() || name.len() < MIN_LENGTH || name.len() > MAX_LENGTH {
            let error_msg =
                format!("name should be between {MIN_LENGTH} and {MAX_LENGTH} characters long")
                    .to_string();
            return Err(DomainError::InvalidArgument(error_msg));
        }
        Ok(())
    }

    fn validate_symbol(symbol: &str) -> Result<(), DomainError> {
        const MIN_LENGTH: usize = 3;
        const MAX_LENGTH: usize = 10;
        if symbol.is_empty() || symbol.len() < MIN_LENGTH || symbol.len() > MAX_LENGTH {
            let error = format!("symbol should be between {MIN_LENGTH} and {MAX_LENGTH} characters long");
            return Err(DomainError::InvalidArgument(error));
        }
        if symbol.chars().any(|c| c.is_whitespace()) {
            let error = "symbol should not contain a whitespace".to_string();
            return Err(DomainError::InvalidArgument(error));
        }
        Ok(())
    }

    fn validate_organization(org: &str) -> Result<(), DomainError> {
        const MIN_LENGTH: usize = 32;
        if org.is_empty() || org.len() < MIN_LENGTH {
            let error = format!("orgId should at least be of length {MIN_LENGTH} characters long");
            return Err(DomainError::InvalidArgument(error));
        }
        if Uuid::parse_str(org).is_err() {
            let error = "orgId should be a valid UUID".to_string();
            return Err(DomainError::InvalidArgument(error));
        }
        Ok(())
    }
}
