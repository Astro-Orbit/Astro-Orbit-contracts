use soroban_sdk::{symbol_short, Address, Env, Symbol};

use crate::types::Role;

const ORG_CREATED: Symbol = symbol_short!("org_creat");
const ORG_XFERRED: Symbol = symbol_short!("org_xfer");
const ORG_ARCHVD: Symbol = symbol_short!("org_archv");
const PROJ_CREAT: Symbol = symbol_short!("proj_crea");
const PROJ_ARCHV: Symbol = symbol_short!("proj_arch");
const DEPLOY_REC: Symbol = symbol_short!("deploy_re");
const ROLE_GRANT: Symbol = symbol_short!("role_grnt");
const ROLE_REVOK: Symbol = symbol_short!("role_revk");

pub fn org_created(env: &Env, org_id: u32, owner: &Address) {
    env.events().publish((ORG_CREATED, org_id), owner);
}

pub fn org_transferred(env: &Env, org_id: u32, old_owner: &Address, new_owner: &Address) {
    env.events()
        .publish((ORG_XFERRED, org_id, old_owner), new_owner);
}

pub fn org_archived(env: &Env, org_id: u32) {
    env.events().publish((ORG_ARCHVD, org_id), ());
}

pub fn project_created(env: &Env, project_id: u32, org_id: u32) {
    env.events().publish((PROJ_CREAT, project_id), org_id);
}

pub fn project_archived(env: &Env, project_id: u32) {
    env.events().publish((PROJ_ARCHV, project_id), ());
}

pub fn deployment_recorded(env: &Env, project_id: u32, version: u32) {
    env.events().publish((DEPLOY_REC, project_id), version);
}

pub fn role_granted(env: &Env, org_id: u32, user: &Address, role: &Role) {
    env.events().publish((ROLE_GRANT, org_id, user), role);
}

pub fn role_revoked(env: &Env, org_id: u32, user: &Address) {
    env.events().publish((ROLE_REVOK, org_id), user);
}
