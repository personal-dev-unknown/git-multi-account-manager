// crates/gm_ports/src/inbound/queries/get_accounts.rs
//
// Query to retrieve a list of accounts. Optionally filtered by platform.
// Queries differ from commands: they have no side effects and do not modify state.

use serde::{Deserialize, Serialize};
use uuid::Uuid;
use gm_shared::models::account::AccountDto;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetAccountsQuery {
    /// Filter by platform UUID. If None, all accounts across all platforms are returned.
    pub platform_id: Option<Uuid>,
}

/// The response to a GetAccountsQuery.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetAccountsResult {
    pub accounts: Vec<AccountDto>,
    pub total:    usize,
}