pub mod auth_strategy_resolver;
pub mod cancellation;
pub mod clone_fallback_engine;
pub mod clone_url_resolver;
pub mod credential_service;
pub mod credential_vault;
pub mod permission_checker;

pub use auth_strategy_resolver::{AuthStrategy, AuthStrategyResolver, OperationKind, ResolvedStrategy};
pub use cancellation::{Cancelled, install_signal_handler, kill_current_child, register_child_killer};
pub use clone_fallback_engine::{CloneAttempt, CloneFallbackEngine};
pub use clone_url_resolver::CloneUrlResolver;
pub use credential_service::CredentialService;
pub use credential_vault::{CredentialVault, EncryptedValue};
pub use permission_checker::PermissionChecker;