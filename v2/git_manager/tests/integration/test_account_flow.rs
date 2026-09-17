use uuid::Uuid;

mod common;

/// Test basic account creation and listing.
#[sqlx::test]
async fn test_create_and_list_account() {
    let pool = common::setup_test_db().await;
    let svc = common::create_account_service(pool);

    let alias = "test-user".to_string();
    let username = "testuser".to_string();
    let email = "test@example.com".to_string();

    let result = svc.add_account(
        alias.clone(),
        common::GITHUB_PLATFORM_ID,
        username.clone(),
        email.clone(),
        gm_domain::accounts::value_objects::AuthMethod::Ssh,
    ).await;

    assert!(result.is_ok(), "add_account should succeed: {:?}", result.err());
    let added = result.unwrap();
    assert_eq!(added.account.alias(), alias);
    assert_eq!(added.account.email(), email);

    // Verify the account is in the list
    let accounts = svc.list_accounts().await.expect("list_accounts should succeed");
    assert!(!accounts.is_empty(), "Should have at least one account");
    assert!(accounts.iter().any(|a| a.alias() == alias));
}

/// Test that duplicate aliases on the same platform are rejected.
#[sqlx::test]
async fn test_duplicate_alias_rejected() {
    let pool = common::setup_test_db().await;
    let svc = common::create_account_service(pool);

    // First account
    svc.add_account(
        "personal".to_string(),
        common::GITHUB_PLATFORM_ID,
        "alice".to_string(),
        "alice@example.com".to_string(),
        gm_domain::accounts::value_objects::AuthMethod::Ssh,
    ).await.expect("First account should succeed");

    // Second with same alias and platform — should fail
    let dup = svc.add_account(
        "personal".to_string(),
        common::GITHUB_PLATFORM_ID,
        "alice2".to_string(),
        "alice2@example.com".to_string(),
        gm_domain::accounts::value_objects::AuthMethod::Ssh,
    ).await;

    assert!(dup.is_err(), "Duplicate alias should be rejected");
}

/// Test that the same alias on different platforms is allowed.
#[sqlx::test]
async fn test_same_alias_different_platform_allowed() {
    let pool = common::setup_test_db().await;
    let svc = common::create_account_service(pool);

    let gitlab_id = uuid::Uuid::from_u128(0x00000000_0002_0000_0000_000000000001);

    // GitHub account
    let gh = svc.add_account(
        "work".to_string(),
        common::GITHUB_PLATFORM_ID,
        "alice".to_string(),
        "alice@corp.com".to_string(),
        gm_domain::accounts::value_objects::AuthMethod::Ssh,
    ).await;
    assert!(gh.is_ok(), "GitHub account should succeed");

    // GitLab account with same alias
    let gl = svc.add_account(
        "work".to_string(),
        gitlab_id,
        "alice.gl".to_string(),
        "alice@freelance.com".to_string(),
        gm_domain::accounts::value_objects::AuthMethod::HttpsPat,
    ).await;
    assert!(gl.is_ok(), "GitLab account with same alias should be allowed on different platform");
}

/// Test account retrieval by UUID.
#[sqlx::test]
async fn test_get_account_by_uuid() {
    let pool = common::setup_test_db().await;
    let svc = common::create_account_service(pool);

    let result = svc.add_account(
        "get-test".to_string(),
        common::GITHUB_PLATFORM_ID,
        "getuser".to_string(),
        "get@example.com".to_string(),
        gm_domain::accounts::value_objects::AuthMethod::Ssh,
    ).await.expect("add_account should succeed");

    let account_uuid = result.account.uuid();

    // Find by UUID
    let fetched = svc.get_account(account_uuid).await.expect("get_account should succeed");
    assert!(fetched.is_some(), "Account should be found by UUID");
    assert_eq!(fetched.unwrap().alias(), "get-test");

    // Non-existent UUID
    let missing = svc.get_account(Uuid::nil()).await.expect("get_account for nil UUID should not error");
    assert!(missing.is_none(), "Nil UUID should return None");
}

/// Test account removal.
#[sqlx::test]
async fn test_remove_account() {
    let pool = common::setup_test_db().await;
    let svc = common::create_account_service(pool);

    let result = svc.add_account(
        "to-remove".to_string(),
        common::GITHUB_PLATFORM_ID,
        "removeuser".to_string(),
        "remove@example.com".to_string(),
        gm_domain::accounts::value_objects::AuthMethod::Ssh,
    ).await.expect("add_account should succeed");

    let account_uuid = result.account.uuid();

    // Remove should succeed
    let remove = svc.remove_account(account_uuid).await;
    assert!(remove.is_ok(), "remove_account should succeed: {:?}", remove.err());

    // Verify it's gone
    let accounts = svc.list_accounts().await.expect("list_accounts should succeed");
    assert!(!accounts.iter().any(|a| a.uuid() == account_uuid), "Removed account should not appear in list");
}
