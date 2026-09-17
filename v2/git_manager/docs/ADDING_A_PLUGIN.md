# Adding a New Platform Plugin

This guide walks through adding a new Git hosting platform plugin — for example, **Gitea** or **Codeberg** (which runs on Gitea). The plugin architecture means the SSH isolation, cloning, fallback engine, and all three UIs (CLI, Web, Desktop) pick up the new platform automatically once registered.

## Overview

If you just need SSH + clone to work for a self-hosted Gitea instance (no API repository listing), you can stop after **Step 0** — the existing `SelfHosted` variant already handles it. A full plugin is only needed when you want repository discovery via the platform's REST API.

## Step 0: Quick Path — Use SelfHosted (no plugin needed)

For ad-hoc Gitea instances without API listing, create an account with the built-in self-hosted type:

```
git-manager account add my-gitea \
    --platform self_hosted \
    --auth-method ssh
```

The existing `SelfHosted(String)` variant uses the domain you enter as the SSH host. Clone URLs are resolved via `PlatformType::from_hostname()` which detects `gitea.example.com` as `Custom(...)` or `SelfHosted(...)` depending on configuration. SSH key isolation, agent management, and connection testing all work out of the box.

## Step 1: Create the Crate

```bash
# Create from the workspace root (v2/git_manager)
mkdir -p crates/gm_plugin_gitea/src
```

**`crates/gm_plugin_gitea/Cargo.toml`:**

```toml
[package]
name        = "gm_plugin_gitea"
version     = "0.1.0"
edition     = "2021"
description = "Gitea provider plugin: RepositoryProvider + AuthProvider"

[dependencies]
gm_shared   = { path = "../gm_shared" }
gm_ports    = { path = "../gm_ports" }
gm_kernel   = { path = "../gm_kernel" }

reqwest     = { workspace = true }
tokio       = { workspace = true }
async-trait = { workspace = true }
serde       = { workspace = true }
serde_json  = { workspace = true }
uuid        = { workspace = true }
chrono      = { workspace = true }
tracing     = { workspace = true }
```

## Step 2: Add the Crate to the Workspace

In `Cargo.toml` (workspace root), add to `[workspace.members]`:

```toml
"crates/gm_plugin_gitea",
```

## Step 3: Write the Plugin Code

### `src/lib.rs` — Crate root

```rust
pub mod auth;
pub mod auth_provider;
pub mod client;
pub mod plugin;
pub mod repository_provider;

pub use auth_provider::GiteaAuthProvider;
pub use plugin::GiteaPlugin;
pub use repository_provider::GiteaRepositoryProvider;

pub fn create_plugin() -> GiteaPlugin {
    GiteaPlugin::new()
}
```

### `src/plugin.rs` — Plugin lifecycle

```rust
use std::sync::Arc;
use gm_kernel::{
    contracts::plugin::{EventSubscription, Plugin, PluginMetadata},
    contracts::provider::ProviderPlugin,
    security::CredentialService,
    service_registry::ServiceRegistry,
};
use gm_shared::errors::PluginError;
use gm_shared::models::platform::PlatformType;
use crate::{auth_provider::GiteaAuthProvider, client::GiteaClient, repository_provider::GiteaRepositoryProvider};

pub struct GiteaPlugin {
    metadata: PluginMetadata,
    client:   Arc<GiteaClient>,
}

impl GiteaPlugin {
    pub fn new() -> Self {
        Self {
            metadata: PluginMetadata {
                name:               "gm_plugin_gitea".to_string(),
                version:            env!("CARGO_PKG_VERSION").to_string(),
                description:        "Gitea repository and authentication provider".to_string(),
                dependencies:       vec![],
                min_kernel_version: "1.0.0".to_string(),
                load_priority:      10,
            },
            client: Arc::new(GiteaClient::new()),
        }
    }
}

impl Plugin for GiteaPlugin {
    fn metadata(&self) -> &PluginMetadata { &self.metadata }

    fn on_load(&self, registry: Arc<ServiceRegistry>) -> Result<(), PluginError> {
        let creds = registry.get::<CredentialService>().ok_or_else(|| {
            PluginError::RegistrationFailed {
                name: self.metadata.name.clone(),
                reason: "CredentialService not in registry".to_string(),
            }
        })?;

        let repo_provider = Arc::new(GiteaRepositoryProvider::new(Arc::clone(&self.client), Arc::clone(&creds)));
        let auth_provider = Arc::new(GiteaAuthProvider::new(Arc::clone(&self.client), Arc::clone(&creds)));

        registry.register::<GiteaRepositoryProvider>(repo_provider);
        registry.register::<GiteaAuthProvider>(auth_provider);
        Ok(())
    }

    fn on_unload(&self) -> Result<(), PluginError> { Ok(()) }
    fn get_event_subscriptions(&self) -> Vec<EventSubscription> { vec![] }
}

impl ProviderPlugin for GiteaPlugin {
    fn platform_type(&self) -> PlatformType { PlatformType::Gitea }
}
```

