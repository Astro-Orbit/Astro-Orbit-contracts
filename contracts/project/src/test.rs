#![cfg(test)]

use soroban_sdk::{testutils::Address as _, Address, BytesN, Env};

use crate::{ProjectContract, ProjectContractClient};

fn setup_env() -> (Env, ProjectContractClient<'static>, Address) {
    let env = Env::default();
    let contract_id = env.register(ProjectContract, ());
    let client = ProjectContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    env.mock_all_auths();
    client.init(&admin);

    (env, client, admin)
}

#[test]
fn test_register() {
    let (_env, client, _admin) = setup_env();
    let h = BytesN::from_array(&_env, &[0u8; 32]);

    let project = client.register(&1, &1, &h);
    assert_eq!(project.status, 0);
    assert_eq!(project.org_id, 1);
}

#[test]
fn test_get() {
    let (_env, client, _admin) = setup_env();
    let h = BytesN::from_array(&_env, &[0u8; 32]);

    client.register(&1, &1, &h);
    let project = client.get(&1);
    assert_eq!(project.status, 0);
}

#[test]
fn test_get_not_found() {
    let (_env, _client, _admin) = setup_env();
    let client = ProjectContractClient::new(&_env, &_env.register(ProjectContract, ()));

    let result = client.try_get(&999);
    assert!(result.is_err());
}

#[test]
fn test_archive() {
    let (_env, client, _admin) = setup_env();
    let h = BytesN::from_array(&_env, &[0u8; 32]);

    client.register(&1, &1, &h);
    client.archive(&1);

    let project = client.get(&1);
    assert_eq!(project.status, 1);
}

#[test]
fn test_register_duplicate() {
    let (_env, client, _admin) = setup_env();
    let h = BytesN::from_array(&_env, &[0u8; 32]);

    client.register(&1, &1, &h);
    let result = client.try_register(&1, &1, &h);
    assert!(result.is_err());
}
