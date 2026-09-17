use uuid::Uuid;

pub fn platform_info(platform_id: Uuid) -> (&'static str, &'static str, u16) {
    match platform_id.to_string().as_str() {
        "00000000-0001-0000-0000-000000000001" => ("github",       "github.com",        22),
        "00000000-0002-0000-0000-000000000001" => ("gitlab",       "gitlab.com",        22),
        "00000000-0003-0000-0000-000000000001" => ("bitbucket",    "bitbucket.org",     22),
        "00000000-0004-0000-0000-000000000001" => ("azure_devops", "ssh.dev.azure.com", 22),
        "00000000-0005-0000-0000-000000000001" => ("sourceforge",  "git.code.sf.net",   22),
        "00000000-0006-0000-0000-000000000001" => ("self_hosted",  "localhost",         22),
        "00000000-0007-0000-0000-000000000001" => ("cloud_storage","",                  22),
        "00000000-0008-0000-0000-000000000001" => ("local_path",   "",                  22),
        _                                       => ("custom",       "github.com",        22),
    }
}

pub fn platform_display_name(platform_id: Uuid) -> Option<String> {
    match platform_id.to_string().as_str() {
        "00000000-0001-0000-0000-000000000001" => Some("GitHub".to_string()),
        "00000000-0002-0000-0000-000000000001" => Some("GitLab".to_string()),
        "00000000-0003-0000-0000-000000000001" => Some("Bitbucket".to_string()),
        "00000000-0004-0000-0000-000000000001" => Some("Azure DevOps".to_string()),
        "00000000-0005-0000-0000-000000000001" => Some("SourceForge".to_string()),
        "00000000-0006-0000-0000-000000000001" => Some("Self-Hosted".to_string()),
        "00000000-0007-0000-0000-000000000001" => Some("Cloud Storage".to_string()),
        "00000000-0008-0000-0000-000000000001" => Some("Local Path".to_string()),
        _                                       => None,
    }
}
