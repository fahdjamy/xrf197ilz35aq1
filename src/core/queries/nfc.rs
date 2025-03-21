use crate::core::{DatabaseError, NFCTrail, PgTransaction, NFC};
use sqlx::PgPool;
use tracing::info;

#[tracing::instrument(skip(nfc_id, pool))]
pub async fn get_nfc_by_id(nfc_id: &str, pool: &PgPool) -> Result<NFC, DatabaseError> {
    info!("Getting nfc by id={}", nfc_id);
    let row = sqlx::query_as!(
        NFC,
        r#"
        SELECT id, asset_id, cert, created_at FROM nfc WHERE id = $1
        "#,
        nfc_id
    )
    .fetch_one(pool)
    .await?;

    Ok(row)
}

#[tracing::instrument(skip(transaction, nf_cert, user_fp))]
pub async fn create_nfc(
    mut transaction: PgTransaction<'_>,
    nf_cert: NFC,
    user_fp: String,
) -> Result<bool, DatabaseError> {
    info!("Creating nfc :: id={}", &nf_cert.id);
    let result = sqlx::query!(
        r#"
        INSERT INTO nfc (id, cert, asset_id, created_at)
        VALUES ($1, $2, $3, $4)
        "#,
        nf_cert.id,
        nf_cert.cert,
        nf_cert.asset_id,
        nf_cert.created_at,
    )
    .execute(&mut *transaction)
    .await?;

    let nfc_created = result.rows_affected() == 1;
    if !nfc_created {
        // No need to rollback here, as nothing has been committed yet.
        return Ok(false);
    }

    // create new nfc trail history if nfc is created
    let trail = NFCTrail::new(nf_cert.id, user_fp, nf_cert.asset_id);
    let nfc_trail_created = create_nfc_trail(&mut transaction, &trail).await?;
    if !nfc_trail_created {
        transaction.rollback().await?;
        return Ok(false);
    }

    transaction.commit().await?;
    Ok(true) // Return true, we know both operations succeeded at this point.
}

#[tracing::instrument(skip(pg_pool, asset_id))]
pub async fn get_nfc_by_asset_id(asset_id: &str, pg_pool: &PgPool) -> Result<NFC, DatabaseError> {
    info!("Getting nfc by id={}", asset_id);
    let row = sqlx::query_as!(
        NFC,
        r#"
        SELECT id, asset_id, cert, created_at 
        FROM nfc 
        WHERE asset_id = $1
        "#,
        asset_id
    )
        .fetch_one(pg_pool)
        .await?;

    Ok(row)
}

#[tracing::instrument(skip(transaction, trail))]
pub async fn create_nfc_trail(
    transaction: &mut PgTransaction<'_>,
    trail: &NFCTrail,
) -> Result<bool, DatabaseError> {
    info!(
        "Creating nfc trail :: nfc_id={} :: created_on={}",
        &trail.nfc_id, trail.transferred_on
    );
    let result = sqlx::query!(
        r#"
        INSERT INTO nfc_asset_trail (nfc_id, user_fp, asset_id, transferred_on)
        VALUES ($1, $2, $3, $4)
        "#,
        trail.nfc_id,
        trail.user_fp,
        trail.asset_id,
        trail.transferred_on,
    )
    .execute(&mut **transaction)
    .await?;
    Ok(result.rows_affected() == 1)
}

#[tracing::instrument(skip(nfc_id, pool))]
pub async fn get_nfc_trails_by_nfc_id(
    nfc_id: &str,
    pool: &PgPool,
) -> Result<Vec<NFCTrail>, DatabaseError> {
    info!("Getting nfc trail history by nfc_id={}", nfc_id);
    let rows = sqlx::query_as!(
        NFCTrail,
        "SELECT * FROM nfc_asset_trail WHERE nfc_id = $1",
        nfc_id
    )
    .fetch_all(pool)
    .await?;
    Ok(rows)
}
