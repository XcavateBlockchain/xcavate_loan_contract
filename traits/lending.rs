use openbrush::{
    contracts::traits::{
        access_control::*,
        pausable::*,
    },
    traits::{
        AccountId,
        Balance,
    },
};

/// Combination of all traits of the contract to simplify calls to the contract
#[openbrush::wrapper]
pub type LendingContractRef = dyn Lending + AccessControl + Pausable;

#[openbrush::wrapper]
pub type LendingRef = dyn Lending;

#[openbrush::trait_definition]
pub trait Lending {

    // In this function the lender sends application for the loan

    #[ink(message)]
    fn apply_loan(&mut self, colleteral_nft: AccountId, amount: Balance) -> Result<(), LendingError>;

    // This function is for the lender to repay the loan

    #[ink(message)]
    fn repay(&mut self, loan_id: AccountId, repay_amount: Balance) -> Result<(), LendingError>;

    // This function lets the lender repay the loan

    #[ink(message)]
    fn withdraw_assets(&mut self, loan_id: AccountId, amount: Balamce) -> Result<(), LendingError>;

    // This function will liquidate the loan

    #[ink(message)]
    fn liquidate_loan(&mut self, loan_id: AccountId) -> Result<(), LendingError>;

    // This function is approving the loan for the lender

    #[ink(message)]
    fn approve_loan(&mut self, loan_id: AccoundId, loan_owner: AccountId, nft_address: AccountId) -> Result<(), LendingError>;

    // This function is for the lender to get the loan

    #[ink(message)]
    fn borrow_assets(&mut self, amount: Balance) -> <Result(), LendingError>;

}

#[derive(Debug, PartialEq, Eq, scale::Encode, scale::Decode)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
pub enum LendingError {
    /// This error will be thrown is the lender has not enough colleteral
    InsufficientColleteral,

    /// This error will be thrown if the lender has not enough funds to repay
    InsufficientBalanceToRepay,

    /// This error will be thrown if the lender has no permission to apply for a loan
    NoApplyingPermission,

    /// This error will be thrown if the loan wasn't approved
    NoPermissionToLend,

}