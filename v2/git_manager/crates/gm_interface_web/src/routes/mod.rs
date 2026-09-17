pub mod accounts;
pub mod api;
pub mod clone;
pub mod git_ops;
pub mod index;
pub mod repositories;
pub mod ssh;
pub mod static_files;

use uuid::Uuid;

/// Maps platform slug to the fixed seed UUID (matches seed migration).
pub fn platform_uuid_for(platform: &str) -> Uuid {
    match platform {
        "github"       => Uuid::parse_str("00000000-0001-0000-0000-000000000001").unwrap(),
        "gitlab"       => Uuid::parse_str("00000000-0002-0000-0000-000000000001").unwrap(),
        "bitbucket"    => Uuid::parse_str("00000000-0003-0000-0000-000000000001").unwrap(),
        "azure_devops" => Uuid::parse_str("00000000-0004-0000-0000-000000000001").unwrap(),
        "sourceforge"  => Uuid::parse_str("00000000-0005-0000-0000-000000000001").unwrap(),
        _              => Uuid::parse_str("00000000-0006-0000-0000-000000000001").unwrap(),
    }
}
