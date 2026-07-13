#![cfg(test)]

use soroban_sdk::{testutils::Address as _, Address, Env};

use astro_orbit_shared::types::Role;

use crate::{PermissionsContract, PermissionsContractClient};

fn setup_env() -> (Env, PermissionsContractClient<'static>, Address) {
    let env = Env::default();
    let contract_id = env.register(PermissionsContract, ());
    let client = PermissionsContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    env.mock_all_auths();
    client.init(&admin);

    (env, client, admin)
}

#[test]
fn test_grant_role() {
    let (env, client, _admin) = setup_env();
    let user = Address::generate(&env);

    env.mock_all_auths();
    client.grant_role(&1, &user, &Role::Developer);

    let role = client.get_role(&1, &user);
    assert_eq!(role, Role::Developer);
}

#[test]
fn test_has_role() {
    let (env, client, _admin) = setup_env();
    let user = Address::generate(&env);

    env.mock_all_auths();
    client.grant_role(&1, &user, &Role::Viewer);

    assert!(client.has_role(&1, &user, &Role::Viewer));
    assert!(!client.has_role(&1, &user, &Role::Admin));
}

#[test]
fn test_revoke_role() {
    let (env, client, _admin) = setup_env();
    let user = Address::generate(&env);

    env.mock_all_auths();
    client.grant_role(&1, &user, &Role::Developer);
    client.revoke_role(&1, &user);

    let has = client.has_role(&1, &user, &Role::Developer);
    assert!(!has);
}

#[test]
fn test_revoke_not_found() {
    let (env, _client, _admin) = setup_env();
    let client = PermissionsContractClient::new(&env, &env.register(PermissionsContract, ()));
    let user = Address::generate(&env);

    let result = client.try_revoke_role(&1, &user);
    assert!(result.is_err());
}

#[test]
fn test_get_role_not_found() {
    let (env, client, _admin) = setup_env();
    let user = Address::generate(&env);

    let result = client.try_get_role(&1, &user);
    assert!(result.is_err());
}

#[test]
fn test_grant_duplicate() {
    let (env, client, _admin) = setup_env();
    let user = Address::generate(&env);

    env.mock_all_auths();
    client.grant_role(&1, &user, &Role::Developer);

    let result = client.try_grant_role(&1, &user, &Role::Admin);
    assert!(result.is_err());
}

#[test]
fn test_hierarchy() {
    let (_env, client, _admin) = setup_env();
    let owner = Address::generate(&_env);
    let admin = Address::generate(&_env);
    let dev = Address::generate(&_env);

    _env.mock_all_auths();
    client.grant_role(&1, &owner, &Role::Owner);
    client.grant_role(&1, &admin, &Role::Admin);
    client.grant_role(&1, &dev, &Role::Developer);

    assert!(client.has_role(&1, &owner, &Role::Viewer));
    assert!(client.has_role(&1, &owner, &Role::Owner));
    assert!(!client.has_role(&1, &dev, &Role::Admin));
}
