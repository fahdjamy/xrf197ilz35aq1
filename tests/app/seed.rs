use sqlx::PgPool;
use uuid::Uuid;
use xrf1::core::{queries, Asset, DomainError};

pub async fn create_and_save_contract(
    user_fp: String,
    pg: &PgPool,
) -> Result<Asset, Box<dyn std::error::Error>> {
    let asset = create_asset(user_fp.clone())?;

    queries::create_new_asset(&asset, user_fp, &pg).await?;

    Ok(asset)
}

pub fn create_asset(owner_fp: String) -> Result<Asset, DomainError> {
    let asset_name = Uuid::new_v4().to_string()[..15].to_string(); // Truncate to the first 15 characters

    let symbol = "XRF-PL1".to_string();
    let org_id = Uuid::new_v4().to_string();
    let description = Uuid::new_v4().to_string();

    Asset::new(asset_name, symbol, owner_fp, description, org_id)
}

pub fn create_asset_owner() -> String {
    Uuid::new_v4().to_string().to_string()
}

pub fn create_org_id() -> String {
    Uuid::new_v4().to_string().to_string()
}