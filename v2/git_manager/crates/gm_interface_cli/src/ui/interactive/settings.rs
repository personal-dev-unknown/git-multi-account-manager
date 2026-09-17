use std::path::PathBuf;

use gm_kernel::config_loader;
use crate::ui::colors;
use super::common::{FlowError, press_enter, section};

pub fn flow_settings() -> Result<(), FlowError> {
    loop {
        section("Settings");

        let show_current = || -> String {
            match config_loader::load() {
                Some(cfg) => match cfg.database {
                    config_loader::DatabaseConfig::Sqlite { path } => {
                        format!("SQLite  —  {path}")
                    }
                    config_loader::DatabaseConfig::Mysql { url } => {
                        format!("MySQL  —  {url}")
                    }
                },
                None => "Not configured".to_string(),
            }
        };

        println!("  Current database:  {}", colors::bold(&show_current()));
        println!();

        let items = &[
            "Technical Settings",
            "Back to Main Menu",
        ];

        let sel = dialoguer::Select::new()
            .with_prompt("Settings")
            .items(items)
            .default(0)
            .interact_opt()
            .map_err(|e| FlowError::Service(
                gm_shared::errors::GitManagerError::Other(e.to_string())
            ))?
            .unwrap_or(1);

        match sel {
            0 => flow_technical_settings()?,
            1 => return Ok(()),
            _ => unreachable!(),
        }
    }
}

fn flow_technical_settings() -> Result<(), FlowError> {
    loop {
        section("Technical Settings");

        let items = &[
            "Database — view / change backend",
            "Back to Settings",
        ];

        let sel = dialoguer::Select::new()
            .with_prompt("Technical Settings")
            .items(items)
            .default(0)
            .interact_opt()
            .map_err(|e| FlowError::Service(
                gm_shared::errors::GitManagerError::Other(e.to_string())
            ))?
            .unwrap_or(1);

        match sel {
            0 => flow_database_settings()?,
            1 => return Ok(()),
            _ => unreachable!(),
        }
    }
}

fn flow_database_settings() -> Result<(), FlowError> {
    loop {
        section("Database Settings");

        let show_current = || -> String {
            match config_loader::load() {
                Some(cfg) => match cfg.database {
                    config_loader::DatabaseConfig::Sqlite { ref path } => {
                        format!("SQLite  —  {path}")
                    }
                    config_loader::DatabaseConfig::Mysql { ref url } => {
                        format!("MySQL  —  {url}")
                    }
                },
                None => "Not configured".to_string(),
            }
        };

        println!("  Current:  {}", colors::bold(&show_current()));
        println!();

        let items = &[
            "Change database type",
            "Edit connection",
            "Delete database configuration",
            "Back",
        ];

        let sel = dialoguer::Select::new()
            .with_prompt("Database Settings")
            .items(items)
            .default(0)
            .interact_opt()
            .map_err(|e| FlowError::Service(
                gm_shared::errors::GitManagerError::Other(e.to_string())
            ))?
            .unwrap_or(3);

        match sel {
            0 => change_database_type()?,
            1 => edit_connection()?,
            2 => delete_config()?,
            3 => return Ok(()),
            _ => unreachable!(),
        }
    }
}