### `src/client.rs` — HTTP API client

```rust
use std::sync::Arc;
use reqwest::Client;
use serde::Deserialize;
use gm_shared::errors::GitManagerError;

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct GiteaRepoResponse {
    id:       i64,
    name:     String,
    full_name: String,
    description: Option<String>,
    private:  bool,
    fork:     bool,
    archived: bool,
    html_url: String,
    ssh_url:  String,
    default_branch: String,
    language: Option<String>,
}

impl GiteaRepoResponse {
    fn to_remote_info(&self) -> gm_ports::outbound::repository_provider::RemoteRepositoryInfo {
        gm_ports::outbound::repository_provider::RemoteRepositoryInfo {
            full_name:        self.full_name.clone(),
            name:             self.name.clone(),
            description:      self.description.clone(),
            clone_url_ssh:    self.ssh_url.clone(),
            clone_url_https:  self.html_url.clone().trim_end_matches('/').to_string() + ".git",
            default_branch:   self.default_branch.clone(),
            is_private:       self.private,
            is_forked:        self.fork,
            is_archived:      self.archived,
            stargazers:       0,
            forks:            0,
            open_issues:      0,
            primary_language: self.language.clone(),
        }
    }
}

pub struct GiteaClient {
    http: Client,
}

impl GiteaClient {
    pub fn new() -> Self {
        Self {
            http: Client::builder()
                .user_agent("git-manager/0.1")
                .build()
                .expect("reqwest Client::new"),
        }
    }

    // Gitea API uses the base URL configured in the account's platform settings.
    pub async fn list_repositories(
        &self, token: &str, base_url: &str, page: u32, per_page: u32,
    ) -> Result<Vec<GiteaRepoResponse>, GitManagerError> {
        let url = format!("{}/api/v1/user/repos?page={}&limit={}", base_url.trim_end_matches('/'), page, per_page);
        let resp = self.http.get(&url)
            .header("Authorization", format!("token {token}"))
            .send().await
            .map_err(|e| GitManagerError::Other(e.to_string()))?;
        let repos: Vec<GiteaRepoResponse> = resp.json().await
            .map_err(|e| GitManagerError::Other(e.to_string()))?;
        Ok(repos)
    }

    pub async fn get_repository(
        &self, token: &str, base_url: &str, full_name: &str,
    ) -> Result<GiteaRepoResponse, GitManagerError> {
        let url = format!("{}/api/v1/repos/{}", base_url.trim_end_matches('/'), full_name);
        let resp = self.http.get(&url)
            .header("Authorization", format!("token {token}"))
            .send().await
            .map_err(|e| GitManagerError::Other(e.to_string()))?;
        let repo: GiteaRepoResponse = resp.json().await
            .map_err(|e| GitManagerError::Other(e.to_string()))?;
        Ok(repo)
    }

    pub async fn search_repositories(
        &self, token: &str, base_url: &str, query: &str, limit: u32,
    ) -> Result<Vec<GiteaRepoResponse>, GitManagerError> {
        let url = format!("{}/api/v1/repos/search?q={}&limit={}", base_url.trim_end_matches('/'), query, limit);
        let resp = self.http.get(&url)
            .header("Authorization", format!("token {token}"))
            .send().await
            .map_err(|e| GitManagerError::Other(e.to_string()))?;
        let body: serde_json::Value = resp.json().await
            .map_err(|e| GitManagerError::Other(e.to_string()))?;
        let repos: Vec<GiteaRepoResponse> = serde_json::from_value(body["data"].clone())
            .map_err(|e| GitManagerError::Other(e.to_string()))?;
        Ok(repos)
    }

    pub async fn validate_token(&self, token: &str, base_url: &str) -> Result<gm_ports::outbound::auth_provider::AuthResult, GitManagerError> {
        let url = format!("{}/api/v1/user", base_url.trim_end_matches('/'));
        let resp = self.http.get(&url)
            .header("Authorization", format!("token {token}"))
            .send().await
            .map_err(|e| GitManagerError::Other(e.to_string()))?;
        match resp.status().is_success() {
            true  => {
                let body: serde_json::Value = resp.json().await.unwrap_or_default();
                Ok(gm_ports::outbound::auth_provider::AuthResult {
                    valid:         true,
                    username:      body["login"].as_str().map(String::from),
                    display_name:  body["full_name"].as_str().map(String::from),
                    email:         body["email"].as_str().map(String::from),
                })
            }
            false => Ok(gm_ports::outbound::auth_provider::AuthResult {
                valid: false, username: None, display_name: None, email: None,
            }),
        }
    }
}
```

