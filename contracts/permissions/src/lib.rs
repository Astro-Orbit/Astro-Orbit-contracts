#![no_std]

use soroban_sdk::{contract, contractimpl, contracttype, Address, Env};

use astro_orbit_shared::errors::ContractError;
use astro_orbit_shared::events;
use astro_orbit_shared::types::Role;

#[contracttype]
pub enum PermDataKey {
    Admin,
    Role(u32, Address),
}

#[contract]
pub struct PermissionsContract;

#[contractimpl]
impl PermissionsContract {
    pub fn init(env: Env, admin: Address) {
        if env.storage().instance().has(&PermDataKey::Admin) {
            panic!("already initialized");
        }
        env.storage().instance().set(&PermDataKey::Admin, &admin);
    }

    pub fn grant_role(
        env: Env,
        org_id: u32,
        user: Address,
        role: Role,
    ) -> Result<(), ContractError> {
        let admin: Address = env
            .storage()
            .instance()
            .get(&PermDataKey::Admin)
            .ok_or(ContractError::StorageFailure)?;

        admin.require_auth();
        user.require_auth();

        if env
            .storage()
            .instance()
            .has(&PermDataKey::Role(org_id, user.clone()))
        {
            return Err(ContractError::AlreadyExists);
        }

        env.storage()
            .instance()
            .set(&PermDataKey::Role(org_id, user.clone()), &role);
        events::role_granted(&env, org_id, &user, &role);
        Ok(())
    }

    pub fn revoke_role(env: Env, org_id: u32, user: Address) -> Result<(), ContractError> {
        let admin: Address = env
            .storage()
            .instance()
            .get(&PermDataKey::Admin)
            .ok_or(ContractError::StorageFailure)?;

        admin.require_auth();

        if !env
            .storage()
            .instance()
            .has(&PermDataKey::Role(org_id, user.clone()))
        {
            return Err(ContractError::NotFound);
        }

        env.storage()
            .instance()
            .remove(&PermDataKey::Role(org_id, user.clone()));
        events::role_revoked(&env, org_id, &user);
        Ok(())
    }

    pub fn has_role(env: Env, org_id: u32, user: Address, role: Role) -> bool {
        env.storage()
            .instance()
            .get::<_, Role>(&PermDataKey::Role(org_id, user))
            .is_some_and(|stored| stored as u32 <= role as u32)
    }

    pub fn get_role(env: Env, org_id: u32, user: Address) -> Result<Role, ContractError> {
        env.storage()
            .instance()
            .get(&PermDataKey::Role(org_id, user))
            .ok_or(ContractError::NotFound)
    }
}

mod test;
