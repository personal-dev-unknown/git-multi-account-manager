// crates/gm_domain/src/ssh/value_objects/test_status.rs
//
// The result of testing an SSH key against its target platform. Every SSH key
// stores its last test result so the UI can show users which keys are verified
// and which need attention without running a connection test on every display.

use chrono::{DateTime, Utc};
use gm_shared::models::ssh_key::TestStatus as SharedTestStatus;

/// The outcome of the most recent SSH connection test for a key.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TestStatus {
    /// No test has been run yet (new key state).
    Untested,
    /// The most recent test connected successfully and the platform authenticated the key.
    Success {
        /// The platform-confirmed username returned in the SSH greeting,
        /// e.g. "Hi shakamoses!" → "shakamoses".
        username:    String,
        /// When the successful test was performed.
        tested_at:   DateTime<Utc>,
    },
    /// The most recent test failed.
    Failed {
        /// Human-readable description of why the test failed.
        reason:    String,
        tested_at: DateTime<Utc>,
    },
}

impl TestStatus {
    /// Returns true if the last test confirmed the key works.
    pub fn is_verified(&self) -> bool {
        matches!(self, TestStatus::Success { .. })
    }

    /// Returns true if this key has never been tested.
    pub fn is_untested(&self) -> bool {
        matches!(self, TestStatus::Untested)
    }

    /// Returns the time of the most recent test, or None for untested keys.
    pub fn last_tested_at(&self) -> Option<DateTime<Utc>> {
        match self {
            TestStatus::Untested              => None,
            TestStatus::Success { tested_at, .. } => Some(*tested_at),
            TestStatus::Failed  { tested_at, .. } => Some(*tested_at),
        }
    }

    pub fn to_shared(&self) -> SharedTestStatus {
        match self {
            TestStatus::Untested     => SharedTestStatus::Untested,
            TestStatus::Success { .. } => SharedTestStatus::Success,
            TestStatus::Failed  { .. } => SharedTestStatus::Failed,
        }
    }

    pub fn from_shared(shared: &SharedTestStatus) -> Self {
        match shared {
            SharedTestStatus::Untested => TestStatus::Untested,
            SharedTestStatus::Success  => TestStatus::Success {
                username:  String::new(),
                tested_at: Utc::now(),
            },
            SharedTestStatus::Failed   => TestStatus::Failed {
                reason:    "unknown".to_string(),
                tested_at: Utc::now(),
            },
        }
    }
}

impl std::fmt::Display for TestStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TestStatus::Untested                          => write!(f, "untested"),
            TestStatus::Success { username, .. }          => write!(f, "verified ({})", username),
            TestStatus::Failed  { reason, .. }            => write!(f, "failed: {}", reason),
        }
    }
}