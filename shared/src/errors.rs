use soroban_sdk::contracterror;

#[contracterror]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ContractError {
    Unauthorized = 1,
    AlreadyExists = 2,
    NotFound = 3,
    InvalidRole = 4,
    InvalidOrganization = 5,
    InvalidProject = 6,
    InvalidDeployment = 7,
    StorageFailure = 8,
    ValidationFailure = 9,
}
