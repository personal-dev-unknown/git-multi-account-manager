use std::sync::Arc;
use clap::Parser;
use uuid::Uuid;
use gm_kernel::kernel::Kernel;
use gm_shared::errors::GitManagerError;

use crate::commands::account::{parse_platform, platform_uuid_for};
use crate::services::CliServicesHandle;
use crate::ui::colors;

/// One-command account setup wizard. Interactive by default; all flags
/// available for scripting.
#[derive(Parser, Debug)]
pub struct SetupCmd {
    /// Platform slug (github, gitlab, bitbucket, azure_devops, etc.).
    #[arg(long)]
    pub platform: Option<String>,

    /// Account alias.
    #[arg(long)]
    pub alias: Option<String>,

    /// Git username on the platform.
    #[arg(long)]
    pub username: Option<String>,

    /// Email address.
    #[arg(long)]
    pub email: Option<String>,

    /// Auth method: "ssh" (default) or "https".
    #[arg(long, default_value = "ssh")]
    pub auth: String,

    /// SSH key type (ed25519, rsa). Only used when auth=ssh.
    #[arg(long, default_value = "ed25519")]
    pub key_type: String,

    /// SSH key passphrase. Only used when auth=ssh.
    #[arg(long)]
    pub passphrase: Option<String>,

    /// Add key to SSH agent immediately. Only used when auth=ssh.
    #[arg(long, default_value_t = true)]
    pub add_to_agent: bool,

    /// Personal Access Token or password. Only used when auth=https.
    #[arg(long)]
    pub token: Option<String>,

    /// Connection test timeout in milliseconds.
    #[arg(long, default_value_t = 15_000)]
    pub timeout_ms: u32,
}

/// Platform picker options: (slug, display label).
const PLATFORM_OPTIONS: &[(&str, &str)] = &[
    ("github",       "GitHub       (github.com)"),
    ("gitlab",       "GitLab       (gitlab.com)"),
    ("bitbucket",    "Bitbucket    (bitbucket.org)"),
    ("azure_devops", "Azure DevOps (ssh.dev.azure.com)"),
    ("sourceforge",  "SourceForge  (git.code.sf.net)"),
    ("self_hosted",  "Self-Hosted  (custom server)"),
    ("cloud_storage","Cloud Storage (S3/GCS/Azure Blob)"),
    ("local_path",   "Local Path   (filesystem)"),
];

pub async fn handle(cmd: SetupCmd, kernel: Arc<Kernel>) -> Result<(), GitManagerError> {
    let svc = kernel.get::<CliServicesHandle>()
        .ok_or_else(|| GitManagerError::Other("CLI services not initialised".to_string()))?;

    let is_interactive = cmd.platform.is_none()
        || cmd.alias.is_none()
        || cmd.username.is_none()
        || cmd.email.is_none();

    if is_interactive {
        run_interactive(svc).await
    } else {
        run_non_interactive(svc, cmd).await
    }
}

// ── Interactive mode ──────────────────────────────────────────────────────────

async fn run_interactive(svc: Arc<CliServicesHandle>) -> Result<(), GitManagerError> {
    println!("{}", colors::bold(&format!("{}", "╔══════════════════════════════════════════╗")));
    println!("{}", colors::bold(&format!("{}", "║         ACCOUNT SETUP WIZARD            ║")));
    println!("{}", colors::bold(&format!("{}", "╚══════════════════════════════════════════╝")));
    println!();

    // ── Step 1: Platform ────────────────────────────────────────────────────
    section("Platform");
    let plat_labels: Vec<&str> = PLATFORM_OPTIONS.iter().map(|(_, l)| *l).collect();
    let plat_idx = select("Choose your Git hosting platform", &plat_labels)?;
    let (slug, _) = PLATFORM_OPTIONS[plat_idx];
    let platform_type = parse_platform(slug)?;
    let platform_id = platform_uuid_for(&platform_type);

    // ── Step 2: Account details ─────────────────────────────────────────────
    section("Account Details");
    let alias = input("Account alias  (e.g. work, personal, client-acme)")?;
    let username = input("Git username on this platform")?;
    let email = input("Email (used in git commit author + SSH key comment)")?;

    // ── Step 3: Auth method ─────────────────────────────────────────────────
    section("Authentication");
    let auth_idx = select("Authentication method", &["SSH key (recommended)", "HTTPS + PAT/token"])?;
    let auth_method = if auth_idx == 0 { "ssh" } else { "https" };

    // ── Step 4: Create account ──────────────────────────────────────────────
    let pb = spinner("Creating account record…");
    let account = svc.services().add_account(gm_ports::inbound::commands::AddAccountCommand {
        alias: alias.clone(),
        platform_id,
        username: username.clone(),
        email: email.clone(),
        auth_method: auth_method.to_string(),
    }).await?;
    pb.finish_and_clear();
    println!("{} Account '{}' created on {}",
        colors::success_prefix(), colors::bold(&alias), colors::dim(&platform_display_name(platform_id)));

    // ── Step 5: Configure auth ──────────────────────────────────────────────
    if auth_idx == 0 {
        setup_ssh_interactive(svc, &account, &email).await?;
    } else {
        setup_https_interactive(svc, &account).await?;
    }

    println!();
    println!("{} Account '{}' is fully set up and ready!",
        colors::success_prefix(), colors::bold(&alias));
    println!("  Try cloning a repo: {}",
        colors::dim(&format!("git-zyrix clone owner/repo --account {}", account.alias)));
    println!();

    Ok(())
}

