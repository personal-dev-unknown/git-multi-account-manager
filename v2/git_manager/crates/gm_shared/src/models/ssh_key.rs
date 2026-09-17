//! SSH key DTO and related enumerations.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Serializable snapshot of an SSH key record.
///
/// Note on security: this DTO deliberately does NOT include the private key
/// passphrase or any encrypted credential material. It contains only the
/// metadata needed for display, and the public key material which is safe
/// to transmit and display.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SshKeyDto {
    pub uuid:              Uuid,
    /// UUID of the account this key belongs to (None for standalone keys)
    pub account_id:        Option<Uuid>,
    /// Human-readable name: "work-github-key"
    pub name:              String,
    pub key_type:          KeyType,
    /// Bit size — only meaningful for RSA and ECDSA keys. Ed25519 is always 256.
    pub key_size_bits:     Option<u16>,
    /// The complete public key string: "ssh-ed25519 AAAA... comment"
    /// Safe to display and copy — this is the value users paste into GitHub.
    pub public_key:        String,
    /// SHA256 fingerprint: "SHA256:base64encoded..."
    /// Used to identify the key in the SSH agent and on the hosting platform.
    pub fingerprint:       String,
    /// Absolute path to the private key file on disk
    pub private_key_path:  String,
    /// Email embedded in the key comment field
    pub comment_email:     Option<String>,
    pub is_added_to_agent: bool,
    pub is_active:         bool,
    pub last_tested_at:    Option<DateTime<Utc>>,
    pub last_test_status:  TestStatus,
    /// The error message from the most recent failed connection test
    pub last_test_error:   Option<String>,
    pub created_at:        DateTime<Utc>,
}

/// The cryptographic algorithm used to generate the key pair.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum KeyType {
    /// Ed25519 — recommended for all new keys. Fixed 256-bit size.
    /// Fast, small keys, and resistant to side-channel attacks.
    Ed25519,
    /// RSA — legacy but widely supported. Use 4096-bit minimum.
    Rsa,
    /// ECDSA with NIST P-256 or P-384. Faster than RSA but not as clean
    /// as Ed25519. Included for compatibility with older systems.
    Ecdsa,
    /// DSA — deprecated and disabled in modern OpenSSH. Only listed for
    /// display of legacy keys; new DSA keys cannot be generated.
    Dsa,
}

impl std::fmt::Display for KeyType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            KeyType::Ed25519 => write!(f, "ed25519"),
            KeyType::Rsa     => write!(f, "rsa"),
            KeyType::Ecdsa   => write!(f, "ecdsa"),
            KeyType::Dsa     => write!(f, "dsa"),
        }
    }
}

/// The result of the most recent SSH connection test for this key.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TestStatus {
    /// Connection test passed; the key is accepted by the hosting platform.
    Success,
    /// Connection test failed; `last_test_error` contains the diagnostic.
    Failed,
    /// No connection test has been run yet for this key.
    Untested,
}

impl std::fmt::Display for TestStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TestStatus::Success  => write!(f, "success"),
            TestStatus::Failed   => write!(f, "failed"),
            TestStatus::Untested => write!(f, "untested"),
        }
    }
}