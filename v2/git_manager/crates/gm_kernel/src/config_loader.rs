// crates/gm_kernel/src/config_loader.rs
//
// ── Purpose ───────────────────────────────────────────────────────────────────
// Locates, reads, validates, and provides the GitZyrix persistent application
// configuration. This is the PRIMARY configuration source for production
// deployments.
//
// Responsibilities of this module:
//   - Detect the OS and resolve the GitZyrix configuration directory.
//   - Read database.json and return the parsed configuration.
//   - Write database.json when a setup process (CLI wizard, Desktop GUI,
//     Web endpoint) has collected the user's choices.
//   - Expose helpers so each interface can construct configuration without
//     knowing the underlying directory layout.
//
// Responsibilities NOT belonging here:
//   - Interactive terminal prompts  → CLI setup wizard
//   - GUI configuration screens     → Desktop Tauri commands
//   - HTTP configuration endpoints  → Web server routes
//
// ── Configuration directory layout ───────────────────────────────────────────
//
//   Linux:   ~/.gitzyrix/
//                config/
//                    database.json
//                gitzyrix.db          (SQLite — when selected)
//
//   Windows: %APPDATA%\GitZyrix\
//                config\
//                    database.json
//                gitzyrix.db
//
//   macOS:   ~/Library/Application Support/GitZyrix/
//                config/
//                    database.json
//                gitzyrix.db
//
// ── Integration ───────────────────────────────────────────────────────────────
// Called from AppConfig::from_env() in bootstrap.rs.
// The config_loader is consulted FIRST (production priority).
// If database.json is absent, from_env() falls back to environment variables
// (.env / developer local configuration).

use std::path::PathBuf;
use std::fs;

use serde::{Deserialize, Serialize};

// ─────────────────────────────────────────────────────────────────────────────
// Public types
// ─────────────────────────────────────────────────────────────────────────────

/// The database backend configured for this GitZyrix installation.
///
/// Serialised as `{"db_type": "sqlite", "path": "..."}` or
/// `{"db_type": "mysql", "url": "..."}` inside database.json.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "db_type", rename_all = "lowercase")]
pub enum DatabaseConfig {
    /// SQLite — single-file embedded database, no external server required.
    /// Recommended default for all user installations.
    Sqlite {
        /// Absolute path to the `.db` file (e.g. `~/.gitzyrix/gitzyrix.db`).
        path: String,
    },

    /// MySQL / MariaDB — requires an external database server.
    /// `url` is the full sqlx connection string:
    ///   `mysql://username:password@host:port/database`
    Mysql {
        url: String,
    },
}

/// Top-level structure of `database.json`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlobalConfig {
    pub database: DatabaseConfig,
}

// ─────────────────────────────────────────────────────────────────────────────
// Directory and path helpers
// ─────────────────────────────────────────────────────────────────────────────

/// Root GitZyrix application directory for the current OS user.
///
/// - Linux:   `~/.gitzyrix`
/// - Windows: `%APPDATA%\GitZyrix`
/// - macOS:   `~/Library/Application Support/GitZyrix`
pub fn get_app_dir() -> PathBuf {
    #[cfg(target_os = "linux")]
    {
        dirs_next::home_dir()
            .unwrap_or_else(|| PathBuf::from("/tmp"))
            .join(".gitzyrix")
    }
    #[cfg(target_os = "windows")]
    {
        dirs_next::data_local_dir()
            .unwrap_or_else(|| PathBuf::from(r"C:\Users\Default\AppData\Local"))
            .join("GitZyrix")
    }
    #[cfg(not(any(target_os = "linux", target_os = "windows")))]
    {
        // macOS: ~/Library/Application Support/GitZyrix
        dirs_next::config_dir()
            .unwrap_or_else(|| PathBuf::from("/tmp"))
            .join("GitZyrix")
    }
}

/// `config/` subdirectory — where `database.json` lives.
pub fn get_config_dir() -> PathBuf {
    get_app_dir().join("config")
}

/// Full path to `database.json`.
pub fn get_config_file() -> PathBuf {
    get_config_dir().join("database.json")
}

/// Default SQLite database file path for this installation.
///
/// Returns `~/.gitzyrix/gitzyrix.db` on Linux (and the OS-equivalent on
/// other platforms). Callers should pass this to `DatabaseConfig::Sqlite`
/// when the user selects (or defaults to) SQLite.
pub fn get_sqlite_db_path() -> PathBuf {
    get_app_dir().join("gitzyrix.db")
}

// ─────────────────────────────────────────────────────────────────────────────
// Load / Save
// ─────────────────────────────────────────────────────────────────────────────

