// crates/gm_interface_cli/src/ui/interactive/ssh.rs
//
// ── Purpose ───────────────────────────────────────────────────────────────────
// Main menu options [6] and [7]:
//   [6] Test SSH connections
//   [7] Generate and Manage SSH keys and PATs
//
// The key management flow owns a second-level sub-menu:
//   [1] Generate new SSH key
//   [2] Setup Personal Access Token (PAT)
//   [3] Back
//
// SSH key generation is the most complex flow in the CLI because it may
// first need to CREATE an account if none exist.  That logic is delegated
// to common::create_account_interactive() — no duplication.

use gm_ports::inbound::commands::{GenerateSshKeyCommand, TestSshConnectionCommand};

use crate::services::CliServicesHandle;
use crate::ui::colors;
use super::common::{
    confirm, create_account_interactive, FlowResult, password, pick_account,
    press_enter, section, select_menu, spinner, step, typed_menu,
};

// ─────────────────────────────────────────────────────────────────────────────
// [6] Test SSH connections
// ─────────────────────────────────────────────────────────────────────────────

pub(super) async fn flow_test_ssh(handle: &CliServicesHandle) -> FlowResult {
    section("Test SSH Connection");

    let account = match pick_account(handle).await? {
        Some(a) => a,
        None    => return Ok(()),
    };

    println!();
    println!("  Testing: {} → {}",
        colors::bold(&account.alias),
        colors::dim(account.platform_name.as_deref().unwrap_or("platform")));

    let pb = spinner(format!("Testing SSH for '{}'…", account.alias));
    let result = handle.services().test_ssh_connection(TestSshConnectionCommand {
        account_uuid: account.uuid,
        timeout_ms:   Some(10_000),
    }).await;
    pb.finish_and_clear();

    match result {
        Ok(r) if r.success => {
            let who = r.username.as_deref().unwrap_or("(unknown)");
            println!("{} Connection succeeded — authenticated as {}",
                colors::success_prefix(), colors::bold(who));
        }
        Ok(r) => {
            println!("{} SSH connection failed.", colors::error_prefix());
            if let Some(err) = &r.error {
                println!("  Reason : {}", colors::dim(err));
            }
            println!();
            println!("  {} Common fixes:", colors::yellow("⚠"));
            println!("    {}  Add the public key to your {} SSH settings.",
                colors::dim("1."),
                account.platform_name.as_deref().unwrap_or("platform"));
            println!("    {}  Use option {} to regenerate the key pair.",
                colors::dim("2."), colors::bold("[7] → [1]"));
            println!("    {}  Verify port 22 is not blocked by a firewall.",
                colors::dim("3."));
        }
        Err(e) => println!("{} Test failed: {}", colors::error_prefix(), e),
    }

    press_enter();
    Ok(())
}

// ─────────────────────────────────────────────────────────────────────────────
// [7] Generate and Manage SSH keys and PATs — sub-menu
// ─────────────────────────────────────────────────────────────────────────────

