// crates/gm_domain/src/accounts/events/account_added.rs
//
// Published by AccountService::add_account() after the account record is
// persisted. Interface plugins subscribe to this event to refresh their
// account lists without polling the database.
//
// The event carries the minimum data needed by subscribers — UUID, platform,
// alias, username, and auth_method. Full account details are fetched on demand
// from the repository by any subscriber that needs more fields.

use uuid::Uuid;
use gm_shared::models::account::AuthMethod;
use gm_shared::constants::events::ACCOUNT_ADDED;

#[derive(Debug, Clone)]
pub struct AccountAdded {
    pub account_uuid: Uuid,
    pub platform_id:  Uuid,
    pub alias:        String,
    pub username:     String,
    pub auth_method:  AuthMethod,
}

impl AccountAdded {
    pub fn event_type() -> &'static str {
        ACCOUNT_ADDED
    }
}