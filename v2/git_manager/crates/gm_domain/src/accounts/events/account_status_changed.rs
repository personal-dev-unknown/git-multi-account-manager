// crates/gm_domain/src/accounts/events/account_status_changed.rs
//
// Published whenever an account transitions between lifecycle states.
// Both the previous and new status strings are included so that subscribers
// can react differently depending on the direction of the transition
// (e.g., notify the user differently for Unverified → Active vs Active → Suspended).

use uuid::Uuid;
use gm_shared::constants::events::ACCOUNT_STATUS_CHANGED;

#[derive(Debug, Clone)]
pub struct AccountStatusChanged {
    pub account_uuid:    Uuid,
    pub previous_status: String,
    pub new_status:      String,
}

impl AccountStatusChanged {
    pub fn event_type() -> &'static str {
        ACCOUNT_STATUS_CHANGED
    }
}