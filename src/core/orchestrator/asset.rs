use crate::core::{queries, DatabaseError, OrchestrateError, NFC};
use sqlx::PgPool;
use tracing::info;

/// Transferring an asset should only happen if
/// 1. It is being transferred from one user in the same org to another
/// 2. If it is being transferred from one org to another.
pub async fn orchestrate_transfer_asset(org_id: &str,
                                        asset_id: &str,
                                        new_org_id: &str,
                                        new_asset_owner: &str,
                                        pg_pool: &PgPool)
                                        -> Result<NFC, OrchestrateError> {
    info!("starting asset transfer :: asset_id={}", asset_id);
    // 1. Get asset information
    let asset = queries::find_asset_by_id_and_org_id(&asset_id, &org_id, &pg_pool)
        .await
        .map_err(|e| match e {
            DatabaseError::NotFound => OrchestrateError::NotFoundError("asset not found in specified org".to_string()),
            _ => OrchestrateError::DatabaseError(e),
        })?;

    // 2. Do not transfer an asset if it's the same org, and it's the same user
    if asset.organization == new_org_id && asset.updated_by == new_asset_owner {
        return Err(OrchestrateError::InvalidArgument("invalid".to_string()));
    }

    // 3. get contract information about the asset
    let _ = queries::find_contract_by_asset_id(asset_id, &pg_pool)
        .await
        .map_err(|e| match e {
            DatabaseError::NotFound => OrchestrateError::NotFoundError(asset_id.to_string()),
            _ => OrchestrateError::DatabaseError(e),
        })?;

    // 4. Transfer asset and get NFC for asset back
    let nfc = queries::transfer_asset_query(new_org_id, asset_id, new_asset_owner, &pg_pool)
        .await
        .map_err(|e| match e {
            DatabaseError::NotFound => OrchestrateError::NotFoundError(asset_id.to_string()),
            _ => OrchestrateError::DatabaseError(e),
        })?;

    Ok(nfc)
}
