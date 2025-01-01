use crate::core::{Asset, DatabaseError, OrderType, UpdateAssetRequest};
use anyhow::anyhow;
use sqlx::{PgPool, QueryBuilder};
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
            updated_at,
            updated_by
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
        ",
        asset.id,
        asset.name,
        asset.symbol,
        asset.description,
        asset.organization,
        asset.created_at,
        asset.updated_at,
        asset.updated_by,
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
            id, name, symbol, description, organization, created_at, updated_at, tradable, listable, updated_by
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
                    id, name, symbol, description, organization, created_at, updated_at, tradable, listable, updated_by
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
                    id, name, symbol, description, organization, created_at, updated_at, tradable, listable, updated_by
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
    tracing::debug!("fetching assets from DB :: symbol = {}", &search_term);
    let result = match order_by {
        OrderType::Asc => {
            sqlx::query_as!(
                Asset,
                r#"
                SELECT
                    id, name, symbol, description, organization, created_at, updated_at, tradable, listable, updated_by
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
                    id, name, symbol, description, organization, created_at, updated_at, tradable, listable, updated_by
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
                    id, name, symbol, description, organization, created_at, updated_at, tradable, listable, updated_by
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
                    id, name, symbol, description, organization, created_at, updated_at, tradable, listable, updated_by
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

#[tracing::instrument(level = "debug", skip(pg_pool))]
pub async fn delete_asset_by_id(asset_id: &str, pg_pool: &PgPool) -> Result<bool, DatabaseError> {
    tracing::debug!("deleting asset :: id = {}", asset_id);
    let result = sqlx::query!(
        "
        DELETE FROM asset
        WHERE id = $1
        ",
        asset_id
    )
        .execute(pg_pool)
        .await?;
    Ok(result.rows_affected() == 1)
}

#[tracing::instrument(level = "debug", skip(pg_pool, asset))]
pub async fn update_asset(asset_id: &str, asset: &UpdateAssetRequest, pg_pool: &PgPool) -> Result<bool, DatabaseError> {
    let mut query_builder = QueryBuilder::new("UPDATE asset SET ");

    let mut set_clauses = Vec::new();
    if let Some(name) = &asset.name {
        set_clauses.push(format!("name = '{}'", name.replace('\'', "''")));
    }
    if let Some(symbol) = &asset.symbol {
        set_clauses.push(format!("symbol = '{}'", symbol.replace('\'', "''")));
    }
    if let Some(description) = &asset.description {
        set_clauses.push(format!("description = '{}'", description.replace('\'', "''")));
    }
    if let Some(listable) = asset.listable {
        set_clauses.push(format!("listable = {}", listable));
    }
    if let Some(tradable) = asset.tradable {
        set_clauses.push(format!("tradable = {}", tradable));
    }
    set_clauses.push(format!("updated_at = '{}'", asset.updated_at));
    if let Some(organization) = &asset.organization {
        set_clauses.push(format!("organization = '{}'", organization.replace('\'', "''")));
    }
    set_clauses.push(format!("updated_by = '{}'", asset.updated_by.replace('\'', "''")));

    query_builder.push(set_clauses.join(", "));
    query_builder.push(" WHERE id = ");
    query_builder.push_bind(asset_id); // Use bind for the ID

    let query = query_builder.build();

    match query.execute(pg_pool).await {
        Ok(res) => {
            if res.rows_affected() > 0 {
                Ok(true)
            } else {
                Ok(false)
            }
        }
        Err(e) => {
            error!("Error executing SQL query: {:?}", e);
            Err(DatabaseError::from(e))
        },
    }
}

fn sanitize_search_term(search_term: &str) -> String {
    search_term.replace('%', "\\%").replace('_', "\\_") // Escape wildcards
}
