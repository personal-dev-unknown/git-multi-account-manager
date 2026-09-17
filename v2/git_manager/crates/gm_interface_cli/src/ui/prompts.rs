// crates/gm_interface_cli/src/ui/prompts.rs
// Reusable interactive prompt builders for the account-add wizard and similar flows.

use gm_shared::errors::GitManagerError;

/// Prompts for a non-empty string value. Reprompts if the user submits blank.
pub fn prompt_required(label: &str, default: Option<&str>) -> Result<String, GitManagerError> {
    use dialoguer::Input;
    let mut input: Input<String> = Input::new().with_prompt(label);
    if let Some(d) = default { input = input.default(d.to_string()); }
    input.interact_text().map_err(|e| GitManagerError::Other(format!("prompt: {e}")))
}

/// Prompts for a selection from a list of options.
pub fn prompt_select(label: &str, options: &[&str]) -> Result<usize, GitManagerError> {
    use dialoguer::Select;
    Select::new()
        .with_prompt(label)
        .items(options)
        .interact()
        .map_err(|e| GitManagerError::Other(format!("select: {e}")))
}

/// Prompts for a password (input hidden).
pub fn prompt_password(label: &str) -> Result<String, GitManagerError> {
    use dialoguer::Password;
    Password::new()
        .with_prompt(label)
        .interact()
        .map_err(|e| GitManagerError::Other(format!("password: {e}")))
}

/// Prompts for a yes/no confirmation.
pub fn confirm(label: &str, default: bool) -> Result<bool, GitManagerError> {
    use dialoguer::Confirm;
    Confirm::new()
        .with_prompt(label)
        .default(default)
        .interact()
        .map_err(|e| GitManagerError::Other(format!("confirm: {e}")))
}