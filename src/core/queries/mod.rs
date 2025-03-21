mod asset;
mod contract;
mod nfc;
mod ordering;

pub use asset::{
    create_new_asset, delete_asset_by_id, find_asset_by_id, find_asset_by_id_and_org_id,
    find_assets_name_like, find_assets_symbol_like, get_all_assets, transfer_asset_query,
    update_asset
};
pub use contract::{create_contract, get_contract_by_asset_id};
pub use nfc::{create_nfc, create_nfc_trail, get_nfc_by_asset_id, get_nfc_by_id, get_nfc_trails_by_nfc_id};
pub use ordering::OrderType;
use sqlx::{Postgres, Transaction};

// PgTransaction type alias for Transaction<'a, Postgres> represents a database transaction.
// It's a "handle" to a series of operations that must happen (either all succeed or are rolled back).
// The 'a is a lifetime param, ensuring the transaction doesn't outlive the db connection it's tied to.
pub type PgTransaction<'a> = Transaction<'a, Postgres>;
