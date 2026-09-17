use std::path::PathBuf;

use gm_kernel::config_loader;

pub fn ensure_database_configured() -> bool {
    if config_loader::is_configured() {
        return false;
    }

    if atty::is(atty::Stream::Stdin) {
        run_setup_wizard();
        true
    } else {
        false
    }
}

pub fn run_setup_wizard() {
    println!();
    println!("  ╔══════════════════════════════════════════════╗");
    println!("  ║     Git Zyrix — First-Time Database Setup    ║");
    println!("  ╚══════════════════════════════════════════════╝");
    println!();

    let choices = &[
        "SQLite  — embedded database, no server required (recommended)",
        "MySQL / MariaDB  — external database server",
    ];

    let selection = dialoguer::Select::new()
        .with_prompt("Select a database backend")
        .items(choices)
        .default(0)
        .interact()
        .unwrap_or(0);

    match selection {
        0 => setup_sqlite(),
        1 => setup_mysql(),
        _ => unreachable!(),
    }
}

fn setup_sqlite() {
    let default_path = config_loader::get_sqlite_db_path();

    let path_str: String = dialoguer::Input::new()
        .with_prompt("SQLite database path")
        .default(default_path.to_string_lossy().into_owned())
        .show_default(true)
        .interact_text()
        .unwrap_or_else(|_| default_path.to_string_lossy().into_owned());

    let path = PathBuf::from(path_str);

    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }

    let config = config_loader::make_sqlite_config(Some(path));
    match config_loader::save(&config) {
        Ok(()) => {
            println!("  ✓ SQLite database configured.");
            println!("  Run the command again to start using it.");
        }
        Err(e) => {
            eprintln!("  ✗ Failed to save configuration: {e}");
            std::process::exit(1);
        }
    }
}

fn setup_mysql() {
    let host: String = dialoguer::Input::new()
        .with_prompt("MySQL host")
        .default("localhost".to_string())
        .show_default(true)
        .interact_text()
        .unwrap_or_else(|_| "localhost".to_string());

    let port: String = dialoguer::Input::new()
        .with_prompt("Port")
        .default("3306".to_string())
        .show_default(true)
        .interact_text()
        .unwrap_or_else(|_| "3306".to_string());

    let user: String = dialoguer::Input::new()
        .with_prompt("Username")
        .interact_text()
        .unwrap_or_default();

    let password = dialoguer::Password::new()
        .with_prompt("Password")
        .with_confirmation("Confirm password", "Passwords do not match")
        .interact()
        .unwrap_or_default();

    let database: String = dialoguer::Input::new()
        .with_prompt("Database name")
        .default("git_manager".to_string())
        .show_default(true)
        .interact_text()
        .unwrap_or_else(|_| "git_manager".to_string());

    let port_u16: u16 = port.parse().unwrap_or(3306);

    let config = config_loader::make_mysql_config(&host, port_u16, &user, &password, &database);
    match config_loader::save(&config) {
        Ok(()) => {
            println!("  ✓ MySQL database configured.");
            println!("  Run the command again to start using it.");
        }
        Err(e) => {
            eprintln!("  ✗ Failed to save configuration: {e}");
            std::process::exit(1);
        }
    }
}
