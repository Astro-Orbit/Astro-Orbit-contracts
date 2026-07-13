#![no_std]

use soroban_sdk::{contract, contractimpl, contracttype, Address, BytesN, Env, Symbol};

use astro_orbit_shared::auth;
use astro_orbit_shared::errors::ContractError;
use astro_orbit_shared::events;
use astro_orbit_shared::types::DeploymentInfo;

#[contracttype]
pub enum DepDataKey {
    Config,
    Deployment(u32, u32),
    DeploymentCount(u32),
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[contracttype]
pub struct DepConfig {
    pub admin: Address,
}

#[contract]
pub struct DeploymentContract;

#[contractimpl]
impl DeploymentContract {
    pub fn init(env: Env, admin: Address) {
        if env.storage().instance().has(&DepDataKey::Config) {
            panic!("already initialized");
        }
        let cfg = DepConfig { admin };
        env.storage().instance().set(&DepDataKey::Config, &cfg);
    }

    pub fn record(
        env: Env,
        project_id: u32,
        contract_id: BytesN<32>,
        artifact_hash: BytesN<32>,
        network: Symbol,
    ) -> Result<DeploymentInfo, ContractError> {
        let cfg: DepConfig = env
            .storage()
            .instance()
            .get(&DepDataKey::Config)
            .ok_or(ContractError::StorageFailure)?;

        auth::authorize(&cfg.admin)?;

        let count: u32 = env
            .storage()
            .instance()
            .get(&DepDataKey::DeploymentCount(project_id))
            .unwrap_or(0);

        let version = count + 1;
        let timestamp = env.ledger().timestamp();

        let deployment = DeploymentInfo {
            contract_id,
            artifact_hash,
            version,
            timestamp,
            network,
        };

        env.storage()
            .instance()
            .set(&DepDataKey::Deployment(project_id, version), &deployment);
        env.storage()
            .instance()
            .set(&DepDataKey::DeploymentCount(project_id), &version);

        events::deployment_recorded(&env, project_id, version);
        Ok(deployment)
    }

    pub fn get(env: Env, project_id: u32, version: u32) -> Result<DeploymentInfo, ContractError> {
        env.storage()
            .instance()
            .get(&DepDataKey::Deployment(project_id, version))
            .ok_or(ContractError::NotFound)
    }

    pub fn count(env: Env, project_id: u32) -> u32 {
        env.storage()
            .instance()
            .get(&DepDataKey::DeploymentCount(project_id))
            .unwrap_or(0)
    }
}

mod test;
