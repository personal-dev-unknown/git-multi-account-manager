// // // crates/gm_interface_cli/src/ui/interactive.rs
// // // Interactive multi-select and fuzzy-search for the clone wizard and bulk ops.

// // use gm_shared::errors::GitManagerError;

// // /// Presents a multi-select list and returns the indices of chosen items.
// // pub fn multi_select(label: &str, options: &[String]) -> Result<Vec<usize>, GitManagerError> {
// //     use dialoguer::MultiSelect;
// //     MultiSelect::new()
// //         .with_prompt(label)
// //         .items(options)
// //         .interact()
// //         .map_err(|e| GitManagerError::Other(format!("multi-select: {e}")))
// // }

// // /// Fuzzy-search a list and return the selected index.
// // pub fn fuzzy_select(label: &str, options: &[String]) -> Result<Option<usize>, GitManagerError> {
// //     use dialoguer::FuzzySelect;
// //     FuzzySelect::new()
// //         .with_prompt(label)
// //         .items(options)
// //         .interact_opt()
// //         .map_err(|e| GitManagerError::Other(format!("fuzzy-select: {e}")))
// // }






// // crates/gm_interface_cli/src/ui/interactive.rs
// //
// // ── Purpose ───────────────────────────────────────────────────────────────────
// // Two existing helpers (multi_select, fuzzy_select) live here unchanged.
// // This module also owns the full interactive TUI session launched when the user
// // runs `git-manager` with no subcommand and selects "Interactive mode".
// //
// // ── Reuse contract — nothing is duplicated ────────────────────────────────────
// // Input collection  →  crate::ui::prompts::*           (prompt_required, prompt_select, ...)
// // Progress feedback →  crate::ui::progress::spinner()
// // Colour output     →  crate::ui::colors::*             (bold, dim, success_prefix, ...)
// // Table display     →  crate::ui::tables::*             (print_accounts, print_ssh_keys, ...)
// // Platform helpers  →  crate::commands::account::{parse_platform, platform_uuid_for}
// // Service calls     →  handle.services().*              (same methods used by commands/*)
// //
// // ── Menu structure (matches v1 Python CLI) ────────────────────────────────────
// //  Repository Operations
// //  [1] Clone a repository
// //  [2] Check current repository account
// //  [3] Git push / pull / sync operations
// //  [4] Set up repository for specific account
// //  Account Management
// //  [5] Show all accounts
// //  [6] Test SSH connections
// //  [7] Generate and Manage SSH keys and PATs
// //  [8] Exit

// use std::io::BufRead;
// use std::sync::Arc;

// use uuid::Uuid;
// use gm_shared::{
//     errors::GitManagerError,
//     models::{account::AccountDto, platform::PlatformType, repository::RepositoryDto},
// };
// use gm_ports::inbound::commands::{
//     AddAccountCommand, CloneRepositoryCommand, GenerateSshKeyCommand,
//     PullRepositoryCommand, PushRepositoryCommand, TestSshConnectionCommand,
// };

// use crate::commands::account::{parse_platform, platform_uuid_for};
// use crate::services::CliServicesHandle;
// use crate::ui::{colors, progress, prompts, tables};

// // ─────────────────────────────────────────────────────────────────────────────
// // Original helpers — kept exactly as they were; nothing removed.
// // ─────────────────────────────────────────────────────────────────────────────

// /// Presents a multi-select list and returns the indices of chosen items.
// pub fn multi_select(label: &str, options: &[String]) -> Result<Vec<usize>, GitManagerError> {
//     use dialoguer::MultiSelect;
//     MultiSelect::new()
//         .with_prompt(label)
//         .items(options)
//         .interact()
//         .map_err(|e| GitManagerError::Other(format!("multi-select: {e}")))
// }

// /// Fuzzy-search a list and return the selected index.
// pub fn fuzzy_select(label: &str, options: &[String]) -> Result<Option<usize>, GitManagerError> {
//     use dialoguer::FuzzySelect;
//     FuzzySelect::new()
//         .with_prompt(label)
//         .items(options)
//         .interact_opt()
//         .map_err(|e| GitManagerError::Other(format!("fuzzy-select: {e}")))
// }

// // ─────────────────────────────────────────────────────────────────────────────
// // Constants
// // ─────────────────────────────────────────────────────────────────────────────

