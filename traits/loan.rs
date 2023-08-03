use openbrush::{
    contracts::traits::{
        ownable::*,
    },
    traits::{
        AccountId,
        Balance,
        Timestamp,
    },
};

#[cfg(feature = "std")]
use ink::storage::traits::StorageLayout;

#[derive(Debug, Clone, scale::Encode, scale::Decode)]
#[cfg_attr(feature = "std", derive(StorageLayout, scale_info::TypeInfo))]

pub struct LoanInfo {
    borrower: AccountId,
    collateral_nft: AccountId,
    colleteral_price: Balance,
    borrow_amount: Balance,
    liquidation_price: Balance,
    timestamp: Timestamp,
    liquidated: bool,
}

#[openbrush::trait_definition]
impl Loan: ownable {

    // This function will be called by the lending contract to create a new loan
    fn create_loan(loan_info: LoanInfo) -> Result<(), LoanError>;

    // This function will delete the loan
    fn delete_loan(loan_id: AccountId) -> Result<(), LoanError>;

    // This function will update the loan
    fn update_loan(new_borrow_amount: Balance, new_timestamp: Timestamp) -> Result<(), LoanError>;

    // This function will liquidate the loan
    fn liquidate_loan(loan_id: AccountId) -> Result<(), LoanError>;

    // This function get the loan info for the caller
    fn get_loan_info(loan_id: AccountId) -> Result<(), LoanError>;

    // This function is for the lender to repay the loan

    #[ink(message)]
    fn repay(&mut self, loan_id: AccountId, repay_amount: Balance) -> Result<(), LendingError>;
    
    // This function lets the lender repay the loan
    
    #[ink(message)]
    fn withdraw_funds(&mut self, loan_id: AccountId, amount: Balamce) -> Result<(), LendingError>;
}

#[derive(Debug, PartialEq, Eq, scale::Encode, scale::Decode)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
pub enum LoanError {
    /// This error will be thrown if there is no loan for the loan_id,
    InvalidLoanId,
    /// This error will be thrown if the lender has not enough funds to repay
    InsufficientBalanceToRepay,
}