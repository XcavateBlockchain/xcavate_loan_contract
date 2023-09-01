use openbrush::{
    traits::{
        AccountId,
        Balance,
        Timestamp,
    },
};

type Id = u32;

#[cfg(feature = "std")]
use ink::storage::traits::StorageLayout;

#[derive(Debug, Clone, scale::Encode, scale::Decode)]
#[cfg_attr(feature = "std", derive(StorageLayout, scale_info::TypeInfo))]
pub struct LoanInfo {
    /// Admin who calls the create_loan function
    pub lender: AccountId,
    /// AccountId of the borrower
    pub borrower: AccountId,
    /// Collection ID of the nft
    pub collection_id: u32,
    /// Item ID of the nft
    pub item_id: u32,
    /// Collateral price of the nft
    pub collateral_price: Balance,
    /// Available amount of funds for the borrower to borrow
    pub available_amount: Balance,
    /// Actual amount of funds that has been borrowed
    pub borrowed_amount: Balance,
    /// Timestamp when the loan has been created or from the latest update
    pub timestamp: Timestamp,
}

impl Default for LoanInfo {
    fn default() -> Self {
        Self {            
            lender: [0u8; 32].into(),           
            borrower: [0u8; 32].into(),           
            collection_id: Default::default(),          
            item_id: Default::default(),          
            collateral_price: Balance::default(),        
            available_amount: Balance::default(),           
            borrowed_amount: Balance::default(),          
            timestamp: Timestamp::default(),
        }
    }
}

#[openbrush::wrapper]
pub type LoanRef = dyn Loan;

#[openbrush::trait_definition]
pub trait Loan {

    // This function will create a new loan
    #[ink(message, payable)]
    fn create_loan(&mut self, lender: AccountId, borrower: AccountId, collection_id: u32, item_id: u32, collateral_price: Balance, available_amount: Balance) -> Result<(), LoanError>;

    // This function will delete the loan and burns the nft
    #[ink(message)]
    fn delete_loan(&mut self, loan_id: Id) -> Result<(), LoanError>;

    // This function will update the loan
    #[ink(message, payable)]
    fn update_loan(&mut self, loan_id: Id, new_borrow_amount: Balance) -> Result<(), LoanError>;

    // This function is for the lender to repay the loan
    #[ink(message, payable)]
    fn repay(&mut self, loan_id: Id, repay_amount: Balance) -> Result<(), LoanError>;
    
    // This function lets the lender withdraw funds from the loan
    #[ink(message)]
    fn withdraw_funds(&mut self,loan_id: Id, amount: Balance) -> Result<(), LoanError>;

    #[ink(message)]
    fn get_loan_info(&self, loan_id: Id) -> LoanInfo;
}

#[derive(Debug, PartialEq, Eq, scale::Encode, scale::Decode)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
pub enum LoanError {
    /// This error will be thrown if the lender has not enough funds to repay
    InsufficientBalanceToRepay,
    /// This error will be thrown if the contract doesn't have enough funds to withdraw for the borrower
    InsufficientLoanBalance,
    /// This error will be thrown if the caller is not the borrower
    NotTheBorrower,
    /// This error will be thrown if the repay amount is zero
    RepayAmountMustBeHigherThanZero,
    /// This error will be thrown if the lender has not enough available funds to borrow
    InsufficientAllowance,
    /// This error will be thrown if if the caller doen't have enough permission for the function
    NoPermission,
    /// This error will be thrown if the loan ID already exist
    LoanIdTaken,
    /// This error will be thrown if the loan hasn't been fully paid back yet
    OngoingLoan,
    /// This error will be thrown if there is no loan for the loan_id,
    NonExistingLoanId,
    /// The contract didn't received enough funds for the loan
    NotEnoughFundsProvided,
    /// Error if the runtime call failed
    CallRuntimeFailed,
    
}

use ink::env::Error as EnvError;

impl From<EnvError> for LoanError {
    fn from(e: EnvError) -> Self {
        match e {
            EnvError::CallRuntimeFailed => LoanError::CallRuntimeFailed,
            _ => panic!("Unexpected error from `pallet-contracts`."),
        }
    }
}