// const MENU_ITEMS: &[&str] = &[
//     "[1] Clone a repository",
//     "[2] Check current repository account",
//     "[3] Git push / pull / sync operations",
//     "[4] Set up repository for specific account",
//     "[5] Show all accounts",
//     "[6] Test SSH connections",
//     "[7] Generate and Manage SSH keys and PATs",
//     "[8] Exit",
// ];

// /// Platform display labels paired with their slugs.
// /// Slugs feed directly into `parse_platform` + `platform_uuid_for`
// /// from commands/account.rs — no duplication of UUID mapping logic.
// const PLATFORM_OPTIONS: &[(&str, &str)] = &[
//     ("github",       "GitHub       (github.com)"),
//     ("gitlab",       "GitLab       (gitlab.com)"),
//     ("bitbucket",    "Bitbucket    (bitbucket.org)"),
//     ("azure_devops", "Azure DevOps (ssh.dev.azure.com)"),
//     ("sourceforge",  "SourceForge  (git.code.sf.net)"),
// ];

// // ─────────────────────────────────────────────────────────────────────────────
// // Entry point
// // ─────────────────────────────────────────────────────────────────────────────

// /// Runs the interactive session.
// /// Loops until the user selects [8] Exit or Ctrl-C at the main menu.
// ///
// /// Called by `plugin.rs::run()` when the user picks "Interactive" at startup.
// pub async fn run_menu(handle: Arc<CliServicesHandle>) -> Result<(), GitManagerError> {
//     loop {
//         println!();
//         println!("  {}  Repository Operations  {}  Account Management",
//             colors::cyan("═══"), colors::cyan("═══"));
//         println!();

//         let selection = prompts::prompt_select("Main Menu", MENU_ITEMS)?;
//         println!();

//         let result: Result<(), GitManagerError> = match selection {
//             0 => flow_clone(&handle).await,
//             1 => flow_check_repo(&handle).await,
//             2 => flow_git_ops(&handle).await,
//             3 => flow_setup_repo(&handle).await,
//             4 => flow_show_accounts(&handle).await,
//             5 => flow_test_ssh(&handle).await,
//             6 => flow_manage_keys(&handle).await,
//             7 => {
//                 println!("\nGoodbye!\n");
//                 return Ok(());
//             }
//             _ => unreachable!(),
//         };

//         // Print any unexpected service error and loop back — never crash.
//         if let Err(e) = result {
//             println!("{} {}", colors::error_prefix(), e);
//             press_enter();
//         }
//     }
// }

// // ─────────────────────────────────────────────────────────────────────────────
// // [1] Clone a repository
// // ─────────────────────────────────────────────────────────────────────────────

// async fn flow_clone(handle: &CliServicesHandle) -> Result<(), GitManagerError> {
//     section("Clone a Repository");

//     let account = match pick_account(handle).await? {
//         Some(a) => a,
//         None    => return Ok(()),
//     };

//     println!("  {}  SSH:   git@github.com:owner/repo.git", colors::dim("hint"));
//     println!("  {}  HTTPS: https://github.com/owner/repo.git", colors::dim("hint"));
//     println!();

//     let url = prompts::prompt_required("Repository URL", None)?;

//     let raw_dest = prompts::prompt_required(
//         "Local destination (blank = ~/git-repos/<repo-name>)",
//         Some(""),
//     )?;
//     let destination = if raw_dest.trim().is_empty() {
//         None
//     } else {
//         Some(raw_dest.trim().to_string())
//     };

//     let shallow = prompts::confirm("Shallow clone? (faster, omits full history)", false)?;

//     let pb = progress::spinner(format!("Cloning {}…", url));
//     let result = handle.services().clone_repository(CloneRepositoryCommand {
//         url:          url.clone(),
//         account_uuid: account.uuid,
//         destination,
//         branch:       None,
//         shallow,
//     }).await;
//     pb.finish_and_clear();

//     match result {
//         Ok(repo) => {
//             println!("{} Cloned successfully!", colors::success_prefix());
//             if let Some(path) = &repo.local_path {
//                 println!("  Location : {}", colors::dim(path));
//             }
//             println!("  Account  : {} ({})", account.alias, account.username);
//             println!();
//             println!("  {}  Use option [3] to push and pull.", colors::dim("→"));
//         }
//         Err(e) => println!("{} Clone failed: {}", colors::error_prefix(), e),
//     }

