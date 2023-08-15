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

#[openbrush::wrapper]
pub type LoanRef = dyn Loan;

#[openbrush::trait_definition]
pub trait Loan {

    // This function will delete the loan
    #[ink(message)]
    fn delete_loan(&mut self, collection: u16, item: u16);

    // This function will update the loan
    #[ink(message)]
    fn update_loan(&mut self, new_borrow_amount: Balance, new_timestamp: Timestamp) -> Result<(), LoanError>;

    // This function will liquidate the loan
    #[ink(message)]
    fn liquidate_loan(&mut self) -> Result<(), LoanError>;

    // This function is for the lender to repay the loan
    #[ink(message, payable)]
    fn repay(&mut self) -> Result<(), LoanError>;
    
    // This function lets the lender withdraw funds from the loan
    #[ink(message)]
    fn withdraw_funds(&mut self, amount: Balance) -> Result<(), LoanError>;
}

#[derive(Debug, PartialEq, Eq, scale::Encode, scale::Decode)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
pub enum LoanError {
    /// This error will be thrown if there is no loan for the loan_id,
    InvalidLoanId,
    /// This error will be thrown if the lender has not enough funds to repay
    InsufficientBalanceToRepay,
    /// This error will be thrown if the contract doesn't have enough funds to withdraw for the borrower
    InsufficientLoanBalance,
    /// This error will be thrown if the caller is not the borrower
    NotTheBorrower,
    ///
    RepayAmountMustBeHigherThanZero,
    /// This error will be thrown if the lender has not enough available runds to borrow
    InsufficientAllowance,
    /// This error will be thrown if the loan got already liquidated
    AlreadyLiquidated,
    
}