async fn setup_ssh_interactive(
    svc: Arc<CliServicesHandle>,
    account: &gm_shared::models::account::AccountDto,
    email: &str,
) -> Result<(), GitManagerError> {
    let key_type = "ed25519";
    let add_to_agent = confirm("Add key to SSH agent now?", true)?;
    let passphrase = if confirm("Protect key with a passphrase?", false)? {
        let pp = password("Key passphrase")?;
        let pp2 = password("Confirm passphrase")?;
        if pp != pp2 {
            println!("{} Passphrases do not match. Using no passphrase.", colors::yellow("!"));
            None
        } else {
            Some(pp)
        }
    } else {
        None
    };

    let pb = spinner(format!("Generating {key_type} key for '{}'…", account.alias));
    let key = svc.services().generate_ssh_key(gm_ports::inbound::commands::GenerateSshKeyCommand {
        account_uuid: account.uuid,
        key_type: key_type.to_string(),
        comment: Some(email.to_string()),
        passphrase,
        add_to_agent,
    }).await?;
    pb.finish_and_clear();

    println!("{} SSH key generated", colors::success_prefix());
    println!("  Type:        {}", key_type);
    println!("  Fingerprint: {}", colors::bold(&key.fingerprint));
    println!("  Public key:  {}", colors::dim(&key.public_key));

    if confirm("Test SSH connection now?", true)? {
        let pb = spinner("Testing SSH connection…");
        let result = svc.services().test_ssh_connection(gm_ports::inbound::commands::TestSshConnectionCommand {
            account_uuid: account.uuid,
            timeout_ms: Some(15_000),
        }).await?;
        pb.finish_and_clear();
        if result.success {
            println!("{} SSH connection successful{}",
                colors::success_prefix(),
                result.username.as_ref().map(|u| format!(" (greeted as {})", colors::bold(u))).unwrap_or_default());
        } else {
            println!("{} SSH connection failed: {}",
                colors::error_prefix(),
                result.error.as_deref().unwrap_or("unknown error"));
        }
    }

    println!();
    println!("  Add this public key to your account settings:");
    println!("  {}", colors::dim(ssh_keys_url(account.platform_id)));

    Ok(())
}

async fn setup_https_interactive(
    svc: Arc<CliServicesHandle>,
    account: &gm_shared::models::account::AccountDto,
) -> Result<(), GitManagerError> {
    let token = password("Personal Access Token")?;

    let pb = spinner("Storing credential…");
    svc.services().store_account_token(account.uuid, &token).await?;
    pb.finish_and_clear();
    println!("{} Credential stored securely", colors::success_prefix());

    Ok(())
}

// ── Non-interactive mode ──────────────────────────────────────────────────────