//     press_enter();
//     Ok(())
// }

// // ─────────────────────────────────────────────────────────────────────────────
// // [2] Check current repository account
// // ─────────────────────────────────────────────────────────────────────────────

// async fn flow_check_repo(handle: &CliServicesHandle) -> Result<(), GitManagerError> {
//     section("Check Repository Account");

//     let path_str = prompts::prompt_required("Repository path", Some("."))?;
//     let path_str = if path_str.trim().is_empty() {
//         ".".to_string()
//     } else {
//         path_str.trim().to_string()
//     };

//     let pb = progress::spinner("Looking up repository…");
//     let repos = handle.services().list_repositories(None).await?;
//     pb.finish_and_clear();

//     let canonical = std::fs::canonicalize(&path_str).ok();
//     let found = repos.iter().find(|r| {
//         r.local_path.as_deref().map(|lp| {
//             std::fs::canonicalize(lp).ok().as_ref() == canonical.as_ref()
//                 || lp == path_str.as_str()
//         }).unwrap_or(false)
//     });

//     match found {
//         Some(repo) => {
//             println!("{} Git Manager is tracking this repository.", colors::success_prefix());
//             println!();
//             println!("  Name     : {}", colors::bold(&repo.name));
//             println!("  Branch   : {}", colors::dim(&repo.default_branch));
//             if let Some(sha) = &repo.last_commit_sha {
//                 println!("  Last SHA : {}", colors::dim(&sha[..sha.len().min(8)]));
//             }
//         }
//         None => {
//             println!("{} This path is not tracked by Git Manager.", colors::warn_prefix());
//             println!("  Use [4] to register it, or [1] to clone with tracking.");
//         }
//     }

//     press_enter();
//     Ok(())
// }

// // ─────────────────────────────────────────────────────────────────────────────
// // [3] Git operations
// // ─────────────────────────────────────────────────────────────────────────────

// const GIT_OPS_ITEMS: &[&str] = &[
//     "[1] Pull   — fetch and integrate remote changes",
//     "[2] Push   — stage all, commit, and push",
//     "[3] Status — show working directory changes",
//     "[4] ← Back",
// ];

// async fn flow_git_ops(handle: &CliServicesHandle) -> Result<(), GitManagerError> {
//     section("Git Operations");

//     let sel = prompts::prompt_select("Select operation", GIT_OPS_ITEMS)?;
//     println!();

//     match sel {
//         0 => flow_git_pull(handle).await,
//         1 => flow_git_push(handle).await,
//         2 => flow_git_status(handle).await,
//         3 => Ok(()),
//         _ => unreachable!(),
//     }
// }

// async fn flow_git_pull(handle: &CliServicesHandle) -> Result<(), GitManagerError> {
//     section("Git Pull");

//     let account = match pick_account(handle).await? { Some(a) => a, None => return Ok(()) };
//     let repo    = match pick_cloned_repo(handle, account.uuid).await? { Some(r) => r, None => return Ok(()) };
//     let rebase  = prompts::confirm("Use --rebase instead of --merge?", false)?;

//     let pb = progress::spinner("Pulling…");
//     let result = handle.services().git_pull(PullRepositoryCommand {
//         repository_uuid: repo.uuid,
//         account_uuid:    account.uuid,
//         branch:          None,
//         rebase,
//     }).await;
//     pb.finish_and_clear();

//     match result {
//         Ok(r) if r.had_conflicts => {
//             println!("{} Pull completed with merge conflicts.", colors::warn_prefix());
//             println!("  Resolve conflicts, then push with [3] → Push.");
//         }
//         Ok(r) => {
//             let sha = r.current_sha.as_deref().map(|s| &s[..s.len().min(8)]).unwrap_or("?");
//             println!("{} Pulled {} commit(s).  HEAD: {}",
//                 colors::success_prefix(), r.commits_transferred, colors::dim(sha));
//         }
//         Err(e) => println!("{} Pull failed: {}", colors::error_prefix(), e),
//     }

//     press_enter();
//     Ok(())
// }

// async fn flow_git_push(handle: &CliServicesHandle) -> Result<(), GitManagerError> {
//     section("Git Push");

