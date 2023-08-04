#![cfg_attr(not(feature = "std"), no_std, no_main)]

#[openbrush::contract]
pub mod loan {

    use xcavate_lending_protocol::traits::loan::*;

    use openbrush::{
        modifiers,
        storage::Mapping,
        traits::{
            DefaultEnv,
            Storage,
            String
        },
    };

    use ink::env::{
        call::{build_call, ExecutionInput, Selector},
        DefaultEnvironment,
    };

    #[ink(storage)]
    //#[derive(Default, Storage)]
    pub struct LoanContract {
        borrower: AccountId,
        collateral_nft: AccountId,
        collateral_price: Balance,
        /// amount of token that the lender took from the contract
        borrow_amount: Balance,
        liquidation_price: Balance,
        timestamp: Timestamp,
        liquidated: bool,
    }

    impl Loan for LoanContract {

        #[ink(message)]
        fn delete_loan(&mut self){
            <Self as DefaultEnv>::env().terminate_contract(<Self as DefaultEnv>::env().caller());
        }

        #[ink(message)]
        fn update_loan(&mut self, new_borrow_amount: Balance, new_timestamp: Timestamp) -> Result<(), LoanError>
        {
            
        }


        #[ink(message)]
        fn liquidate_loan(&mut self) -> Result<(), LoanError>
        {
            if 2 > 3 {
                return Err(LoanError::NotTheBorrower)
            }
            Ok(())
        }


        #[ink(message)]
        fn repay(&mut self, repay_amount: Balance) -> Result<(), LoanError>
        {
            if 2 > 3 {
                return Err(LoanError::NotTheBorrower)
            }
            Ok(())
        }
    

        #[ink(message)]
        fn withdraw_funds(&mut self, amount: Balance) -> Result<(), LoanError> {
            if amount <= <Self as DefaultEnv>::env().balance() {
                return Err(LoanError::InsufficientLoanBalance)
            }
            if self.borrower != <Self as DefaultEnv>::env().caller() {
                return Err(LoanError::NotTheBorrower)
            }
            <Self as DefaultEnv>::env().transfer(self.borrower, amount);
            self.borrow_amount += amount;
            Ok(())
        }


        
    }

    impl LoanContract {
        /// Constructor that initializes loan information for the contract
        #[ink(constructor, payable)]
        pub fn new(borrower: AccountId, collateral_nft: AccountId, collateral_price: Balance, borrow_amount: AccountId, liquidation_price: Balance) -> Self {
            let timestamp = <Self as DefaultEnv>::env().block_timestamp();
            let liquidated = Default::default();
            let borrow_amount = 0;
            
            LoanContract {
                borrower,
                collateral_nft,
                collateral_price,
                borrow_amount,
                liquidation_price,
                timestamp,
                liquidated
            }
        }
/*        #[ink(message)]
        pub fn get_loan_info(&self) -> LoanContract {
            self
        }  */
    }
}
