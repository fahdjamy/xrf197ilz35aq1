use crate::core::domain::key::generate_unique_key;

pub struct Asset {
    pub id: String,
    pub name: String,
    pub created_at: String,
    pub updated_at: String,
    pub description: String,
    pub organization: String,
}

impl Asset {
    pub fn new(name: String, description: String, organization: String) -> Self {
        Asset {
            name,
            description,
            organization,
            id: generate_unique_key(45),
            created_at: "".to_string(),
            updated_at: "".to_string(),
        }
    }
}