//     let account = match pick_account(handle).await? { Some(a) => a, None => return Ok(()) };
//     let repo    = match pick_cloned_repo(handle, account.uuid).await? { Some(r) => r, None => return Ok(()) };

//     // Show what will be committed before asking for a message.
//     if let Some(ref local) = repo.local_path {
//         let pb = progress::spinner("Checking working directory…");
//         let entries = handle.services()
//             .git_status(std::path::Path::new(local)).await
//             .unwrap_or_default();
//         pb.finish_and_clear();

//         if entries.is_empty() {
//             println!("{} Working directory clean — nothing to commit.", colors::info_prefix());
//             press_enter();
//             return Ok(());
//         }
//         println!("  {} file(s) to be committed:", entries.len());
//         for e in &entries {
//             println!("    {}  {}", colors::dim(&e.status), e.path);
//         }
//         println!();
//     }

//     let message = prompts::prompt_required("Commit message", None)?;
//     let branch  = prompts::prompt_required("Branch", Some(&repo.default_branch))?;

//     let pb = progress::spinner("Staging → committing → pushing…");
//     let result = handle.services().git_push(PushRepositoryCommand {
//         repository_uuid: repo.uuid,
//         account_uuid:    account.uuid,
//         commit_message:  message,
//         branch,
//         force: false,
//     }).await;
//     pb.finish_and_clear();

//     match result {
//         Ok(r) => {
//             let sha = r.current_sha.as_deref().map(|s| &s[..s.len().min(8)]).unwrap_or("?");
//             println!("{} Pushed {} commit(s).  HEAD: {}",
//                 colors::success_prefix(), r.commits_transferred, colors::dim(sha));
//         }
//         Err(e) => println!("{} Push failed: {}", colors::error_prefix(), e),
//     }

//     press_enter();
//     Ok(())
// }

// async fn flow_git_status(handle: &CliServicesHandle) -> Result<(), GitManagerError> {
//     section("Working Directory Status");

//     let path_str = prompts::prompt_required("Repository path", Some("."))?;
//     let path_str = if path_str.trim().is_empty() { ".".to_string() } else { path_str.trim().to_string() };

//     let pb = progress::spinner("Checking status…");
//     let result = handle.services().git_status(std::path::Path::new(&path_str)).await;
//     pb.finish_and_clear();

//     match result {
//         Ok(entries) if entries.is_empty() => {
//             println!("{} Working directory is clean.", colors::success_prefix());
//         }
//         Ok(entries) => {
//             println!("  {} file(s) changed:", entries.len());
//             for e in &entries {
//                 let label = match e.status.trim() {
//                     s if s.starts_with('M') || s.ends_with('M') => colors::yellow("modified"),
//                     s if s.starts_with('A')                     => colors::green("added   "),
//                     s if s.starts_with('D') || s.ends_with('D') => colors::red("deleted "),
//                     "??"                                         => colors::dim("untracked"),
//                     "UU"                                         => colors::red("conflict "),
//                     _                                            => colors::dim("changed  "),
//                 };
//                 println!("    {} {}  {}", colors::dim(&e.status), label, e.path);
//             }
//         }
//         Err(e) => println!("{} Could not get status: {}", colors::error_prefix(), e),
//     }

//     press_enter();
//     Ok(())
// }

// // ─────────────────────────────────────────────────────────────────────────────
// // [4] Set up repository for specific account
// // ─────────────────────────────────────────────────────────────────────────────

// async fn flow_setup_repo(handle: &CliServicesHandle) -> Result<(), GitManagerError> {
//     section("Setup Local Project with Remote");

//     let account = match pick_account(handle).await? { Some(a) => a, None => return Ok(()) };

//     let local_path = prompts::prompt_required("Local repository path", None)?;
//     let local_path = local_path.trim().to_string();

//     if !std::path::Path::new(&local_path).join(".git").exists() {
//         println!("{} No .git directory found at '{}' — is this a git repository?",
//             colors::warn_prefix(), local_path);
//         press_enter();
//         return Ok(());
//     }

//     let remote_url = prompts::prompt_required("Remote URL (SSH format)", None)?;
//     let remote_url = remote_url.trim().to_string();

