// crates/gm_interface_cli/src/commands/theme.rs
//
// Theme management: list available colour schemes, switch, show current,
// and preview the active palette.

use std::sync::{Arc, Mutex};
use clap::{Parser, Subcommand};
use gm_kernel::kernel::Kernel;
use gm_shared::errors::GitManagerError;

use crate::services::CliServicesHandle;
use crate::ui::{colors::*, theme_engine::ThemeRegistry};

#[derive(Parser, Debug)]
pub struct ThemeCmd {
    #[command(subcommand)]
    pub action: ThemeAction,
}

#[derive(Subcommand, Debug)]
pub enum ThemeAction {
    /// List all available colour schemes.
    List,
    /// Show the currently active colour scheme.
    Current,
    /// Switch to a colour scheme by slug name.
    Set { slug: String },
    /// Preview the current theme's colour palette in the terminal.
    Preview,
}

pub async fn handle(cmd: ThemeCmd, kernel: Arc<Kernel>) -> Result<(), GitManagerError> {
    match cmd.action {
        ThemeAction::List => handle_list(&kernel),
        ThemeAction::Current => handle_current(&kernel),
        ThemeAction::Set { slug } => handle_set(&kernel, &slug).await,
        ThemeAction::Preview => handle_preview(&kernel),
    }
}

fn reg(kernel: &Kernel) -> Result<Arc<Mutex<ThemeRegistry>>, GitManagerError> {
    kernel
        .get::<Mutex<ThemeRegistry>>()
        .ok_or_else(|| GitManagerError::Other("Theme registry not initialised".to_string()))
}

fn handle_list(kernel: &Kernel) -> Result<(), GitManagerError> {
    let r = reg(kernel)?;
    let lock = r.lock()
        .expect("theme: registry Mutex poisoned");
    let current_slug = lock.current().slug.clone();

    println!("{} Available colour schemes:\n", bold("Theme Manager"));
    println!("  {:<18} {:<8} {:<50}  {}", "Slug", "Type", "Name", "");
    println!("  {}", dim(&"─".repeat(100)));

    for theme in lock.all() {
        let marker = if theme.slug == current_slug {
            format!("{}", green("◀ current"))
        } else {
            String::new()
        };
        println!("  {:<18} {:<8} {:<50}  {}",
            dim(&theme.slug),
            cyan(&theme.theme_type.to_string()),
            bold(&theme.name),
            marker,
        );
    }
    println!();
    println!("  Use: {} {} {}",
        dim("git-zyrix theme set"),
        green("<slug>"),
        dim("to switch"));
    Ok(())
}

fn handle_current(kernel: &Kernel) -> Result<(), GitManagerError> {
    let r = reg(kernel)?;
    let lock = r.lock()
        .expect("theme: registry Mutex poisoned");
    let theme = lock.current();
    let c = &theme.colors;

    println!();
    println!("  {}  {}  ({})", bold(&theme.name), dim(&theme.slug),
        cyan(&theme.theme_type.to_string()));
    println!("  {}", dim(&theme.description));
    println!();
    println!("  {}   {:?}", bold("Primary:"),   c.primary);
    println!("  {} {:?}", bold("Secondary:"), c.secondary);
    println!("  {}    {:?}", bold("Accent:"),    c.accent);
    println!("  {}   {:?}", bold("Success:"),   c.success);
    println!("  {}     {:?}", bold("Error:"),     c.error);
    println!("  {}   {:?}", bold("Warning:"),   c.warning);
    println!("  {}      {:?}", bold("Info:"),      c.info);
    println!("  {}    {:?}", bold("Banner:"),    c.banner);
    println!();
    Ok(())
}

async fn handle_set(kernel: &Kernel, slug: &str) -> Result<(), GitManagerError> {
    let r = reg(kernel)?;
    {
        let mut lock = r.lock()
            .expect("theme: registry Mutex poisoned");
        if lock.slug_index(slug).is_none() {
            return Err(GitManagerError::Other(format!(
                "Unknown theme '{}'. Use 'git-zyrix theme list' to see available themes.",
                slug
            )));
        }
        lock.set_by_slug(slug);
        lock.save_state();
    }

    // Persist to database via ConfigService
    if let Some(handle) = kernel.get::<CliServicesHandle>() {
        let _ = handle.services().config_set("ui.theme_active_slug", slug).await;
    }

    let r2 = r.lock()
        .expect("theme: registry Mutex poisoned");
    let theme = r2.current();
    println!("{} Theme switched to '{}'", success_prefix(), bold(&theme.name));
    println!("  {}", dim(&theme.description));
    println!();
    println!("  Preview: {} {}", dim("git-zyrix theme preview"), dim("to see colours"));
    Ok(())
}

fn handle_preview(kernel: &Kernel) -> Result<(), GitManagerError> {
    use owo_colors::OwoColorize;

    let r = reg(kernel)?;
    let lock = r.lock()
        .expect("theme: registry Mutex poisoned");
    let c = &lock.current_colors();

    let show = |label: &str, color: owo_colors::AnsiColors| -> String {
        let sample = format!("  ██████  {label}  ");
        if colors_enabled() {
            sample.color(color).to_string()
        } else {
            sample
        }
    };

    println!();
    println!("  {}  \"{}\"", bold("Theme Preview –"), green(&lock.current().name));
    println!();
    println!("  {}", show("primary",   c.primary));
    println!("  {}", show("secondary", c.secondary));
    println!("  {}", show("accent",    c.accent));
    println!("  {}", show("success",   c.success));
    println!("  {}", show("error",     c.error));
    println!("  {}", show("warning",   c.warning));
    println!("  {}", show("info",      c.info));
    println!("  {}", show("banner",    c.banner));
    println!();
    println!("  {} {} {} {}", dim("Prefixes:"),
        success_prefix(), error_prefix(), warn_prefix());
    println!();
    println!("  {} Use {} to keep this theme",
        dim("→"), dim("git-zyrix theme list"));
    println!("  {} Use {} to switch",
        dim("→"), dim("git-zyrix theme set <slug>"));
    Ok(())
}

fn colors_enabled() -> bool {
    std::env::var("NO_COLOR").is_err()
        && std::env::var("TERM").map_or(true, |t| t != "dumb")
        && atty::is(atty::Stream::Stdout)
}
