//! The Credential entity — an encrypted secret associated with an account.
//!
//! Credentials are NOT stored in plaintext anywhere in the domain. The
//! `encrypted_value` field holds ciphertext produced by the kernel's
//! CredentialVault (AES-256-GCM) or the platform keychain via the Zig layer.
//! The domain entity records metadata about the credential and references
//! the key that encrypted it, but never holds or derives the plaintext.
//!
//! The domain service that creates credentials receives a pre-encrypted byte
//! string from the kernel security layer and stores it here — the domain has
//! no access to the encryption key or decryption logic.

use chrono::{DateTime, Utc};
use uuid::Uuid;

/// The type of secret stored in a credential record.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CredentialType {
    /// Personal Access Token from the hosting platform.
    Pat,
    /// OAuth 2.0 access token (short-lived; paired with a RefreshToken).
    OauthAccessToken,
    /// OAuth 2.0 refresh token (long-lived; used to obtain new access tokens).
    OauthRefreshToken,
    /// HTTPS password (legacy; avoided where PATs are available).
    HttpsPassword,
    /// Passphrase for an SSH private key (stored if the user wants agent-less operation).
    SshPassphrase,
}

impl CredentialType {
    pub fn as_str(&self) -> &'static str {
        match self {
            CredentialType::Pat              => "pat",
            CredentialType::OauthAccessToken => "oauth_access_token",
            CredentialType::OauthRefreshToken=> "oauth_refresh_token",
            CredentialType::HttpsPassword    => "https_password",
            CredentialType::SshPassphrase    => "ssh_passphrase",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "pat"                => Some(CredentialType::Pat),
            "oauth_access_token" => Some(CredentialType::OauthAccessToken),
            "oauth_refresh_token"=> Some(CredentialType::OauthRefreshToken),
            "https_password"     => Some(CredentialType::HttpsPassword),
            "ssh_passphrase"     => Some(CredentialType::SshPassphrase),
            _                    => None,
        }
    }
}

impl std::fmt::Display for CredentialType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// An encrypted credential linked to a single account.
///
/// The `encrypted_value` is an opaque ciphertext produced by the kernel's
/// AES-256-GCM vault. The `encryption_key_id` identifies which key version
/// was used, enabling future key rotation without losing access to stored
/// credentials. The `iv_hex` is the initialisation vector, stored alongside
/// the ciphertext because AES-GCM IVs are not secret — they must simply
/// be unique per encryption.
#[derive(Debug, Clone)]
pub struct Credential {
    uuid:             Uuid,
    account_id:       Uuid,
    credential_type:  CredentialType,
    /// AES-256-GCM ciphertext in hex or base64 encoding.
    /// Never log, display, or transmit this field without first decrypting it.
    encrypted_value:  String,
    /// Identifies which vault key was used. Format: "v1", "v2", etc.
    encryption_key_id: String,
    /// Hex-encoded 12-byte AES-GCM initialisation vector.
    iv_hex:           String,
    /// When this credential expires. None means non-expiring (e.g. PATs without expiry).
    expires_at:       Option<DateTime<Utc>>,
    is_active:        bool,
    created_at:       DateTime<Utc>,
    updated_at:       DateTime<Utc>,
}

impl Credential {
    /// Creates a new Credential record from pre-encrypted material.
    /// The caller (AccountService) receives the encrypted_value from the
    /// kernel's CredentialVault before calling this constructor.
    pub fn new(
        account_id:       Uuid,
        credential_type:  CredentialType,
        encrypted_value:  String,
        encryption_key_id: String,
        iv_hex:           String,
        expires_at:       Option<DateTime<Utc>>,
    ) -> Self {
        let now = Utc::now();
        Self {
            uuid: Uuid::new_v4(),
            account_id,
            credential_type,
            encrypted_value,
            encryption_key_id,
            iv_hex,
            expires_at,
            is_active:  true,
            created_at: now,
            updated_at: now,
        }
    }

    /// Rehydrates a Credential from the database without re-running construction logic.
    pub fn rehydrate(
        uuid:             Uuid,
        account_id:       Uuid,
        credential_type:  CredentialType,
        encrypted_value:  String,
        encryption_key_id: String,
        iv_hex:           String,
        expires_at:       Option<DateTime<Utc>>,
        is_active:        bool,
        created_at:       DateTime<Utc>,
        updated_at:       DateTime<Utc>,
    ) -> Self {
        Self {
            uuid, account_id, credential_type, encrypted_value,
            encryption_key_id, iv_hex, expires_at, is_active, created_at, updated_at,
        }
    }

    /// Returns true if the credential has passed its expiry time.
    /// A credential without an expiry is considered non-expiring (never expired).
    pub fn is_expired(&self) -> bool {
        self.expires_at.map(|exp| Utc::now() > exp).unwrap_or(false)
    }

    /// Deactivates the credential. Called when the platform reports the token
    /// is revoked or when the user manually rotates their credentials.
    pub fn deactivate(&mut self) {
        self.is_active  = false;
        self.updated_at = Utc::now();
    }

    pub fn uuid(&self)              -> Uuid               { self.uuid }
    pub fn account_id(&self)        -> Uuid               { self.account_id }
    pub fn credential_type(&self)   -> &CredentialType    { &self.credential_type }
    pub fn encrypted_value(&self)   -> &str               { &self.encrypted_value }
    pub fn encryption_key_id(&self) -> &str               { &self.encryption_key_id }
    pub fn iv_hex(&self)            -> &str               { &self.iv_hex }
    pub fn expires_at(&self)        -> Option<DateTime<Utc>> { self.expires_at }
    pub fn is_active(&self)         -> bool               { self.is_active }
    pub fn created_at(&self)        -> DateTime<Utc>      { self.created_at }
}