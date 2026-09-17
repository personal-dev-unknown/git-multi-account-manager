// crates/gm_domain/src/ssh/value_objects/key_type.rs
//
// The SSH key algorithm type. Ed25519 is the recommended default because it
// produces smaller keys with equivalent security to RSA-3072 and is faster
// to generate and verify. RSA is kept for compatibility with older systems
// that pre-date Ed25519 support (OpenSSH added it in 6.5, released 2014).
// ECDSA is included for completeness but Ed25519 is preferred over ECDSA
// because Ed25519 is not vulnerable to weak random number generation.

use gm_shared::models::ssh_key::KeyType as SharedKeyType;

/// The SSH key algorithm used for a key pair.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum KeyType {
    /// Ed25519 elliptic curve — recommended for all new keys.
    Ed25519,
    /// RSA with the specified key size in bits. Accepted sizes: 2048, 3072, 4096.
    /// Anything below 2048 is rejected at construction time.
    Rsa(u32),
    /// ECDSA using the NIST P-256 curve.
    Ecdsa,
}

impl KeyType {
    /// Returns the string passed to ssh-keygen's -t flag.
    pub fn as_keygen_type(&self) -> &'static str {
        match self {
            KeyType::Ed25519   => "ed25519",
            KeyType::Rsa(_)    => "rsa",
            KeyType::Ecdsa     => "ecdsa",
        }
    }

    /// Returns the bits argument for ssh-keygen's -b flag.
    /// Returns None for key types where bit size is fixed (Ed25519, ECDSA).
    pub fn bit_size(&self) -> Option<u32> {
        match self {
            KeyType::Rsa(bits) => Some(*bits),
            _                  => None,
        }
    }

    /// Validates that an RSA key size is acceptable.
    /// Sizes below 2048 are cryptographically weak; sizes above 4096 are
    /// impractical without measurable security benefit.
    pub fn is_valid_rsa_size(bits: u32) -> bool {
        matches!(bits, 2048 | 3072 | 4096)
    }

    pub fn to_shared(&self) -> SharedKeyType {
        match self {
            KeyType::Ed25519 => SharedKeyType::Ed25519,
            KeyType::Rsa(_)  => SharedKeyType::Rsa,  // bit size lives in SshKeyDto.key_size_bits
            KeyType::Ecdsa   => SharedKeyType::Ecdsa,
        }
    }

    /// Returns the RSA bit size for populating SshKeyDto.key_size_bits.
    /// Returns None for fixed-size key types (Ed25519, ECDSA).
    pub fn rsa_bits(&self) -> Option<u16> {
        match self {
            KeyType::Rsa(bits) => Some(*bits as u16),
            _                  => None,
        }
    }

    pub fn from_shared(shared: &SharedKeyType) -> Self {
        match shared {
            SharedKeyType::Ed25519 => KeyType::Ed25519,
            SharedKeyType::Rsa     => KeyType::Rsa(4096), // default when size not in DTO
            SharedKeyType::Ecdsa   => KeyType::Ecdsa,
            SharedKeyType::Dsa     => KeyType::Ed25519,   // DSA is deprecated; treat as Ed25519
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "ed25519" => Some(KeyType::Ed25519),
            "rsa"     => Some(KeyType::Rsa(4096)),
            "ecdsa"   => Some(KeyType::Ecdsa),
            _         => None,
        }
    }
}

impl std::fmt::Display for KeyType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            KeyType::Ed25519      => write!(f, "ed25519"),
            KeyType::Rsa(bits)    => write!(f, "rsa-{bits}"),
            KeyType::Ecdsa        => write!(f, "ecdsa"),
        }
    }
}