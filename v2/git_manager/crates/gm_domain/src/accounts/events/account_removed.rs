// crates/gm_domain/src/accounts/events/account_removed.rs
//
// Published by AccountService::remove_account() after the account record is
// deleted. The workflow engine listens for this event and can trigger cleanup
// workflows — for example, removing SSH host config entries or archiving
// repository records associated with the deleted account.

use uuid::Uuid;
use gm_shared::constants::events::ACCOUNT_REMOVED;

#[derive(Debug, Clone)]
pub struct AccountRemoved {
    pub account_uuid: Uuid,
    pub platform_id:  Uuid,
    /// The alias at the time of deletion — useful for audit log display since
    /// the account record no longer exists to query.
    pub alias:        String,
}

impl AccountRemoved {
    pub fn event_type() -> &'static str {
        ACCOUNT_REMOVED
    }
}