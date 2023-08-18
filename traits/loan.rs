use openbrush::{
    traits::{
        AccountId,
        Balance,
        Timestamp,
    },
};

type Id = u128;

#[cfg(feature = "std")]
use ink::storage::traits::StorageLayout;

#[derive(Debug, Clone, scale::Encode, scale::Decode)]
#[cfg_attr(feature = "std", derive(StorageLayout, scale_info::TypeInfo))]
pub struct LoanInfo {
    pub lender: AccountId,
    pub borrower: AccountId,
    pub collection_id: u32,
    pub item_id: u32,
    pub collateral_price: Balance,
    /// amount of token that the lender can borrow from the contract
    pub available_amount: Balance,
    /// amount of token that the lender took from the contract
    pub borrowed_amount: Balance,
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
    #[ink(message)]
    fn create_loan(&mut self, borrower: AccountId, collection_id: u32, item_id: u32, collateral_price: Balance, available_amount: Balance) -> Result<(), LoanError>;

    // This function will delete the loan
    #[ink(message)]
    fn delete_loan(&mut self, loan_id: Id) -> Result<(), LoanError>;

    // This function will update the loan
    #[ink(message)]
    fn update_loan(&mut self, loan_id: Id, new_borrow_amount: Balance) -> Result<(), LoanError>;

    // This function is for the lender to repay the loan
    #[ink(message, payable)]
    fn repay(&mut self, loan_id: Id) -> Result<(), LoanError>;
    
    // This function lets the lender withdraw funds from the loan
    #[ink(message)]
    fn withdraw_funds(&mut self,loan_id: Id, amount: Balance) -> Result<(), LoanError>;

    #[ink(message)]
    fn get_loan_info(&self, loan_id: Id) -> LoanInfo;
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

    NoPermission,

    UnexpectedLoanId,

    LoanIdTaken,

    OngoingLoan,

    NonExistingLoanId,

    NotEnoughFundsProvided,
    
}