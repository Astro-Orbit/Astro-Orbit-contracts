#![no_std]

use soroban_sdk::{contract, contractimpl, contracttype, Address, BytesN, Env};

use astro_orbit_shared::auth;
use astro_orbit_shared::errors::ContractError;
use astro_orbit_shared::events;
use astro_orbit_shared::types::{ProjectInfo, STATUS_ACTIVE};

#[contracttype]
pub enum ProjDataKey {
    Config,
    Project(u32),
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[contracttype]
pub struct ProjConfig {
    pub admin: Address,
}

#[contract]
pub struct ProjectContract;

#[contractimpl]
impl ProjectContract {
    pub fn init(env: Env, admin: Address) {
        if env.storage().instance().has(&ProjDataKey::Config) {
            panic!("already initialized");
        }
        let cfg = ProjConfig { admin };
        env.storage().instance().set(&ProjDataKey::Config, &cfg);
    }

    pub fn register(
        env: Env,
        project_id: u32,
        org_id: u32,
        project_hash: BytesN<32>,
    ) -> Result<ProjectInfo, ContractError> {
        let cfg: ProjConfig = env
            .storage()
            .instance()
            .get(&ProjDataKey::Config)
            .ok_or(ContractError::StorageFailure)?;

        auth::authorize(&cfg.admin)?;

        if env
            .storage()
            .instance()
            .has(&ProjDataKey::Project(project_id))
        {
            return Err(ContractError::AlreadyExists);
        }

        let project = ProjectInfo {
            org_id,
            project_hash,
            created_at: env.ledger().timestamp(),
            status: STATUS_ACTIVE,
        };

        env.storage()
            .instance()
            .set(&ProjDataKey::Project(project_id), &project);
        events::project_created(&env, project_id, org_id);
        Ok(project)
    }

    pub fn archive(env: Env, project_id: u32) -> Result<ProjectInfo, ContractError> {
        let cfg: ProjConfig = env
            .storage()
            .instance()
            .get(&ProjDataKey::Config)
            .ok_or(ContractError::StorageFailure)?;

        let mut project = read_project(&env, project_id)?;
        auth::authorize(&cfg.admin)?;
        auth::check_active(project.status)?;

        project.status = astro_orbit_shared::types::STATUS_ARCHIVED;
        env.storage()
            .instance()
            .set(&ProjDataKey::Project(project_id), &project);

        events::project_archived(&env, project_id);
        Ok(project)
    }

    pub fn get(env: Env, project_id: u32) -> Result<ProjectInfo, ContractError> {
        read_project(&env, project_id)
    }
}

fn read_project(env: &Env, project_id: u32) -> Result<ProjectInfo, ContractError> {
    env.storage()
        .instance()
        .get(&ProjDataKey::Project(project_id))
        .ok_or(ContractError::NotFound)
}

mod test;