//     // Rewrite the remote URL to use the account's SSH host alias.
//     // git@github.com:owner/repo.git → git@github.com-work:owner/repo.git
//     let alias_url = if let Some(at_pos) = remote_url.find('@') {
//         if let Some(rel_colon) = remote_url[at_pos..].find(':') {
//             let host_end = at_pos + rel_colon;
//             let host     = &remote_url[at_pos + 1..host_end];
//             format!("git@{}-{}:{}", host, account.alias, &remote_url[host_end + 1..])
//         } else {
//             remote_url.clone()
//         }
//     } else {
//         remote_url.clone()
//     };

//     println!();
//     println!("{} Run these commands to connect the repository to '{}':",
//         colors::success_prefix(), account.alias);
//     println!();
//     println!("    cd {}", local_path);
//     println!("    {}", colors::bold(&format!("git remote set-url origin {}", alias_url)));
//     println!();
//     println!("  This uses the '{}' SSH config entry so git authenticates", colors::dim(&alias_url));
//     println!("  as '{}' automatically on every push and pull.", account.alias);

//     press_enter();
//     Ok(())
// }

// // ─────────────────────────────────────────────────────────────────────────────
// // [5] Show all accounts
// // ─────────────────────────────────────────────────────────────────────────────

// async fn flow_show_accounts(handle: &CliServicesHandle) -> Result<(), GitManagerError> {
//     section("All Accounts");

//     let pb = progress::spinner("Loading accounts…");
//     let accounts = handle.services().list_accounts(None).await?;
//     pb.finish_and_clear();

//     if accounts.is_empty() {
//         println!("{} No accounts configured yet.", colors::info_prefix());
//         println!("  Use option [7] to add an account and generate an SSH key.");
//     } else {
//         tables::print_accounts(&accounts); // reuses ui/tables.rs — zero duplication
//     }

//     press_enter();
//     Ok(())
// }

// // ─────────────────────────────────────────────────────────────────────────────
// // [6] Test SSH connections
// // ─────────────────────────────────────────────────────────────────────────────

// async fn flow_test_ssh(handle: &CliServicesHandle) -> Result<(), GitManagerError> {
//     section("Test SSH Connection");

//     let account = match pick_account(handle).await? { Some(a) => a, None => return Ok(()) };

//     let pb = progress::spinner(format!("Testing SSH for '{}'…", account.alias));
//     let result = handle.services().test_ssh_connection(TestSshConnectionCommand {
//         account_uuid: account.uuid,
//         timeout_ms:   Some(10_000),
//     }).await;
//     pb.finish_and_clear();

//     match result {
//         Ok(r) if r.success => {
//             let who = r.username.as_deref().unwrap_or("(unknown)");
//             println!("{} Connection succeeded — authenticated as {}",
//                 colors::success_prefix(), colors::bold(who));
//         }
//         Ok(r) => {
//             println!("{} Connection failed.", colors::error_prefix());
//             if let Some(err) = &r.error { println!("  Reason: {}", err); }
//             println!();
//             println!("  Fixes:");
//             println!("    1. Add the public key to your {} SSH settings.",
//                 account.platform_name.as_deref().unwrap_or("platform"));
//             println!("    2. Use option [7] to generate a new key pair.");
//             println!("    3. Verify port 22 is not blocked.");
//         }
//         Err(e) => println!("{} Test failed: {}", colors::error_prefix(), e),
//     }

//     press_enter();
//     Ok(())
// }

// // ─────────────────────────────────────────────────────────────────────────────
// // [7] Generate and Manage SSH keys and PATs
// // ─────────────────────────────────────────────────────────────────────────────

// const KEY_MENU_ITEMS: &[&str] = &[
//     "[1] Generate new SSH key  (for existing or new account)",
//     "[2] Setup Personal Access Token (PAT)",
//     "[3] ← Back",
// ];

// async fn flow_manage_keys(handle: &CliServicesHandle) -> Result<(), GitManagerError> {
//     section("Account Authentication Setup");

//     let sel = prompts::prompt_select("Select action", KEY_MENU_ITEMS)?;
//     println!();

//     match sel {
//         0 => flow_generate_ssh_key(handle).await,
//         1 => flow_setup_pat(handle).await,
//         2 => Ok(()),
//         _ => unreachable!(),
//     }
// }