pub(super) async fn flow_manage_keys(handle: &CliServicesHandle) -> FlowResult {
    section("Git Auth");

    // Sub-menu (typed numbers, matching v1)
    println!("  {}  Generate new SSH key",      colors::yellow("[1]"));
    println!("  {}  Setup Personal Access Token (PAT)", colors::yellow("[2]"));
    println!("  {}  Setup OAuth token",          colors::yellow("[3]"));
    println!("  {}  Store password",             colors::yellow("[4]"));
    println!("  {}  SSH validate (10 checks)",   colors::yellow("[5]"));
    println!("  {}  List SSH keys",              colors::yellow("[6]"));
    println!("  {}  ← Back to main menu",        colors::dim("[7]"));
    println!();

    let sel = typed_menu(7, 7)?; // default = 7 (Back)

    println!();
    match sel {
        0 => flow_generate_ssh_key(handle).await,
        1 => flow_setup_pat(handle).await,
        2 => flow_setup_oauth_token(handle).await,
        3 => flow_store_password(handle).await,
        4 => flow_ssh_validate(handle).await,
        5 => flow_list_ssh_keys(handle).await,
        6 => Ok(()), // Back
        _ => Ok(()),
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// SSH key generation — Step 1/2/3 wizard
// ─────────────────────────────────────────────────────────────────────────────

async fn flow_generate_ssh_key(handle: &CliServicesHandle) -> FlowResult {
    section("SSH Key Generation");

    // ── Step 1: Account ───────────────────────────────────────────────────────
    step(1, "Account Details");
    println!();

    let accounts = handle.services().list_accounts(None).await?;
    let account = if accounts.is_empty() {
        println!("{} No accounts yet — creating one first.", colors::info_prefix());
        println!();
        create_account_interactive(handle).await?
    } else {
        // Offer existing accounts + "create new"
        let mut labels: Vec<String> = accounts.iter()
            .map(|a| format!("  {:<20}  {:<14}  {}",
                a.alias,
                a.platform_name.as_deref().unwrap_or("?"),
                colors::dim(&a.username)))
            .collect();
        labels.push("  + Create a new account".to_string());

        let opts: Vec<&str> = labels.iter().map(|s| s.as_str()).collect();
        let idx = select_menu("Account", &opts)?;

        if idx == accounts.len() {
            println!();
            create_account_interactive(handle).await?
        } else {
            accounts[idx].clone()
        }
    };

    // ── Step 2: Key configuration ─────────────────────────────────────────────
    println!();
    step(2, "Key Configuration");
    println!();

    let key_type_idx = select_menu("Key type", &[
        "Ed25519  — recommended  (modern, fast, small key)",
        "RSA 4096 — legacy platforms that don't support Ed25519",
    ])?;
    let key_type = if key_type_idx == 0 { "ed25519" } else { "rsa" };

    let add_to_agent = confirm("Add key to SSH agent automatically after generation?", true)?;

    // ── Step 3: Generate ──────────────────────────────────────────────────────
    println!();
    step(3, "Generating");
    println!();
    println!("  Account  : {}", colors::bold(&account.alias));
    println!("  Platform : {}", account.platform_name.as_deref().unwrap_or("?"));
    println!("  Key type : {}", key_type);
    println!();

    let pb = spinner(format!("Generating {} key for '{}'…", key_type, account.alias));
    let result = handle.services().generate_ssh_key(GenerateSshKeyCommand {
        account_uuid: account.uuid,
        key_type:     key_type.to_string(),
        comment:      Some(account.email.clone()),
        passphrase:   None,
        add_to_agent,
    }).await;
    pb.finish_and_clear();

    match result {
        Ok(key) => {
            println!("{} SSH key generated for '{}'!",
                colors::success_prefix(), colors::bold(&account.alias));
            println!();
            println!("  Key name    : {}", colors::bold(&key.name));
            println!("  Fingerprint : {}", colors::dim(&key.fingerprint));
            println!("  Private key : {}", colors::dim(&key.private_key_path));
            println!();

            // Print the platform-specific URL where the user should add the key
            let settings_url = platform_ssh_settings_url(
                account.platform_name.as_deref().unwrap_or(""),
            );
            println!("  {} Copy this public key and add it to your {} SSH settings:",
                colors::info_prefix(),
                colors::bold(account.platform_name.as_deref().unwrap_or("platform")));
            if !settings_url.is_empty() {
                println!("  {} {}", colors::dim("URL:"), colors::cyan(settings_url));
            }
            println!();

            // Show the full public key so it can be copied
            println!("  {}", colors::dim(&"─".repeat(60)));
            println!("  {}", key.public_key);
            println!("  {}", colors::dim(&"─".repeat(60)));
            println!();
            println!("  {}  Then use option {} to verify the connection works.",
                colors::dim("→"), colors::bold("[6]"));
        }
        Err(e) => println!("{} Key generation failed: {}", colors::error_prefix(), e),
    }

    press_enter();
    Ok(())
}

// ─────────────────────────────────────────────────────────────────────────────
// PAT / App Password setup
// ─────────────────────────────────────────────────────────────────────────────

async fn flow_setup_pat(handle: &CliServicesHandle) -> FlowResult {
    section("Personal Access Token Setup");

    let account = match pick_account(handle).await? { Some(a) => a, None => return Ok(()) };

    println!("  Platform : {}",
        colors::bold(account.platform_name.as_deref().unwrap_or("unknown")));
    println!("  Account  : {}  ({})", account.alias, account.username);
    println!();
    println!("  Token format by platform:");
    println!("    {}  GitHub       →  {}",
        colors::dim("•"), colors::dim("ghp_..."));
    println!("    {}  GitLab       →  {}",
        colors::dim("•"), colors::dim("glpat-..."));
    println!("    {}  Bitbucket    →  {}",
        colors::dim("•"), colors::dim("username:app_password"));
    println!("    {}  Azure DevOps →  {}",
        colors::dim("•"), colors::dim("organization:token"));
    println!();

    let token = password("Paste PAT / App Password (hidden input)")?;
    if token.trim().is_empty() {
        println!("{} No token entered — cancelled.", colors::warn_prefix());
        press_enter();
        return Ok(());
    }

    let pb = spinner("Storing token in OS keychain…");
    let result = handle.services()
        .store_account_token(account.uuid, token.trim()).await;
    pb.finish_and_clear();

    match result {
        Ok(_) => println!("{} Token stored securely in the OS keychain.",
            colors::success_prefix()),
        Err(e) => println!("{} Failed to store token: {}", colors::error_prefix(), e),
    }

    press_enter();
    Ok(())
}

// ─────────────────────────────────────────────────────────────────────────────
// OAuth token setup
// ─────────────────────────────────────────────────────────────────────────────

async fn flow_setup_oauth_token(handle: &CliServicesHandle) -> FlowResult {
    section("OAuth Token Setup");

    let account = match pick_account(handle).await? { Some(a) => a, None => return Ok(()) };

    println!("  Platform : {}",
        colors::bold(account.platform_name.as_deref().unwrap_or("unknown")));
    println!("  Account  : {}  ({})", account.alias, account.username);
    println!();
    println!("  Paste the OAuth token generated from the platform.");
    println!();

    let token = password("Paste OAuth token (hidden input)")?;
    if token.trim().is_empty() {
        println!("{} No token entered — cancelled.", colors::warn_prefix());
        press_enter();
        return Ok(());
    }

    let pb = spinner("Storing OAuth token in OS keychain...");
    let result = handle.services().store_account_token(account.uuid, token.trim()).await;
    pb.finish_and_clear();

    match result {
        Ok(_) => println!("{} OAuth token stored securely.", colors::success_prefix()),
        Err(e) => println!("{} Failed: {}", colors::error_prefix(), e),
    }

    press_enter();
    Ok(())
}

// ─────────────────────────────────────────────────────────────────────────────
// Password storage
// ─────────────────────────────────────────────────────────────────────────────

async fn flow_store_password(handle: &CliServicesHandle) -> FlowResult {
    section("Password / Credential Storage");

    let account = match pick_account(handle).await? { Some(a) => a, None => return Ok(()) };

    println!("  Platform : {}",
        colors::bold(account.platform_name.as_deref().unwrap_or("unknown")));
    println!("  Account  : {}  ({})", account.alias, account.username);
    println!();

    let password_val = password("Enter password (hidden input)")?;
    if password_val.trim().is_empty() {
        println!("{} No password entered — cancelled.", colors::warn_prefix());
        press_enter();
        return Ok(());
    }

    let pb = spinner("Storing password in OS keychain...");
    // Passwords are stored as tokens via the same credential service
    let result = handle.services().store_account_token(account.uuid, password_val.trim()).await;
    pb.finish_and_clear();

    match result {
        Ok(_) => println!("{} Password stored securely.", colors::success_prefix()),
        Err(e) => println!("{} Failed: {}", colors::error_prefix(), e),
    }

    press_enter();
    Ok(())
}

// ─────────────────────────────────────────────────────────────────────────────
// SSH validate (full 10-check suite)
// ─────────────────────────────────────────────────────────────────────────────

async fn flow_ssh_validate(handle: &CliServicesHandle) -> FlowResult {
    section("SSH Validation");

    let account = match pick_account(handle).await? { Some(a) => a, None => return Ok(()) };

    println!("  Validating: {} → {}",
        colors::bold(&account.alias),
        colors::dim(account.platform_name.as_deref().unwrap_or("platform")));
    println!();

    let pb = spinner("Running 10 validation checks...");
    let result = handle.services().validate_ssh_setup(account.uuid, 10_000).await;
    pb.finish_and_clear();

    match result {
        Ok(validation) => {
            for check in &validation.checks {
                let icon = if check.passed { colors::green("✓") } else { colors::red("✗") };
                println!("  {}  {}  — {}", icon, colors::bold(&check.name), colors::dim(&check.detail));
            }
            println!();
            if validation.all_passed {
                println!("{} All 10 checks passed!", colors::success_prefix());
            } else {
                println!("{} Some checks failed — review the details above.", colors::warn_prefix());
            }
        }
        Err(e) => println!("{} Validation failed: {}", colors::error_prefix(), e),
    }

    press_enter();
    Ok(())
}

// ─────────────────────────────────────────────────────────────────────────────
// List SSH keys
// ─────────────────────────────────────────────────────────────────────────────

async fn flow_list_ssh_keys(handle: &CliServicesHandle) -> FlowResult {
    section("SSH Keys");

    let account = match pick_account(handle).await? { Some(a) => a, None => return Ok(()) };

    let pb = spinner("Fetching SSH keys...");
    let keys = handle.services().list_ssh_keys(account.uuid).await;
    pb.finish_and_clear();

    match keys {
        Ok(key_list) => {
            if key_list.is_empty() {
                println!("{} No SSH keys for {}.", colors::info_prefix(), colors::bold(&account.alias));
                println!("  Generate one with options {} → {}", colors::yellow("[7]"), colors::yellow("[1]"));
            } else {
                for key in &key_list {
                    println!("  {}  ({})", colors::bold(&key.name), colors::dim(&format!("{}", key.key_type)));
                    println!("    Fingerprint: {}", colors::dim(&key.fingerprint));
                    println!("    Private:     {}", colors::dim(&key.private_key_path));
                    println!("    Last tested: {}",
                        key.last_tested_at.map(|t| t.to_string()).unwrap_or_else(|| "never".to_string()));
                    println!();
                }
            }
        }
        Err(e) => println!("{} Failed: {}", colors::error_prefix(), e),
    }

    press_enter();
    Ok(())
}

// ─────────────────────────────────────────────────────────────────────────────
// Helpers
// ─────────────────────────────────────────────────────────────────────────────

/// Returns the direct URL where the user should paste the generated public key.
fn platform_ssh_settings_url(platform_name: &str) -> &'static str {
    match platform_name.to_lowercase().as_str() {
        "github"        => "https://github.com/settings/ssh/new",
        "gitlab"        => "https://gitlab.com/-/user_settings/ssh_keys",
        "bitbucket"     => "https://bitbucket.org/account/settings/ssh-keys/",
        "azure devops"  => "https://dev.azure.com/_usersettings/keys",
        "sourceforge"   => "https://sourceforge.net/auth/settings/",
        _               => "",
    }
}