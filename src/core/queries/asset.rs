use crate::core::queries::{create_nfc, create_nfc_trail, get_nfc_by_asset_id, OrderType};
use crate::core::{Asset, DatabaseError, NFCTrail, UpdateAssetRequest, NFC};
use anyhow::anyhow;
use chrono::Utc;
use sqlx::{PgPool, QueryBuilder};
use tracing::error;

#[tracing::instrument(level = "debug", skip(pg_pool, asset), name = "Create new asset")]
pub async fn create_new_asset(
    asset: &Asset,
    user_fp: String,
    pg_pool: &PgPool,
) -> Result<bool, anyhow::Error> {
    tracing::debug!("saving new asset to DB :: id={}", &asset.id);
    let mut transaction = pg_pool.begin().await?;
    let result = sqlx::query!(
        "
        INSERT INTO asset (
            id,
            name,
            symbol,
            owner_fp,
            description,
            organization,
            created_at,
            updated_at,
            updated_by,
            listable
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
        ",
        asset.id,
        asset.name,
        asset.symbol,
        asset.owner_fp,
        asset.description,
        asset.organization,
        asset.created_at,
        asset.updated_at,
        asset.updated_by,
        asset.listable,
    )
        .execute(&mut *transaction)
        .await
        .map_err(|e| {
            error!("Error executing SQL query: {:?}", e);
            anyhow!("something went wrong")
        })?;
    let nf_cert = NFC::new(asset.id.clone()).map_err(|e| anyhow!(e))?;

    if result.rows_affected() == 0 {
        return Ok(false);
    }

    create_nfc(transaction, nf_cert, user_fp)
        .await
        .map_err(|e| {
            error!("Error creating NFC table: {:?}", e);
            anyhow!("Error creating NFC table")
        })?;

    Ok(true)
}

#[tracing::instrument(level = "debug", skip(pg_pool))]
pub async fn find_asset_by_id(asset_id: &str, pg_pool: &PgPool) -> Result<Asset, DatabaseError> {
    tracing::debug!("fetching asset :: id={}", asset_id);
    let result = sqlx::query_as!(
        Asset,
        r#"
        SELECT
            id, name, symbol, description, organization, created_at, updated_at, tradable, listable,
            updated_by, owner_fp
        FROM asset
        WHERE id = $1"#,
        asset_id
    ).
        fetch_one(pg_pool)
        .await?;
    Ok(result)
}

#[tracing::instrument(level = "debug", skip(pg_pool, asset_id, org_id))]
pub async fn find_asset_by_id_and_org_id(
    asset_id: &str,
    org_id: &str,
    pg_pool: &PgPool,
) -> Result<Asset, DatabaseError> {
    let result = sqlx::query_as!(
        Asset,
        r#"
        SELECT
            id, name, symbol, description, organization, created_at, updated_at, tradable, listable,
            updated_by, owner_fp
        FROM asset
        WHERE id = $1 AND organization = $2"#,
        asset_id,
        org_id
    )
        .fetch_one(pg_pool)
        .await?;
    Ok(result)
}

#[tracing::instrument(level = "debug", skip(pg_pool, limit, offset, order_by))]
pub async fn get_all_assets(
    pg_pool: &PgPool,
    offset: i64,
    limit: i64,
    order_by: OrderType,
) -> Result<Vec<Asset>, DatabaseError> {
    if limit < 1 || limit > 100 {
        return Err(DatabaseError::InvalidArgument(
            "limit must be between 1 and 100".to_string(),
        ));
    }
    tracing::debug!(
        "fetching assets from DB :: start={} :: limit={}",
        &offset,
        &limit
    );
    // SQLx often requires i64 for LIMIT & OFFSET to ensure compatibility w/ various DB types and potential large values.
    let result = match order_by {
        OrderType::Asc => {
            sqlx::query_as!(
                Asset,
                r#"
                SELECT
                    id, name, symbol, description, organization, created_at, updated_at, tradable,
                    listable, updated_by, owner_fp
                FROM asset
                ORDER BY name
                LIMIT $1 OFFSET $2
                "#,
                limit,
                offset
            )
                .fetch_all(pg_pool)
                .await?
        }
        OrderType::Desc => {
            sqlx::query_as!(
                Asset,
                r#"
                SELECT
                    id, name, symbol, description, organization, created_at, updated_at, tradable,
                    listable, updated_by, owner_fp
                FROM asset
                ORDER BY name DESC
                LIMIT $1 OFFSET $2
                "#,
                limit,
                offset
            )
                .fetch_all(pg_pool)
                .await?
        }
    };
    Ok(result)
}

#[tracing::instrument(level = "debug", skip(pg_pool, limit, offset, symbol, order_by))]
pub async fn find_assets_symbol_like(
    symbol: &str,
    limit: i16,
    offset: i64,
    order_by: OrderType,
    pg_pool: &PgPool,
) -> Result<Vec<Asset>, DatabaseError> {
    // TO DO: Look into
    // 1. Full-text search: Better for complex searches w/ multiple words, phrases, and linguistic considerations
    // OR
    // 2. Trigram matching: Efficient for finding similar strings, even with typos or partial matches
    let search_term = format!("%{}%", sanitize_search_term(symbol).to_uppercase());
    tracing::debug!("fetching assets from DB :: symbol = {}", &search_term);
    let result = match order_by {
        OrderType::Asc => {
            sqlx::query_as!(
                Asset,
                r#"
                SELECT
                    id, name, symbol, description, organization, created_at, updated_at, tradable,
                    listable, updated_by, owner_fp
                FROM asset
                WHERE symbol ILIKE $1
                ORDER BY symbol
                LIMIT $2
                OFFSET $3"#,
                search_term,
                limit as i64,
                offset
            )
                .fetch_all(pg_pool)
                .await?
        }
        OrderType::Desc => {
            sqlx::query_as!(
                Asset,
                r#"
                SELECT
                    id, name, symbol, description, organization, created_at, updated_at, tradable,
                    listable, updated_by, owner_fp
                FROM asset
                WHERE symbol ILIKE $1
                ORDER BY symbol DESC
                LIMIT $2"#,
                search_term,
                limit as i64
            )
                .fetch_all(pg_pool)
                .await?
        }
    };
    Ok(result)
}

#[tracing::instrument(level = "debug", skip(pg_pool, owner_fp, offset, limit, order_by))]
pub async fn find_assets_by_owner(owner_fp: &str,
                                  limit: i16,
                                  offset: i64,
                                  listable: bool,
                                  order_by: OrderType, pg_pool: &PgPool) -> Result<Vec<Asset>, DatabaseError> {
    let search_term = format!("%{}%", sanitize_search_term(owner_fp).to_uppercase());
    tracing::debug!("fetching assets from DB :: symbol = {}", &search_term);
    let result = match order_by {
        OrderType::Asc => {
            sqlx::query_as!(
                Asset,
                r#"
                SELECT
                    id, name, symbol, description, organization, created_at, updated_at, tradable, listable, updated_by, owner_fp
                FROM asset
                WHERE owner_fp = $1 AND listable = $2
                ORDER BY symbol
                LIMIT $3
                OFFSET $4"#,
                owner_fp,
                listable,
                limit as i64,
                offset
            )
                .fetch_all(pg_pool)
                .await?
        }
        OrderType::Desc => {
            sqlx::query_as!(
                Asset,
                r#"
                SELECT
                    id, name, symbol, description, organization, created_at, updated_at, tradable, listable, updated_by, owner_fp
                FROM asset
                WHERE owner_fp = $1 AND listable = $2
                ORDER BY symbol DESC
                LIMIT $3
                OFFSET $4"#,
                search_term,
                listable,
                limit as i64,
                offset as i64,
            )
                .fetch_all(pg_pool)
                .await?
        }
    };

    Ok(result)
}

#[tracing::instrument(level = "debug", skip(pg_pool, limit, order_by, offset))]
pub async fn find_assets_name_like(
    name: &str,
    offset: i64,
    limit: usize,
    order_by: OrderType,
    pg_pool: &PgPool,
) -> Result<Vec<Asset>, DatabaseError> {
    let search_term = format!("%{}%", sanitize_search_term(name));
    tracing::debug!(
        "fetching assets from DB :: name={}",
        sanitize_search_term(name)
    );

    let result = match order_by {
        OrderType::Asc => {
            sqlx::query_as!(
                Asset,
                r#"
                SELECT
                    id, name, symbol, description, organization, created_at, updated_at, tradable,
                    listable, updated_by, owner_fp
                FROM asset
                WHERE name ILIKE $1
                ORDER BY name
                LIMIT $2
                OFFSET $3"#,
                search_term,
                limit as i64,
                offset
            ).fetch_all(pg_pool).await?
        }
        OrderType::Desc => {
            sqlx::query_as!(
                Asset,
                r#"
                SELECT
                    id, name, symbol, description, organization, created_at, updated_at, tradable, listable, updated_by,
                    owner_fp
                FROM asset
                WHERE name ILIKE $1
                ORDER BY name DESC
                LIMIT $2
                OFFSET $3"#,
                search_term,
                limit as i64,
                offset
            ).fetch_all(pg_pool).await?
        }
    };

    Ok(result)
}

#[tracing::instrument(level = "debug", skip(pg_pool))]
pub async fn delete_asset_by_id(asset_id: &str, pg_pool: &PgPool) -> Result<bool, DatabaseError> {
    tracing::debug!("deleting asset :: id = {}", asset_id);
    let result = sqlx::query!("DELETE FROM asset WHERE id = $1", asset_id)
        .execute(pg_pool)
        .await?;
    Ok(result.rows_affected() == 1)
}

#[tracing::instrument(level = "debug", skip(pg_pool, asset_id, new_owner_fp))]
pub async fn transfer_asset_query(new_org: &str,
                                  asset_id: &str,
                                  new_owner_fp: &str,
                                  pg_pool: &PgPool)
                                  -> Result<NFC, DatabaseError> {
    let nfc = get_nfc_by_asset_id(asset_id, pg_pool)
        .await
        .map_err(|e| match e {
            DatabaseError::NotFound => DatabaseError::InvalidRecordState("Invalid asset without nfc".to_string()),
            _ => DatabaseError::Unknown("something went wrong".to_string())
        })?;

    let mut transaction = pg_pool.begin().await?;
    let result = sqlx::query!(r#"
    UPDATE asset
    SET organization = $1, updated_by = $2, owner_fp = $2 WHERE id = $3
"#,
        new_org, new_owner_fp, asset_id,
    )
        .execute(&mut *transaction)
        .await?;

    if result.rows_affected() != 1 {
        transaction.rollback().await?;
        return Err(DatabaseError::TransactionStepError("Failed to transfer asset and rolled back".to_string()));
    }

    let nfc_trail = NFCTrail::new(nfc.id.clone(), new_owner_fp.to_string(), asset_id.to_string());
    let trail_created = create_nfc_trail(&mut transaction, &nfc_trail).await?;

    if !trail_created {
        transaction.rollback().await?;
        return Err(DatabaseError::TransactionStepError("failed to create NFC trail and rolled back".to_string()));
    }

    transaction.commit().await?;
    Ok(nfc)
}

#[tracing::instrument(level = "debug", skip(pg_pool, asset))]
pub async fn update_asset(
    asset_id: &str,
    updated_by: &str,
    asset: &UpdateAssetRequest,
    pg_pool: &PgPool,
) -> Result<bool, DatabaseError> {
    if updated_by.is_empty() || updated_by.len() < 50 {
        return Err(DatabaseError::InvalidArgument(
            "updated_by is required".to_string(),
        ));
    }
    // no field is there to be updated, return early
    if asset.name.is_none()
        && asset.symbol.is_none()
        && asset.listable.is_none()
        && asset.tradable.is_none()
        && asset.description.is_none()
        && asset.organization.is_none()
    {
        return Ok(true);
    }
    let mut first = true;
    let str_fields = vec![
        ("name", &asset.name),
        ("symbol", &asset.symbol),
        ("description", &asset.description),
        ("organization", &asset.organization),
    ];
    let mut query_builder = QueryBuilder::new("UPDATE asset SET ");
    for (field_name, field_value) in str_fields.iter().filter(|(_, v)| v.is_some()) {
        if !first {
            query_builder.push(", ");
        }
        query_builder.push(format!("{} = ", field_name));
        query_builder.push_bind(field_value);
        first = false;
    }

    if let Some(listable) = asset.listable {
        if !first {
            query_builder.push(", ");
        }
        query_builder.push("listable = ").push_bind(listable);
        first = false;
    }
    // this is the last field to be added, remember to add 'first = false' if it's not
    if let Some(tradable) = asset.tradable {
        if !first {
            query_builder.push(", ");
        }
        query_builder.push("tradable = ").push_bind(tradable);
        first = false;
    }

    if !first {
        query_builder.push(", ");
    }

    // SET the necessary fields
    query_builder.push("updated_at = ").push_bind(Utc::now());
    query_builder
        .push(", updated_by = ")
        .push_bind(updated_by);

    // SET WHERE clause
    query_builder.push(" WHERE id = ").push_bind(asset_id);

    if pg_pool.is_closed() {
        error!("Database connection pool is closed");
        return Err(DatabaseError::PoolClosed);
    }

    match query_builder.build().execute(pg_pool).await {
        Ok(res) => Ok(res.rows_affected() > 0),
        Err(e) => {
            error!("Error executing SQL query: {:?}", e);
            Err(DatabaseError::from(e))
        }
    }
}

fn sanitize_search_term(search_term: &str) -> String {
    search_term.replace('%', "\\%").replace('_', "\\_") // Escape wildcards
}
