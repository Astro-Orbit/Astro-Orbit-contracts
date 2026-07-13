use soroban_sdk::{contracttype, Address, BytesN, Symbol};

pub const STATUS_ACTIVE: u32 = 0;
pub const STATUS_ARCHIVED: u32 = 1;

#[contracttype]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OrgInfo {
    pub owner: Address,
    pub metadata_hash: BytesN<32>,
    pub created_at: u64,
    pub status: u32,
}

#[contracttype]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProjectInfo {
    pub org_id: u32,
    pub project_hash: BytesN<32>,
    pub created_at: u64,
    pub status: u32,
}

#[contracttype]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeploymentInfo {
    pub contract_id: BytesN<32>,
    pub artifact_hash: BytesN<32>,
    pub version: u32,
    pub timestamp: u64,
    pub network: Symbol,
}

#[contracttype]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Role {
    Owner = 0,
    Admin = 1,
    Developer = 2,
    Viewer = 3,
}
