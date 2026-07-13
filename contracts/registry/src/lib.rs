#![no_std]

use soroban_sdk::{contract, contractimpl, contracttype, Address, Env};

use astro_orbit_shared::errors::ContractError;
use astro_orbit_shared::types::{DeploymentInfo, OrgInfo, ProjectInfo};

#[contracttype]
pub enum RegDataKey {
    Config,
    OrgIndex(u32),
    ProjectIndex(u32),
    DeploymentIndex(u32, u32),
    DeploymentCount(u32),
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[contracttype]
pub struct RegConfig {
    pub admin: Address,
    pub org_contract: Address,
    pub project_contract: Address,
    pub deployment_contract: Address,
}

#[contract]
pub struct RegistryContract;

#[contractimpl]
impl RegistryContract {
    pub fn init(
        env: Env,
        admin: Address,
        org_contract: Address,
        project_contract: Address,
        deployment_contract: Address,
    ) {
        if env.storage().instance().has(&RegDataKey::Config) {
            panic!("already initialized");
        }
        let cfg = RegConfig {
            admin,
            org_contract,
            project_contract,
            deployment_contract,
        };
        env.storage().instance().set(&RegDataKey::Config, &cfg);
    }

    pub fn index_org(env: Env, org_id: u32, info: OrgInfo) -> Result<(), ContractError> {
        let cfg: RegConfig = env
            .storage()
            .instance()
            .get(&RegDataKey::Config)
            .ok_or(ContractError::StorageFailure)?;
        cfg.admin.require_auth();

        env.storage()
            .instance()
            .set(&RegDataKey::OrgIndex(org_id), &info);
        Ok(())
    }

    pub fn index_project(
        env: Env,
        project_id: u32,
        info: ProjectInfo,
    ) -> Result<(), ContractError> {
        let cfg: RegConfig = env
            .storage()
            .instance()
            .get(&RegDataKey::Config)
            .ok_or(ContractError::StorageFailure)?;
        cfg.admin.require_auth();

        env.storage()
            .instance()
            .set(&RegDataKey::ProjectIndex(project_id), &info);
        Ok(())
    }

    pub fn index_deployment(
        env: Env,
        project_id: u32,
        version: u32,
        info: DeploymentInfo,
    ) -> Result<(), ContractError> {
        let cfg: RegConfig = env
            .storage()
            .instance()
            .get(&RegDataKey::Config)
            .ok_or(ContractError::StorageFailure)?;
        cfg.admin.require_auth();

        let idx = env
            .storage()
            .instance()
            .get::<_, u32>(&RegDataKey::DeploymentCount(project_id))
            .unwrap_or(0);

        if version > idx {
            env.storage()
                .instance()
                .set(&RegDataKey::DeploymentCount(project_id), &version);
        }

        env.storage()
            .instance()
            .set(&RegDataKey::DeploymentIndex(project_id, version), &info);
        Ok(())
    }

    pub fn lookup_org(env: Env, org_id: u32) -> Result<OrgInfo, ContractError> {
        env.storage()
            .instance()
            .get(&RegDataKey::OrgIndex(org_id))
            .ok_or(ContractError::NotFound)
    }

    pub fn lookup_project(env: Env, project_id: u32) -> Result<ProjectInfo, ContractError> {
        env.storage()
            .instance()
            .get(&RegDataKey::ProjectIndex(project_id))
            .ok_or(ContractError::NotFound)
    }

    pub fn lookup_deployment(
        env: Env,
        project_id: u32,
        version: u32,
    ) -> Result<DeploymentInfo, ContractError> {
        env.storage()
            .instance()
            .get(&RegDataKey::DeploymentIndex(project_id, version))
            .ok_or(ContractError::NotFound)
    }
}

mod test;