/// Attempt to load the persisted GitZyrix configuration.
///
/// Returns `Some(GlobalConfig)` if `database.json` exists and is valid.
/// Returns `None` if the file is absent (first run / not yet configured).
///
/// # Errors (non-fatal)
/// If the file exists but cannot be parsed, a warning is logged and `None` is
/// returned so the caller can fall back gracefully. The corrupted file is not
/// deleted automatically.
pub fn load() -> Option<GlobalConfig> {
    let config_file = get_config_file();

    if !config_file.exists() {
        return None;
    }

    let content = match fs::read_to_string(&config_file) {
        Ok(s)  => s,
        Err(e) => {
            tracing::warn!(
                path = %config_file.display(),
                error = %e,
                "GitZyrix: could not read database.json"
            );
            return None;
        }
    };

    match serde_json::from_str::<GlobalConfig>(&content) {
        Ok(cfg) => {
            tracing::info!(
                path = %config_file.display(),
                "GitZyrix: database configuration loaded"
            );
            Some(cfg)
        }
        Err(e) => {
            tracing::warn!(
                path = %config_file.display(),
                error = %e,
                "GitZyrix: database.json could not be parsed — \
                 run setup again to regenerate it"
            );
            None
        }
    }
}

/// Injects the loaded configuration into the environment variables.
/// This allows the existing initialization flow to use `std::env::var` 
/// seamlessly without knowing where the configuration came from.
pub fn inject_into_env() {
    // If the environment already contains database configuration (e.g. from .env or explicit shell variables),
    // we do not overwrite it. This preserves developer workflows while making database.json the primary 
    // source of truth in production environments.
    if std::env::var("GIT_MANAGER_DB_URL").is_ok() || std::env::var("GIT_ZYRIX_DB_URL").is_ok() || 
       std::env::var("GIT_MANAGER_SQLITE_PATH").is_ok() || std::env::var("GIT_ZYRIX_SQLITE_PATH").is_ok() {
        return;
    }

    if let Some(config) = load() {
        match config.database {
            DatabaseConfig::Mysql { url } => {
                std::env::set_var("GIT_MANAGER_DB_URL", url);
            }
            DatabaseConfig::Sqlite { path } => {
                std::env::set_var("GIT_MANAGER_SQLITE_PATH", path);
            }
        }
    }
}

/// Persist a `GlobalConfig` to `database.json`.
///
/// Creates the `config/` directory if it does not exist.
/// Called by each interface's setup flow after the user has provided
/// their database choice and credentials.
///
/// # Errors
/// Returns `Err` if the directory cannot be created or the file cannot be
/// written. The caller is responsible for surfacing this to the user.
pub fn save(config: &GlobalConfig) -> Result<(), String> {
    let config_dir  = get_config_dir();
    let config_file = get_config_file();

    fs::create_dir_all(&config_dir).map_err(|e| {
        format!(
            "GitZyrix: failed to create config directory {}: {e}",
            config_dir.display()
        )
    })?;

    let json = serde_json::to_string_pretty(config).map_err(|e| {
        format!("GitZyrix: failed to serialise configuration: {e}")
    })?;

    fs::write(&config_file, json).map_err(|e| {
        format!(
            "GitZyrix: failed to write {}: {e}",
            config_file.display()
        )
    })?;

    tracing::info!(
        path = %config_file.display(),
        "GitZyrix: configuration saved"
    );

    Ok(())
}

/// Returns `true` if `database.json` already exists for this user.
///
/// Use this to decide whether to show a first-time setup screen.
pub fn is_configured() -> bool {
    get_config_file().exists()
}

// ─────────────────────────────────────────────────────────────────────────────
// Convenience constructors for each interface's setup flow
// ─────────────────────────────────────────────────────────────────────────────

/// Build a `GlobalConfig` for the SQLite backend.
///
/// Passing `None` uses the default platform path (`~/.gitzyrix/gitzyrix.db`).
/// Interface setup flows call this and then call `save()`.
pub fn make_sqlite_config(path: Option<PathBuf>) -> GlobalConfig {
    let db_path = path.unwrap_or_else(get_sqlite_db_path);
    GlobalConfig {
        database: DatabaseConfig::Sqlite {
            path: db_path.to_string_lossy().into_owned(),
        },
    }
}

/// Build a `GlobalConfig` for the MySQL backend.
///
/// Constructs the connection URL from individual fields.
/// Interface setup flows call this and then call `save()`.
pub fn make_mysql_config(
    host:     &str,
    port:     u16,
    username: &str,
    password: &str,
    database: &str,
) -> GlobalConfig {
    let url = format!("mysql://{username}:{password}@{host}:{port}/{database}");
    GlobalConfig {
        database: DatabaseConfig::Mysql { url },
    }
}
