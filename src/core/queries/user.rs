use crate::core::{Asset, DatabaseError, OrderType};
use anyhow::anyhow;
use sqlx::PgPool;
use tracing::error;

#[tracing::instrument(level = "debug", skip(pg_pool, asset), name = "Create new asset")]
pub async fn create_new_asset(asset: &Asset, pg_pool: &PgPool) -> Result<bool, anyhow::Error> {
    tracing::debug!("saving new asset to DB :: id={}", &asset.id);
    let result = sqlx::query!(
        "
        INSERT INTO asset (
            id,
            name,
            symbol,
            description,
            organization,
            created_at,
            updated_at
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7)
        ",
        asset.id,
        asset.name,
        asset.symbol,
        asset.description,
        asset.organization,
        asset.created_at,
        asset.updated_at
    )
        .execute(pg_pool)
        .await
        .map_err(|e| {
            error!("Error executing SQL query: {:?}", e);
            anyhow!("something went wrong")
        })?;
    Ok(result.rows_affected() == 1)
}

#[tracing::instrument(level = "debug", skip(pg_pool))]
pub async fn find_asset_by_id(asset_id: &str, pg_pool: &PgPool) -> Result<Asset, DatabaseError> {
    tracing::debug!("fetching asset :: id={}", asset_id);
    let result = sqlx::query_as!(
        Asset,
        r#"
        SELECT
            id, name, symbol, description, organization, created_at, updated_at
        FROM asset
        WHERE id = $1"#,
        asset_id
    ).
        fetch_one(pg_pool)
        .await?;
    Ok(result)
}

#[tracing::instrument(level = "debug", skip(pg_pool, limit, offset))]
pub async fn get_all_assets(pg_pool: &PgPool, offset: i64, limit: i64, order_by: OrderType) -> Result<Vec<Asset>, DatabaseError> {
    if limit < 1 || limit > 100 {
        return Err(DatabaseError::InvalidArgument("limit must be between 1 and 100".to_string()));
    }
    tracing::debug!("fetching assets from DB :: start={} :: limit={}", &offset, &limit);
    // SQLx often requires i64 for LIMIT & OFFSET to ensure compatibility w/ various DB types & potential large values.
    let result = match order_by {
        OrderType::Asc => {
            sqlx::query_as!(
                Asset,
                r#"
                SELECT
                    id, name, symbol, description, organization, created_at, updated_at
                FROM asset
                ORDER BY name ASC
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
                    id, name, symbol, description, organization, created_at, updated_at
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

#[tracing::instrument(level = "debug", skip(pg_pool, limit, order_by))]
pub async fn find_assets_symbol_like(symbol: &str, limit: i16, offset: i64, order_by: OrderType, pg_pool: &PgPool)
                                     -> Result<Vec<Asset>, DatabaseError> {
    // TO DO: Look into
    // 1. Full-text search: Better for complex searches w/ multiple words, phrases, & linguistic considerations
    // OR
    // 2. Trigram matching: Efficient for finding similar strings, even with typos or partial matches
    let search_term = format!("%{}%", sanitize_search_term(symbol).to_uppercase());
    tracing::debug!("fetching assets from DB :: symbol={}", &search_term);
    let result = match order_by {
        OrderType::Asc => {
            sqlx::query_as!(
                Asset,
                r#"
                SELECT
                    id, name, symbol, description, organization, created_at, updated_at
                FROM asset
                WHERE symbol LIKE $1
                ORDER BY symbol ASC
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
                    id, name, symbol, description, organization, created_at, updated_at
                FROM asset
                WHERE symbol LIKE $1
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

#[tracing::instrument(level = "debug", skip(pg_pool, limit, order_by, offset))]
pub async fn find_assets_name_like(name: &str, offset: i64, limit: usize, order_by: OrderType, pg_pool: &PgPool)
                                   -> Result<Vec<Asset>, DatabaseError> {
    let search_term = format!("%{}%", sanitize_search_term(name));
    tracing::debug!("fetching assets from DB :: name={}", sanitize_search_term(name));

    let result = match order_by {
        OrderType::Asc => {
            sqlx::query_as!(
                Asset,
                r#"
                SELECT
                    id, name, symbol, description, organization, created_at, updated_at
                FROM asset
                WHERE name LIKE $1
                ORDER BY name ASC
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
                    id, name, symbol, description, organization, created_at, updated_at
                FROM asset
                WHERE name LIKE $1
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

fn sanitize_search_term(search_term: &str) -> String {
    search_term.replace('%', "\\%").replace('_', "\\_") // Escape wildcards
}
