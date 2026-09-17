use std::sync::Arc;

use uuid::Uuid;

mod common;

/// Test SSH key repository save and find operations.
#[sqlx::test]
async fn test_ssh_key_persistence() {
    let pool = common::setup_test_db().await;

    let repo = Arc::new(
        gm_adapters::persistence::mysql::MySqlSshKeyRepository::new(pool)
    );

    let (account_id, platform_id) = create_test_account().await;
    let key_type = gm_domain::ssh::value_objects::KeyType::Ed25519;

    let key = gm_domain::ssh::entities::SshKey::new(
        account_id,
        "id_ed25519_github_test".to_string(),
        key_type,
        "SHA256:test_fingerprint".to_string(),
        "ssh-ed25519 AAAATEST test@example.com".to_string(),
        "/tmp/.ssh/id_ed25519_github_test".to_string(),
    );

    // Save
    repo.save(&key).await.expect("save should succeed");

    // Find by ID
    let found = repo.find_by_id(key.uuid()).await.expect("find_by_id should succeed");
    assert!(found.is_some(), "Key should be found by UUID");
    assert_eq!(found.unwrap().name(), key.name());

    // List by account
    let keys = repo.list_by_account(account_id).await.expect("list_by_account should succeed");
    assert_eq!(keys.len(), 1, "Should have 1 key for account");
    assert_eq!(keys[0].fingerprint(), "SHA256:test_fingerprint");

    // Deactivate all
    repo.deactivate_all_for_account(account_id).await.expect("deactivate should succeed");
    let active = repo.find_active_for_account(account_id).await.expect("find_active should succeed");
    assert!(active.is_none(), "No key should be active after deactivate_all");
}

/// Test SSH host config repository save and find.
#[sqlx::test]
async fn test_ssh_host_config_persistence() {
    let pool = common::setup_test_db().await;

    let repo = Arc::new(
        gm_adapters::persistence::mysql::MySqlSshHostConfigRepository::new(pool)
    );

    let (account_id, _platform_id) = create_test_account().await;

    let config = gm_domain::ssh::entities::SshHostConfig::new(
        account_id,
        Uuid::new_v4(),
        "github.com-test".to_string(),
        "github.com".to_string(),
        "/tmp/.ssh/id_ed25519_github_test".to_string(),
        22,
    );

    repo.save(&config).await.expect("save should succeed");

    let found = repo.find_by_account(account_id).await.expect("find should succeed");
    assert!(found.is_some(), "Host config should be found by account");
    assert_eq!(found.unwrap().host_alias(), "github.com-test");
}

/// Helper: creates an account in the DB and returns (account_id, platform_id).
async fn create_test_account() -> (Uuid, Uuid) {
    let pool = common::setup_test_db().await;
    let svc = common::create_account_service(pool);

    let result = svc.add_account(
        "ssh-test".to_string(),
        common::GITHUB_PLATFORM_ID,
        "sshtest".to_string(),
        "ssh@example.com".to_string(),
        gm_domain::accounts::value_objects::AuthMethod::Ssh,
    ).await.expect("add_account should succeed");

    (result.account.uuid(), result.account.platform_id())
}
