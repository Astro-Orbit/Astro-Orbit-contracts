#![cfg(test)]

use soroban_sdk::{testutils::Address as _, Address, BytesN, Env, Symbol};

use crate::{DeploymentContract, DeploymentContractClient};

fn setup_env() -> (Env, DeploymentContractClient<'static>, Address) {
    let env = Env::default();
    let contract_id = env.register(DeploymentContract, ());
    let client = DeploymentContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    env.mock_all_auths();
    client.init(&admin);

    (env, client, admin)
}

#[test]
fn test_record() {
    let (_env, client, _admin) = setup_env();
    let cid = BytesN::from_array(&_env, &[2u8; 32]);
    let ah = BytesN::from_array(&_env, &[3u8; 32]);
    let net = Symbol::new(&_env, "testnet");

    let dep = client.record(&1, &cid, &ah, &net);
    assert_eq!(dep.version, 1);
    assert_eq!(dep.network, net);
}

#[test]
fn test_get() {
    let (_env, client, _admin) = setup_env();
    let cid = BytesN::from_array(&_env, &[2u8; 32]);
    let ah = BytesN::from_array(&_env, &[3u8; 32]);
    let net = Symbol::new(&_env, "testnet");

    client.record(&1, &cid, &ah, &net);
    let dep = client.get(&1, &1);
    assert_eq!(dep.version, 1);
}

#[test]
fn test_get_not_found() {
    let (_env, _client, _admin) = setup_env();
    let client = DeploymentContractClient::new(&_env, &_env.register(DeploymentContract, ()));

    let result = client.try_get(&999, &1);
    assert!(result.is_err());
}

#[test]
fn test_count() {
    let (_env, client, _admin) = setup_env();
    let cid = BytesN::from_array(&_env, &[2u8; 32]);
    let ah = BytesN::from_array(&_env, &[3u8; 32]);
    let net = Symbol::new(&_env, "testnet");

    assert_eq!(client.count(&1), 0);
    client.record(&1, &cid, &ah, &net);
    assert_eq!(client.count(&1), 1);
    client.record(&1, &cid, &ah, &net);
    assert_eq!(client.count(&1), 2);
}

#[test]
fn test_multiple_versions() {
    let (_env, client, _admin) = setup_env();
    let cid = BytesN::from_array(&_env, &[2u8; 32]);
    let net = Symbol::new(&_env, "testnet");

    let ah1 = BytesN::from_array(&_env, &[10u8; 32]);
    let dep1 = client.record(&1, &cid, &ah1, &net);
    assert_eq!(dep1.version, 1);

    let ah2 = BytesN::from_array(&_env, &[20u8; 32]);
    let dep2 = client.record(&1, &cid, &ah2, &net);
    assert_eq!(dep2.version, 2);

    let dep1_read = client.get(&1, &1);
    assert_eq!(dep1_read.artifact_hash, ah1);

    let dep2_read = client.get(&1, &2);
    assert_eq!(dep2_read.artifact_hash, ah2);
}
