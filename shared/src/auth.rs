use soroban_sdk::Address;

use crate::errors::ContractError;

pub fn authorize(owner: &Address) -> Result<(), ContractError> {
    owner.require_auth();
    Ok(())
}

pub fn check_active(status: u32) -> Result<(), ContractError> {
    if status == crate::types::STATUS_ARCHIVED {
        return Err(ContractError::InvalidOrganization);
    }
    Ok(())
}