async fn run_non_interactive(svc: Arc<CliServicesHandle>, cmd: SetupCmd) -> Result<(), GitManagerError> {
    let platform_type = parse_platform(cmd.platform.as_deref().unwrap_or("github"))?;
    let platform_id = platform_uuid_for(&platform_type);

    let account = svc.services().add_account(gm_ports::inbound::commands::AddAccountCommand {
        alias: cmd.alias.clone().unwrap_or_default(),
        platform_id,
        username: cmd.username.clone().unwrap_or_default(),
        email: cmd.email.clone().unwrap_or_default(),
        auth_method: cmd.auth.clone(),
    }).await?;
    println!("{} Account '{}' created on {}",
        colors::success_prefix(), colors::bold(&account.alias),
        colors::dim(&platform_display_name(platform_id)));

    if cmd.auth == "ssh" {
        let key = svc.services().generate_ssh_key(gm_ports::inbound::commands::GenerateSshKeyCommand {
            account_uuid: account.uuid,
            key_type: cmd.key_type.clone(),
            comment: Some(cmd.email.clone().unwrap_or_default()),
            passphrase: cmd.passphrase.clone(),
            add_to_agent: cmd.add_to_agent,
        }).await?;
        println!("{} SSH key generated — fingerprint: {}", colors::success_prefix(), colors::bold(&key.fingerprint));

        let pb = spinner("Testing SSH connection…");
        let result = svc.services().test_ssh_connection(gm_ports::inbound::commands::TestSshConnectionCommand {
            account_uuid: account.uuid,
            timeout_ms: Some(cmd.timeout_ms),
        }).await?;
        pb.finish_and_clear();
        if result.success {
            println!("{} SSH connection successful{}", colors::success_prefix(),
                result.username.as_ref().map(|u| format!(" (greeted as {})", colors::bold(u))).unwrap_or_default());
        } else {
            println!("{} SSH connection failed: {}", colors::error_prefix(),
                result.error.as_deref().unwrap_or("unknown error"));
        }

        println!("  Add public key: {}", colors::dim(ssh_keys_url(platform_id)));
    } else if let Some(token) = cmd.token {
        svc.services().store_account_token(account.uuid, &token).await?;
        println!("{} Credential stored", colors::success_prefix());
    }

    Ok(())
}

// ── Helpers ───────────────────────────────────────────────────────────────────

fn section(title: &str) {
    println!();
    println!("  {} {} {}", colors::cyan("═══"), colors::bold(title), colors::cyan("═══"));
    println!();
}

fn spinner(msg: impl Into<String>) -> indicatif::ProgressBar {
    let pb = indicatif::ProgressBar::new_spinner();
    pb.set_style(indicatif::ProgressStyle::default_spinner()
        .template("{spinner:.cyan} {msg}")
        .expect("setup: hardcoded spinner template must be valid"));
    pb.set_message(msg.into());
    pb.enable_steady_tick(std::time::Duration::from_millis(80));
    pb
}

fn select(label: &str, items: &[&str]) -> Result<usize, GitManagerError> {
    dialoguer::Select::with_theme(&dialoguer::theme::ColorfulTheme::default())
        .with_prompt(label)
        .items(items)
        .default(0)
        .interact_opt()
        .map_err(|_| GitManagerError::Other("Input cancelled".to_string()))?
        .ok_or(GitManagerError::Other("Input cancelled".to_string()))
}

fn input(label: &str) -> Result<String, GitManagerError> {
    dialoguer::Input::<String>::with_theme(&dialoguer::theme::ColorfulTheme::default())
        .with_prompt(label)
        .allow_empty(false)
        .interact_text()
        .map_err(|_| GitManagerError::Other("Input cancelled".to_string()))
}

fn password(label: &str) -> Result<String, GitManagerError> {
    dialoguer::Password::with_theme(&dialoguer::theme::ColorfulTheme::default())
        .with_prompt(label)
        .allow_empty_password(false)
        .interact()
        .map_err(|_| GitManagerError::Other("Input cancelled".to_string()))
}

fn confirm(label: &str, default: bool) -> Result<bool, GitManagerError> {
    dialoguer::Confirm::with_theme(&dialoguer::theme::ColorfulTheme::default())
        .with_prompt(label)
        .default(default)
        .interact_opt()
        .map_err(|_| GitManagerError::Other("Input cancelled".to_string()))?
        .ok_or(GitManagerError::Other("Input cancelled".to_string()))
}

fn platform_display_name(platform_id: Uuid) -> String {
    match platform_id.to_string().as_str() {
        "00000000-0001-0000-0000-000000000001" => "GitHub",
        "00000000-0002-0000-0000-000000000001" => "GitLab",
        "00000000-0003-0000-0000-000000000001" => "Bitbucket",
        "00000000-0004-0000-0000-000000000001" => "Azure DevOps",
        "00000000-0005-0000-0000-000000000001" => "SourceForge",
        "00000000-0006-0000-0000-000000000001" => "Self-Hosted",
        "00000000-0007-0000-0000-000000000001" => "Cloud Storage",
        "00000000-0008-0000-0000-000000000001" => "Local Path",
        _ => "Unknown",
    }.to_string()
}

fn ssh_keys_url(platform_id: Uuid) -> &'static str {
    match platform_id.to_string().as_str() {
        "00000000-0001-0000-0000-000000000001" => "https://github.com/settings/ssh/new",
        "00000000-0002-0000-0000-000000000001" => "https://gitlab.com/-/user_settings/ssh_keys",
        "00000000-0003-0000-0000-000000000001" => "https://bitbucket.org/account/settings/ssh-keys/",
        "00000000-0004-0000-0000-000000000001" => "https://ssh.dev.azure.com",
        _ => "https://github.com/settings/ssh/new",
    }
}