// async fn flow_generate_ssh_key(handle: &CliServicesHandle) -> Result<(), GitManagerError> {
//     section("SSH Key Generation");

//     // Step 1: Pick or create an account.
//     let accounts = handle.services().list_accounts(None).await?;
//     let account: AccountDto = if accounts.is_empty() {
//         println!("{} No accounts yet — creating one first.", colors::info_prefix());
//         println!();
//         create_account_interactive(handle).await?
//     } else {
//         let mut labels: Vec<String> = accounts.iter()
//             .map(|a| format!("{:<20} {:<12} {}",
//                 a.alias,
//                 a.platform_name.as_deref().unwrap_or("?"),
//                 colors::dim(&a.username)))
//             .collect();
//         labels.push("+ Create a new account".to_string());

//         let opts: Vec<&str> = labels.iter().map(|s| s.as_str()).collect();
//         let idx = prompts::prompt_select("Step 1: Account", &opts)?;

//         if idx == accounts.len() {
//             println!();
//             create_account_interactive(handle).await?
//         } else {
//             accounts[idx].clone()
//         }
//     };

//     // Step 2: Key type.
//     println!();
//     let key_type_idx = prompts::prompt_select("Step 2: Key type", &[
//         "Ed25519  — recommended (modern, fast, small)",
//         "RSA 4096 — for legacy platforms without Ed25519 support",
//     ])?;
//     let key_type = if key_type_idx == 0 { "ed25519" } else { "rsa" };

//     let add_to_agent = prompts::confirm("Add to SSH agent automatically?", true)?;

//     // Step 3: Generate.
//     println!();
//     let pb = progress::spinner(format!("Generating {} key for '{}'…", key_type, account.alias));
//     let result = handle.services().generate_ssh_key(GenerateSshKeyCommand {
//         account_uuid: account.uuid,
//         key_type:     key_type.to_string(),
//         comment:      Some(account.email.clone()),
//         passphrase:   None,
//         add_to_agent,
//     }).await;
//     pb.finish_and_clear();

//     match result {
//         Ok(key) => {
//             println!("{} SSH key generated for '{}'!", colors::success_prefix(), account.alias);
//             println!();
//             println!("  Key name    : {}", colors::bold(&key.name));
//             println!("  Fingerprint : {}", colors::dim(&key.fingerprint));
//             println!("  Private key : {}", colors::dim(&key.private_key_path));
//             println!();
//             println!("  {} Add this public key to your {} SSH settings:",
//                 colors::info_prefix(),
//                 account.platform_name.as_deref().unwrap_or("platform"));
//             println!();
//             println!("  {}", key.public_key);
//             println!();
//             println!("  {} Then use option [6] to verify the connection.", colors::dim("→"));
//         }
//         Err(e) => println!("{} Key generation failed: {}", colors::error_prefix(), e),
//     }

//     press_enter();
//     Ok(())
// }

// async fn flow_setup_pat(handle: &CliServicesHandle) -> Result<(), GitManagerError> {
//     section("Personal Access Token Setup");

//     let account = match pick_account(handle).await? { Some(a) => a, None => return Ok(()) };

//     println!("  Platform : {}", account.platform_name.as_deref().unwrap_or("unknown"));
//     println!("  Account  : {} ({})", account.alias, account.username);
//     println!();
//     println!("  Token format:  GitHub → ghp_...   GitLab → glpat-...");
//     println!("                 Bitbucket → username:app_password");
//     println!("                 Azure DevOps → organization:token");
//     println!();

//     let token = prompts::prompt_password("Paste PAT / App Password (hidden input)")?;
//     if token.trim().is_empty() {
//         println!("{} No token entered — cancelled.", colors::warn_prefix());
//         press_enter();
//         return Ok(());
//     }

//     let pb = progress::spinner("Storing token in OS keychain…");
//     let result = handle.services().store_account_token(account.uuid, token.trim()).await;
//     pb.finish_and_clear();

//     match result {
//         Ok(_) => println!("{} Token stored securely in the OS keychain.", colors::success_prefix()),
//         Err(e) => println!("{} Failed to store token: {}", colors::error_prefix(), e),
//     }

//     press_enter();
//     Ok(())
// }

