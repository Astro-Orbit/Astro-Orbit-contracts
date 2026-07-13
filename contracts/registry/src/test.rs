#![cfg(test)]

use soroban_sdk::{testutils::Address as _, Address, BytesN, Env, Symbol};

use astro_orbit_shared::types::{DeploymentInfo, OrgInfo, ProjectInfo, STATUS_ACTIVE};

use crate::{RegistryContract, RegistryContractClient};

fn setup_env() -> (Env, RegistryContractClient<'static>) {
    let env = Env::default();
    let contract_id = env.register(RegistryContract, ());
    let client = RegistryContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let org_addr = Address::generate(&env);
    let proj_addr = Address::generate(&env);
    let dep_addr = Address::generate(&env);

    env.mock_all_auths();
    client.init(&admin, &org_addr, &proj_addr, &dep_addr);

    (env, client)
}

fn make_org_info(env: &Env) -> OrgInfo {
    OrgInfo {
        owner: Address::generate(env),
        metadata_hash: BytesN::from_array(env, &[0u8; 32]),
        created_at: 1000,
        status: STATUS_ACTIVE,
    }
}

fn make_project_info(env: &Env) -> ProjectInfo {
    ProjectInfo {
        org_id: 1,
        project_hash: BytesN::from_array(env, &[1u8; 32]),
        created_at: 1000,
        status: STATUS_ACTIVE,
    }
}

fn make_deployment_info(env: &Env) -> DeploymentInfo {
    DeploymentInfo {
        contract_id: BytesN::from_array(env, &[2u8; 32]),
        artifact_hash: BytesN::from_array(env, &[3u8; 32]),
        version: 1,
        timestamp: 1000,
        network: Symbol::new(env, "testnet"),
    }
}

#[test]
fn test_index_and_lookup_org() {
    let (_env, client) = setup_env();
    let info = make_org_info(&_env);

    client.index_org(&1, &info);
    let retrieved = client.lookup_org(&1);
    assert_eq!(retrieved.owner, info.owner);
}

#[test]
fn test_index_and_lookup_project() {
    let (_env, client) = setup_env();
    let info = make_project_info(&_env);

    client.index_project(&1, &info);
    let retrieved = client.lookup_project(&1);
    assert_eq!(retrieved.org_id, info.org_id);
}

#[test]
fn test_index_and_lookup_deployment() {
    let (_env, client) = setup_env();
    let info = make_deployment_info(&_env);

    client.index_deployment(&1, &1, &info);
    let retrieved = client.lookup_deployment(&1, &1);
    assert_eq!(retrieved.version, info.version);
}

#[test]
fn test_lookup_org_not_found() {
    let (_env, client) = setup_env();
    let result = client.try_lookup_org(&999);
    assert!(result.is_err());
}

#[test]
fn test_lookup_project_not_found() {
    let (_env, client) = setup_env();
    let result = client.try_lookup_project(&999);
    assert!(result.is_err());
}
