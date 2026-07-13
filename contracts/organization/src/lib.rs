#![no_std]

use soroban_sdk::{contract, contractimpl, contracttype, Address, BytesN, Env};

use astro_orbit_shared::auth;
use astro_orbit_shared::errors::ContractError;
use astro_orbit_shared::events;
use astro_orbit_shared::types::OrgInfo;
use astro_orbit_shared::types::STATUS_ACTIVE;

#[contracttype]
pub enum OrgDataKey {
    Config,
    Org(u32),
}

#[contract]
pub struct OrganizationContract;

#[contractimpl]
impl OrganizationContract {
    pub fn init(env: Env, owner: Address) {
        if env.storage().instance().has(&OrgDataKey::Config) {
            panic!("already initialized");
        }
        env.storage().instance().set(&OrgDataKey::Config, &owner);
    }

    pub fn create(
        env: Env,
        org_id: u32,
        metadata_hash: BytesN<32>,
    ) -> Result<OrgInfo, ContractError> {
        let owner: Address = env
            .storage()
            .instance()
            .get(&OrgDataKey::Config)
            .ok_or(ContractError::StorageFailure)?;

        auth::authorize(&owner)?;

        if env.storage().instance().has(&OrgDataKey::Org(org_id)) {
            return Err(ContractError::AlreadyExists);
        }

        let org = OrgInfo {
            owner: owner.clone(),
            metadata_hash,
            created_at: env.ledger().timestamp(),
            status: STATUS_ACTIVE,
        };

        env.storage().instance().set(&OrgDataKey::Org(org_id), &org);
        events::org_created(&env, org_id, &owner);
        Ok(org)
    }

    pub fn transfer(env: Env, org_id: u32, new_owner: Address) -> Result<OrgInfo, ContractError> {
        let mut org = read_org(&env, org_id)?;
        auth::authorize(&org.owner)?;
        auth::check_active(org.status)?;

        let old_owner = org.owner.clone();
        org.owner = new_owner.clone();
        env.storage().instance().set(&OrgDataKey::Org(org_id), &org);

        events::org_transferred(&env, org_id, &old_owner, &new_owner);
        Ok(org)
    }

    pub fn update_metadata(
        env: Env,
        org_id: u32,
        new_hash: BytesN<32>,
    ) -> Result<OrgInfo, ContractError> {
        let mut org = read_org(&env, org_id)?;
        auth::authorize(&org.owner)?;
        auth::check_active(org.status)?;

        org.metadata_hash = new_hash;
        env.storage().instance().set(&OrgDataKey::Org(org_id), &org);
        Ok(org)
    }

    pub fn archive(env: Env, org_id: u32) -> Result<OrgInfo, ContractError> {
        let mut org = read_org(&env, org_id)?;
        auth::authorize(&org.owner)?;
        auth::check_active(org.status)?;

        org.status = astro_orbit_shared::types::STATUS_ARCHIVED;
        env.storage().instance().set(&OrgDataKey::Org(org_id), &org);

        events::org_archived(&env, org_id);
        Ok(org)
    }

    pub fn get(env: Env, org_id: u32) -> Result<OrgInfo, ContractError> {
        read_org(&env, org_id)
    }
}

fn read_org(env: &Env, org_id: u32) -> Result<OrgInfo, ContractError> {
    env.storage()
        .instance()
        .get(&OrgDataKey::Org(org_id))
        .ok_or(ContractError::NotFound)
}

mod test;