### `src/repository_provider.rs` — Repository listing

```rust
use std::sync::Arc;
use async_trait::async_trait;
use uuid::Uuid;
use gm_kernel::security::CredentialService;
use gm_ports::outbound::repository_provider::{
    ForkRepositoryInput, PageCursor, RemoteRepositoryInfo, RepositoryProvider,
};
use gm_shared::errors::GitManagerError;
use gm_shared::models::platform::PlatformType;
use crate::client::GiteaClient;

pub struct GiteaRepositoryProvider {
    client: Arc<GiteaClient>,
    creds:  Arc<CredentialService>,
}

impl GiteaRepositoryProvider {
    pub fn new(client: Arc<GiteaClient>, creds: Arc<CredentialService>) -> Self {
        Self { client, creds }
    }

    async fn token_and_base_url(&self, account_uuid: Uuid) -> Result<(String, String), GitManagerError> {
        let token = self.creds.get_token(account_uuid).await?
            .ok_or_else(|| GitManagerError::Other(format!("no Gitea credential found for account {account_uuid}")))?;
        // Base URL comes from the platform record in the database.
        // For simplicity, you may store it as part of the account metadata
        // or look it up from the platforms table.
        let base_url = "https://codeberg.org".to_string(); // or retrieve from account metadata
        Ok((token, base_url))
    }
}

#[async_trait]
impl RepositoryProvider for GiteaRepositoryProvider {
    fn platform_type(&self) -> PlatformType { PlatformType::Gitea }

    async fn list_repositories(
        &self, account_uuid: Uuid, cursor: Option<PageCursor>,
    ) -> Result<(Vec<RemoteRepositoryInfo>, PageCursor), GitManagerError> {
        let (token, base_url) = self.token_and_base_url(account_uuid).await?;
        let page     = cursor.as_ref().map(|c| c.page).unwrap_or(1);
        let per_page = cursor.as_ref().map(|c| c.per_page).unwrap_or(50);
        let repos    = self.client.list_repositories(&token, &base_url, page, per_page).await?;
        let has_next = repos.len() as u32 >= per_page;
        let infos: Vec<RemoteRepositoryInfo> = repos.iter().map(|r| r.to_remote_info()).collect();
        Ok((infos, PageCursor { page: page + 1, per_page, has_next }))
    }

    async fn get_repository(
        &self, account_uuid: Uuid, full_name: &str,
    ) -> Result<RemoteRepositoryInfo, GitManagerError> {
        let (token, base_url) = self.token_and_base_url(account_uuid).await?;
        let repo = self.client.get_repository(&token, &base_url, full_name).await?;
        Ok(repo.to_remote_info())
    }

    async fn search_repositories(
        &self, account_uuid: Uuid, query: &str, limit: u32,
    ) -> Result<Vec<RemoteRepositoryInfo>, GitManagerError> {
        let (token, base_url) = self.token_and_base_url(account_uuid).await?;
        let repos = self.client.search_repositories(&token, &base_url, query, limit).await?;
        Ok(repos.iter().map(|r| r.to_remote_info()).collect())
    }

    async fn fork_repository(
        &self, account_uuid: Uuid, _input: ForkRepositoryInput,
    ) -> Result<RemoteRepositoryInfo, GitManagerError> {
        Err(GitManagerError::Other("Gitea fork via API not yet implemented".to_string()))
    }
}
```

