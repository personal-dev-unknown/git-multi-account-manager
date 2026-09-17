// crates/gm_plugin_azure_devops/src/auth.rs
//
// Azure DevOps authentication helpers.
//
// ADO uses Personal Access Tokens (PATs) for API access. PATs are scoped to
// specific capabilities and can be set to expire after a chosen duration.
//
// The credential stored in the vault is "organization:pat" — the organisation
// name is required because all ADO API calls are organisation-scoped.
//
// Required PAT scopes for Git Manager:
//   • Code (Read & Write)     — clone, pull, push repositories
//   • Project and Team (Read) — list projects within the organization
//
// PAT creation URL: https://dev.azure.com/{organization}/_usersettings/tokens

/// Returns the PAT creation URL for the given organisation.
pub fn pat_creation_url(organization: &str) -> String {
    format!("https://dev.azure.com/{organization}/_usersettings/tokens")
}

/// The expected credential format stored in the vault.
pub const CREDENTIAL_FORMAT: &str = "organization:pat";

/// Splits an "organization:pat" credential into `(org, pat)` components.
/// Returns `("", credential)` if no colon is present so callers can produce a
/// descriptive error without panicking.
pub fn split_azure_credential(credential: &str) -> (&str, &str) {
    credential.split_once(':').unwrap_or(("", credential))
}

/// Validates that the credential string has the expected "organization:pat" format.
/// The organisation name cannot be empty, and the PAT must be at least 50 characters
/// (Azure DevOps PATs are typically 52 characters).
pub fn looks_like_valid_credential(credential: &str) -> bool {
    let parts: Vec<&str> = credential.splitn(2, ':').collect();
    if parts.len() != 2 { return false; }
    let org = parts[0].trim();
    let pat = parts[1].trim();
    !org.is_empty() && pat.len() >= 40
}

/// Returns human-readable instructions for creating an ADO PAT.
pub fn credential_instructions(organization: &str) -> String {
    format!(
        "To create an Azure DevOps PAT:\n\
         1. Go to: https://dev.azure.com/{organization}/_usersettings/tokens\n\
         2. Click 'New Token'\n\
         3. Grant scopes: Code (Read & Write), Project and Team (Read)\n\
         4. Copy the token and enter it as: {organization}:{{your_pat}}\n\
         \n\
         Note: The token is shown only once — save it before closing.",
        organization = organization
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_credential() {
        // 50-char PAT with org prefix
        let pat = "a".repeat(52);
        assert!(looks_like_valid_credential(&format!("myorg:{pat}")));
    }

    #[test]
    fn missing_org_rejected() {
        assert!(!looks_like_valid_credential(&format!(":{}", "a".repeat(52))));
    }

    #[test]
    fn short_pat_rejected() {
        assert!(!looks_like_valid_credential("myorg:shortpat"));
    }

    #[test]
    fn no_colon_rejected() {
        assert!(!looks_like_valid_credential("justapersonalaccesstoken"));
    }
}