// // ─────────────────────────────────────────────────────────────────────────────
// // Shared private helpers
// // ─────────────────────────────────────────────────────────────────────────────

// /// Creates a new account interactively and returns the resulting AccountDto.
// async fn create_account_interactive(handle: &CliServicesHandle) -> Result<AccountDto, GitManagerError> {
//     section("New Account Details");

//     let alias = prompts::prompt_required("Account alias (e.g. work, personal, client-acme)", None)?;

//     let plat_labels: Vec<&str> = PLATFORM_OPTIONS.iter().map(|(_, l)| *l).collect();
//     let plat_idx               = prompts::prompt_select("Platform", &plat_labels)?;
//     let (slug, _)              = PLATFORM_OPTIONS[plat_idx];
//     let platform_type          = parse_platform(slug)?; // reuses commands/account.rs helper
//     let platform_id            = platform_uuid_for(&platform_type); // reuses commands/account.rs helper

//     let username = prompts::prompt_required("Git username on this platform", None)?;
//     let email    = prompts::prompt_required("Email (used in commits and SSH key comment)", None)?;

//     let pb = progress::spinner(format!("Creating account '{}'…", alias.trim()));
//     let result = handle.services().add_account(AddAccountCommand {
//         alias:       alias.trim().to_string(),
//         platform_id,
//         username:    username.trim().to_string(),
//         email:       email.trim().to_string(),
//         auth_method: "ssh".to_string(),
//     }).await;
//     pb.finish_and_clear();

//     match result {
//         Ok(a) => {
//             println!("{} Account '{}' created.", colors::success_prefix(), a.alias);
//             Ok(a)
//         }
//         Err(e) => Err(e),
//     }
// }

// /// Shows a list of all accounts and lets the user pick one.
// /// Returns None if no accounts exist.
// async fn pick_account(handle: &CliServicesHandle) -> Result<Option<AccountDto>, GitManagerError> {
//     let accounts = handle.services().list_accounts(None).await?;
//     if accounts.is_empty() {
//         println!("{} No accounts found. Use option [7] to add one.", colors::info_prefix());
//         press_enter();
//         return Ok(None);
//     }

//     let labels: Vec<String> = accounts.iter()
//         .map(|a| format!("{:<20} {:<12} {}",
//             a.alias,
//             a.platform_name.as_deref().unwrap_or("?"),
//             colors::dim(&a.username)))
//         .collect();
//     let opts: Vec<&str> = labels.iter().map(|s| s.as_str()).collect();

//     let idx = prompts::prompt_select("Select account", &opts)?;
//     Ok(Some(accounts[idx].clone()))
// }

// /// Shows cloned repositories for an account and lets the user pick one.
// /// Returns None if none are cloned yet.
// async fn pick_cloned_repo(
//     handle: &CliServicesHandle,
//     account_uuid: Uuid,
// ) -> Result<Option<RepositoryDto>, GitManagerError> {
//     let repos  = handle.services().list_repositories(Some(account_uuid)).await?;
//     let cloned: Vec<_> = repos.into_iter().filter(|r| r.is_cloned).collect();

//     if cloned.is_empty() {
//         println!("{} No cloned repositories for this account.", colors::info_prefix());
//         println!("  Use option [1] to clone one first.");
//         press_enter();
//         return Ok(None);
//     }

//     let labels: Vec<String> = cloned.iter()
//         .map(|r| format!("{:<30} {}",
//             r.name, colors::dim(r.local_path.as_deref().unwrap_or("?"))))
//         .collect();
//     let opts: Vec<&str> = labels.iter().map(|s| s.as_str()).collect();

//     let idx = prompts::prompt_select("Select repository", &opts)?;
//     Ok(Some(cloned[idx].clone()))
// }

// /// Section header printed before each flow's output.
// fn section(title: &str) {
//     println!("  {} {} {}", colors::cyan("═══"), colors::bold(title), colors::cyan("═══"));
//     println!();
// }

// /// Blocks until Enter is pressed — gives the user time to read output
// /// before the main menu is redrawn.
// fn press_enter() {
//     println!();
//     println!("  {}  Press Enter to return to the main menu…", colors::dim("→"));
//     let stdin = std::io::stdin();
//     let mut lock = stdin.lock();
//     let mut line = String::new();
//     let _ = lock.read_line(&mut line);
// }
