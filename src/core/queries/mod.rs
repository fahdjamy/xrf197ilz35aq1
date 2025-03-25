mod asset;
mod contract;
mod nfc;
mod ordering;

pub use asset::*;
pub use contract::{create_contract, find_contract_by_asset_id};
pub use nfc::{create_nfc, create_nfc_trail, get_nfc_by_asset_id, get_nfc_by_id, get_nfc_trails_by_nfc_id};
pub use ordering::OrderType;
use sqlx::{Postgres, Transaction};

// PgTransaction type alias for Transaction<'a, Postgres> represents a database transaction.
// It's a "handle" to a series of operations that must happen (either all succeed or are rolled back).
// The 'a is a lifetime param, ensuring the transaction doesn't outlive the db connection it's tied to.
pub type PgTransaction<'a> = Transaction<'a, Postgres>;