fn change_database_type() -> Result<(), FlowError> {
    section("Change Database Type");

    let choices = &[
        "SQLite  — embedded database, no server required",
        "MySQL / MariaDB  — external database server",
    ];

    let sel = dialoguer::Select::new()
        .with_prompt("Select database backend")
        .items(choices)
        .default(0)
        .interact_opt()
        .map_err(|e| FlowError::Service(
            gm_shared::errors::GitManagerError::Other(e.to_string())
        ))?
        .unwrap_or(0);

    match sel {
        0 => {
            let default_path = config_loader::get_sqlite_db_path();
            let path_str: String = dialoguer::Input::new()
                .with_prompt("SQLite database path")
                .default(default_path.to_string_lossy().into_owned())
                .show_default(true)
                .interact_text()
                .map_err(|e| FlowError::Service(
                    gm_shared::errors::GitManagerError::Other(e.to_string())
                ))?;
            let path = PathBuf::from(&path_str);
            if let Some(parent) = path.parent() {
                let _ = std::fs::create_dir_all(parent);
            }
            let config = config_loader::make_sqlite_config(Some(path));
            config_loader::save(&config).map_err(|e| {
                FlowError::Service(gm_shared::errors::GitManagerError::Other(e))
            })?;
            println!("  ✓ Database changed to SQLite.");
        }
        1 => {
            let host: String = dialoguer::Input::new()
                .with_prompt("MySQL host")
                .default("localhost".to_string())
                .show_default(true)
                .interact_text()
                .map_err(|e| FlowError::Service(
                    gm_shared::errors::GitManagerError::Other(e.to_string())
                ))?;
            let port: String = dialoguer::Input::new()
                .with_prompt("Port")
                .default("3306".to_string())
                .show_default(true)
                .interact_text()
                .map_err(|e| FlowError::Service(
                    gm_shared::errors::GitManagerError::Other(e.to_string())
                ))?;
            let user: String = dialoguer::Input::new()
                .with_prompt("Username")
                .interact_text()
                .map_err(|e| FlowError::Service(
                    gm_shared::errors::GitManagerError::Other(e.to_string())
                ))?;
            let password = dialoguer::Password::new()
                .with_prompt("Password")
                .with_confirmation("Confirm password", "Passwords do not match")
                .interact()
                .map_err(|e| FlowError::Service(
                    gm_shared::errors::GitManagerError::Other(e.to_string())
                ))?;
            let database: String = dialoguer::Input::new()
                .with_prompt("Database name")
                .default("git_manager".to_string())
                .show_default(true)
                .interact_text()
                .map_err(|e| FlowError::Service(
                    gm_shared::errors::GitManagerError::Other(e.to_string())
                ))?;
            let port_u16: u16 = port.parse().unwrap_or(3306);
            let config = config_loader::make_mysql_config(&host, port_u16, &user, &password, &database);
            config_loader::save(&config).map_err(|e| {
                FlowError::Service(gm_shared::errors::GitManagerError::Other(e))
            })?;
            println!("  ✓ Database changed to MySQL.");
        }
        _ => unreachable!(),
    }

    press_enter();
    Ok(())
}

fn edit_connection() -> Result<(), FlowError> {
    let current = config_loader::load();

    match current {
        Some(config_loader::GlobalConfig {
            database: config_loader::DatabaseConfig::Sqlite { path: current_path },
        }) => {
            section("Edit SQLite Path");

            let path_str: String = dialoguer::Input::new()
                .with_prompt("SQLite database path")
                .default(current_path.clone())
                .show_default(true)
                .interact_text()
                .map_err(|e| FlowError::Service(
                    gm_shared::errors::GitManagerError::Other(e.to_string())
                ))?;

            let path = PathBuf::from(&path_str);
            if let Some(parent) = path.parent() {
                let _ = std::fs::create_dir_all(parent);
            }

            let config = config_loader::make_sqlite_config(Some(path));
            config_loader::save(&config).map_err(|e| {
                FlowError::Service(gm_shared::errors::GitManagerError::Other(e))
            })?;
            println!("  ✓ SQLite path updated.");
        }
        Some(config_loader::GlobalConfig {
            database: config_loader::DatabaseConfig::Mysql { url: ref current_url },
        }) => {
            section("Edit MySQL Connection");

            let (cur_host, cur_port, cur_user, cur_pass, cur_db) = urlparse_url(current_url);

            let host: String = dialoguer::Input::new()
                .with_prompt("Host")
                .default(cur_host)
                .show_default(true)
                .interact_text()
                .map_err(|e| FlowError::Service(
                    gm_shared::errors::GitManagerError::Other(e.to_string())
                ))?;
            let port: String = dialoguer::Input::new()
                .with_prompt("Port")
                .default(cur_port)
                .show_default(true)
                .interact_text()
                .map_err(|e| FlowError::Service(
                    gm_shared::errors::GitManagerError::Other(e.to_string())
                ))?;
            let user: String = dialoguer::Input::new()
                .with_prompt("Username")
                .default(cur_user)
                .show_default(true)
                .interact_text()
                .map_err(|e| FlowError::Service(
                    gm_shared::errors::GitManagerError::Other(e.to_string())
                ))?;
            let password: String = dialoguer::Input::new()
                .with_prompt("Password (leave empty to keep current)")
                .default(cur_pass.clone())
                .show_default(true)
                .interact_text()
                .map_err(|e| FlowError::Service(
                    gm_shared::errors::GitManagerError::Other(e.to_string())
                ))?;
            let database: String = dialoguer::Input::new()
                .with_prompt("Database name")
                .default(cur_db)
                .show_default(true)
                .interact_text()
                .map_err(|e| FlowError::Service(
                    gm_shared::errors::GitManagerError::Other(e.to_string())
                ))?;

            let port_u16: u16 = port.parse().unwrap_or(3306);
            let config = config_loader::make_mysql_config(&host, port_u16, &user, &password, &database);
            config_loader::save(&config).map_err(|e| {
                FlowError::Service(gm_shared::errors::GitManagerError::Other(e))
            })?;
            println!("  ✓ MySQL connection updated.");
        }
        None => {
            println!("  No database configuration found.");
        }
    }

    press_enter();
    Ok(())
}