### `src/auth_provider.rs` — Token validation

```rust
use std::sync::Arc;
use async_trait::async_trait;
use uuid::Uuid;
use gm_kernel::security::CredentialService;
use gm_ports::outbound::auth_provider::{AuthProvider, AuthResult, TokenRefreshResult};
use gm_shared::errors::GitManagerError;
use gm_shared::models::platform::PlatformType;
use crate::client::GiteaClient;

pub struct GiteaAuthProvider {
    client: Arc<GiteaClient>,
    creds:  Arc<CredentialService>,
}

impl GiteaAuthProvider {
    pub fn new(client: Arc<GiteaClient>, creds: Arc<CredentialService>) -> Self {
        Self { client, creds }
    }
}

#[async_trait]
impl AuthProvider for GiteaAuthProvider {
    fn platform_type(&self) -> PlatformType { PlatformType::Gitea }

    async fn validate_credential(
        &self, account_uuid: Uuid, credential: &str,
    ) -> Result<AuthResult, GitManagerError> {
        let base_url = "https://codeberg.org".to_string();
        self.client.validate_token(credential, &base_url).await
    }

    async fn refresh_token(
        &self, _account_uuid: Uuid, _refresh_token: &str,
    ) -> Result<TokenRefreshResult, GitManagerError> {
        Err(GitManagerError::Other("Gitea tokens cannot be refreshed via API".to_string()))
    }
}
```

### `src/auth.rs` — Token detection helper

```rust
/// Rough heuristic to detect Gitea access tokens.
pub fn looks_like_gitea_token(token: &str) -> bool {
    // Gitea tokens are 40-character hex strings.
    token.len() == 40 && token.chars().all(|c| c.is_ascii_hexdigit())
}
```

## Step 4: Register the Platform Type

### Add a variant to `PlatformType`

**`crates/gm_shared/src/models/platform.rs`:**

Add `Gitea` to the enum (alphabetically before `GitHub` or after `GitLab`):

```rust
pub enum PlatformType {
    // ... existing variants ...
    Gitea,
    GitHub,
    // ...
}
```

Then add matching arms to every `match` block in that file:

- `slug()` → `"gitea"`
- `default_ssh_host()` → `Some("codeberg.org")` (or `None` if fully dynamic)
- `from_hostname()` → add `if h.contains("codeberg.org") || h.contains("gitea") { return PlatformType::Gitea; }`
- `Display` → `write!(f, "Gitea")`
- All `#[cfg(test)]` tests

### Register the platform UUID

UUIDs follow the pattern `00000000-00XX-0000-0000-000000000001`. Pick the next available number:

| Platform    | UUID |
|-------------|------|
| GitHub      | `00000000-0001-...` |
| GitLab      | `00000000-0002-...` |
| Bitbucket   | `00000000-0003-...` |
| Azure DevOps| `00000000-0004-...` |
| SourceForge | `00000000-0005-...` |
| Self-Hosted | `00000000-0006-...` |
| Cloud Stor. | `00000000-0007-...` |
| Local Path  | `00000000-0008-...` |
| Custom      | `00000000-0009-...` |
| **Gitea**   | **`00000000-000A-0000-0000-000000000001`** ← next |

Add to all of these locations:

1. **`crates/gm_adapters/src/persistence/migrations/20240101000001_seed_platforms.sql`**
   ```sql
   INSERT IGNORE INTO platforms (uuid, name, display_name, api_base_url, ssh_host, ...)
       VALUES ('00000000-000A-0000-0000-000000000001', 'gitea', 'Gitea', 'https://codeberg.org/api/v1', 'codeberg.org');
   ```

2. **`crates/gm_interface_cli/src/commands/account.rs`** — `platform_uuid_for()`:
   ```rust
   PlatformType::Gitea => Uuid::parse_str("00000000-000A-0000-0000-000000000001").unwrap(),
   ```

