#![cfg(test)]

use soroban_sdk::{testutils::Address as _, Address, BytesN, Env};

use crate::{OrganizationContract, OrganizationContractClient};

fn setup_env() -> (Env, OrganizationContractClient<'static>, Address) {
    let env = Env::default();
    let contract_id = env.register(OrganizationContract, ());
    let client = OrganizationContractClient::new(&env, &contract_id);
    let owner = Address::generate(&env);
    env.mock_all_auths();
    client.init(&owner);
    (env, client, owner)
}

fn hash(env: &Env) -> BytesN<32> {
    BytesN::from_array(env, &[0u8; 32])
}

fn create_org(env: &Env, client: &OrganizationContractClient<'static>, id: u32, _owner: &Address) {
    env.mock_all_auths();
    client.create(&id, &hash(env));
}

#[test]
fn test_create() {
    let (env, client, owner) = setup_env();
    create_org(&env, &client, 1, &owner);
    let org = client.get(&1);
    assert_eq!(org.status, 0);
    assert_eq!(org.owner, owner);
}

#[test]
fn test_create_duplicate() {
    let (env, client, _owner) = setup_env();
    create_org(&env, &client, 1, &_owner);
    let result = client.try_create(&1, &hash(&env));
    assert!(result.is_err());
}

#[test]
fn test_get() {
    let (env, client, _owner) = setup_env();
    create_org(&env, &client, 1, &_owner);
    let org = client.get(&1);
    assert_eq!(org.status, 0);
}

#[test]
fn test_get_not_found() {
    let (env, _client, _owner) = setup_env();
    let client = OrganizationContractClient::new(&env, &env.register(OrganizationContract, ()));
    let result = client.try_get(&999);
    assert!(result.is_err());
}

#[test]
fn test_transfer() {
    let (env, client, _owner) = setup_env();
    create_org(&env, &client, 1, &_owner);
    let new_owner = Address::generate(&env);
    env.mock_all_auths();
    client.transfer(&1, &new_owner);
    let org = client.get(&1);
    assert_eq!(org.owner, new_owner);
}

#[test]
fn test_transfer_unauthorized() {
    let (env, client, _owner) = setup_env();
    client.create(&1, &hash(&env));

    // Replace recording auth manager with enforcing one (no entries)
    // so subsequent require_auth calls will fail for unapproved addresses.
    env.set_auths(&[]);

    let unauthorized = Address::generate(&env);
    let result = client.try_transfer(&1, &unauthorized);
    assert!(result.is_err());
}

#[test]
fn test_archive() {
    let (env, client, owner) = setup_env();
    create_org(&env, &client, 1, &owner);
    client.archive(&1);
    let org = client.get(&1);
    assert_eq!(org.status, 1);
}

#[test]
fn test_update_metadata() {
    let (env, client, _owner) = setup_env();
    create_org(&env, &client, 1, &_owner);
    let new_hash = BytesN::from_array(&env, &[1u8; 32]);
    client.update_metadata(&1, &new_hash);
    let org = client.get(&1);
    assert_eq!(org.metadata_hash, new_hash);
}