fn delete_config() -> Result<(), FlowError> {
    section("Delete Database Configuration");

    let config_file = config_loader::get_config_file();
    if !config_file.exists() {
        println!("  No database configuration to delete.");
        press_enter();
        return Ok(());
    }

    println!("  {}  WARNING: This will delete the database configuration!", colors::yellow("⚠"));
    println!("  The database file itself will NOT be deleted.");
    println!();

    let proceed = confirm("Are you sure you want to delete the configuration?")?;
    if !proceed {
        println!("  Deletion cancelled.");
        press_enter();
        return Ok(());
    }

    let double_check = confirm("Type 'yes' to confirm deletion")?;
    if !double_check {
        println!("  Deletion cancelled.");
        press_enter();
        return Ok(());
    }

    match std::fs::remove_file(&config_file) {
        Ok(()) => {
            println!("  ✓ Database configuration deleted.");
            println!("  Run the command again to set up a new database.");
        }
        Err(e) => {
            eprintln!("  ✗ Failed to delete configuration: {e}");
        }
    }

    press_enter();
    Ok(())
}

fn urlparse_url(url: &str) -> (String, String, String, String, String) {
    let mut host = "localhost".to_string();
    let mut port = "3306".to_string();
    let mut user = String::new();
    let mut password = String::new();
    let mut database = "git_manager".to_string();

    if let Some(rest) = url.strip_prefix("mysql://") {
        let auth_and_rest: Vec<&str> = rest.splitn(2, '@').collect();
        if auth_and_rest.len() == 2 {
            let auth: Vec<&str> = auth_and_rest[0].splitn(2, ':').collect();
            if auth.len() == 2 {
                user = auth[0].to_string();
                password = auth[1].to_string();
            } else {
                user = auth[0].to_string();
            }

            let host_and_db: Vec<&str> = auth_and_rest[1].splitn(2, '/').collect();
            if !host_and_db.is_empty() {
                let hp: Vec<&str> = host_and_db[0].splitn(2, ':').collect();
                host = hp[0].to_string();
                if hp.len() > 1 {
                    port = hp[1].to_string();
                }
            }
            if host_and_db.len() > 1 {
                database = host_and_db[1].to_string();
            }
        }
    }

    (host, port, user, password, database)
}

fn confirm(prompt: &str) -> Result<bool, FlowError> {
    dialoguer::Confirm::new()
        .with_prompt(prompt)
        .default(false)
        .interact()
        .map_err(|e| FlowError::Service(
            gm_shared::errors::GitManagerError::Other(e.to_string())
        ))
}