3. **`apps/cli/src/platform_helpers.rs`** — `platform_info()` and `platform_display_name()`:
   ```rust
   "00000000-000A-0000-0000-000000000001" => ("gitea", "codeberg.org", 22),
   // ...
   "00000000-000A-0000-0000-000000000001" => Some("Gitea".to_string()),
   ```

4. **`apps/web/src/main.rs`** — same `platform_info()` and `platform_display_name()` functions.

5. **`apps/desktop/src/main.rs`** — same `platform_info()` and `platform_display_name()` functions.

6. **`crates/gm_interface_web/src/routes/mod.rs`** — `platform_uuid_for()`:
   ```rust
   "gitea" => Uuid::parse_str("00000000-000A-0000-0000-000000000001").unwrap(),
   ```

7. **`crates/gm_interface_cli/src/commands/account.rs`** — `parse_platform()` function (add `"gitea" => PlatformType::Gitea`).

## Step 5: Register in All Binary Entry Points

### `apps/cli/src/main.rs`

Add to the plugins vec:

```rust
Box::new(gm_plugin_gitea::create_plugin()),
```

Add the `insert_provider!` macro call:

```rust
insert_provider!("00000000-000A-0000-0000-000000000001", gm_plugin_gitea::GiteaRepositoryProvider);
```

### `apps/web/src/main.rs`

Same two additions (plugins vec + insert_provider). Also add to `platform_id_from_url()` or equivalent URL-to-UUID mapping if it exists.

### `apps/desktop/src/main.rs`

Same two additions.

## Step 6: Add Hostname Detection in URL Parsing

If the CLI has a `platform_id_from_url()` function (in `apps/cli/src/cli_services.rs`), add a match for `"codeberg.org"` or your Gitea instance hostname:

```rust
fn platform_id_from_url(url: &str) -> Uuid {
    // ... existing matches ...
    if url.contains("codeberg.org") || url.contains("gitea") {
        return Uuid::parse_str("00000000-000A-0000-0000-000000000001").unwrap();
    }
    // ... fallback ...
}
```

Also add the detection in `PlatformType::from_hostname()` (step 4 above) so clone URL resolution picks it up automatically.

## Step 7: Build and Verify

```bash
# Build the workspace to check for compilation errors
cd v2/git_manager
cargo check --workspace

# If everything compiles, run tests
cargo test --workspace
```

The new platform will now appear in:
- `git-manager account add --help` (platform choices)
- Account listing (with display name from platform seed data)
- Repository listing via the Gitea API
- SSH key generation, agent management, and connection testing
- Clone URL resolution (SSH isolation with `GIT_SSH_COMMAND`)
- Clone fallback engine (SSH → anonymous)
- All three frontends (CLI, Web, Desktop)

## What Each Piece Handles

| Component | What it does for the new platform |
|-----------|-----------------------------------|
| **`PlatformType::Gitea`** | Enum variant used for type-level dispatch |
| **Seed migration** | Platform record in the database (API base URL, SSH host, feature flags) |
| **`Plugin::on_load()`** | Registers `GiteaRepositoryProvider` and `GiteaAuthProvider` in the kernel |
| **`RepositoryProvider`** | Lists/search/gets repos via Gitea REST API |
| **`AuthProvider`** | Validates PAT tokens against Gitea API |
| **SSH service** | Works automatically — no changes needed (uses `~/.ssh/config` Host blocks) |
| **Clone fallback** | Works automatically — uses `GIT_SSH_COMMAND` env var with per-key isolation |
| **CLI/Web/Desktop** | Display the platform name, SSH host, and allow account management |

## Alternative Approach: Generic Gitea via SelfHosted

If you don't need API-based repository listing, you can skip the plugin entirely:

```
git-manager account add my-gitea \
    --platform self_hosted \
    --auth-method ssh
```

The `SelfHosted(String)` variant uses the domain as the SSH host. SSH key generation, connection testing (ssh -T), and cloning all work. The only missing feature is listing remote repos through the UI — since there's no REST API integration, the "list remote repositories" command won't be available for that account.
