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
        /// amount of token that the lender can borrow from the contract
        available_amount: Balance,
        /// amount of token that the lender took from the contract
        borrowed_amount: Balance,
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
        fn update_loan(&mut self, new_available_amount: Balance, new_timestamp: Timestamp) -> Result<(), LoanError>
        {
            if self.liquidated == true {
                return Err(LoanError::AlreadyLiquidated)
            }
            self.available_amount = new_available_amount;
            self.timestamp = <Self as DefaultEnv>::env().block_timestamp();
            Ok(())
        }


        #[ink(message)]
        fn liquidate_loan(&mut self) -> Result<(), LoanError>
        {
            // Whats happens to the nft?
            if self.liquidated == true {
                return Err(LoanError::AlreadyLiquidated)
            }
            self.liquidated = true;
            Ok(())
        }


        #[ink(message, payable)]
        fn repay(&mut self) -> Result<(), LoanError>
        {
            let repay_amount = <Self as DefaultEnv>::env().transferred_value();
            if repay_amount == 0 {
                return Err(LoanError::RepayAmountMustBeHigherThanZero)
            }
            self.borrowed_amount -= repay_amount;
            Ok(())
        }
    

        #[ink(message)]
        fn withdraw_funds(&mut self, amount: Balance) -> Result<(), LoanError> {
            if amount >= <Self as DefaultEnv>::env().balance() {
                return Err(LoanError::InsufficientLoanBalance)
            }
            if amount >= self.available_amount {
                return Err(LoanError::InsufficientLoanBalance)
            }
            if self.borrower != <Self as DefaultEnv>::env().caller() {
                return Err(LoanError::NotTheBorrower)
            }
            <Self as DefaultEnv>::env().transfer(self.borrower, amount);
            self.borrowed_amount += amount;
            self.available_amount -= amount;
            Ok(())
        }
    }

    impl LoanContract {
        /// Constructor that initializes loan information for the contract
        #[ink(constructor, payable)]
        pub fn new(borrower: AccountId, collateral_nft: AccountId, collateral_price: Balance, available_amount: Balance, liquidation_price: Balance) -> Self {
            let timestamp = <Self as DefaultEnv>::env().block_timestamp();
            let liquidated = Default::default();
            let borrowed_amount = 0;
            
            LoanContract {
                borrower,
                collateral_nft,
                collateral_price,
                available_amount,
                borrowed_amount,
                liquidation_price,
                timestamp,
                liquidated
            }
        }

/*         #[ink(message)]
        pub fn get_loan_info(&self) -> &LoanContract {
            self
        }   */
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        use ink::env::pay_with_call;
        use ink::env::test::*;

        fn create_contract(borrower: AccountId, collateral_nft: AccountId, collateral_price: Balance, available_amount: Balance, liquidation_price: Balance) -> LoanContract{
            let accounts = default_accounts();
            set_sender(accounts.alice);
            set_balance(contract_id(), 1000);
            LoanContract::new(borrower, collateral_nft, collateral_price, available_amount,liquidation_price)
        }
        fn contract_id() -> AccountId {
            ink::env::test::callee::<ink::env::DefaultEnvironment>()
        }
        fn default_accounts(
        ) -> ink::env::test::DefaultAccounts<ink::env::DefaultEnvironment> {
            ink::env::test::default_accounts::<ink::env::DefaultEnvironment>()
        }
        fn set_sender(sender: AccountId) {
            ink::env::test::set_caller::<ink::env::DefaultEnvironment>(sender);
        }
        fn set_balance(account_id: AccountId, balance: Balance) {
            ink::env::test::set_account_balance::<ink::env::DefaultEnvironment>(
                account_id, balance,
            )
        }

        #[ink::test]
        fn withdraw_works() {
            let accounts = default_accounts();
            let mut loan = create_contract(accounts.bob, [0x09; 32].into(), 2000, 1000, 1500);
            let contract_balance_before = ink::env::balance::<ink::env::DefaultEnvironment>();
            let bob_balance_before = get_account_balance::<ink::env::DefaultEnvironment>(accounts.bob);
            assert_eq!(Ok(1000), bob_balance_before);
            assert_eq!(1000, contract_balance_before);
            set_sender(accounts.bob);
            let result = loan.withdraw_funds(500);
            assert_eq!(Ok(()), result);
            let contract_balance_after = ink::env::balance::<ink::env::DefaultEnvironment>();
            let bob_balance_after = get_account_balance::<ink::env::DefaultEnvironment>(accounts.bob);
            assert_eq!(Ok(1500), bob_balance_after);
            assert_eq!(500, contract_balance_after);
            assert_eq!(500, loan.borrowed_amount);
            assert_eq!(500, loan.available_amount);
        }

        #[ink::test]
        fn withdraw_fails_if_someone_but_the_borrower_calls() {
            let accounts = default_accounts();
            let mut loan = create_contract(accounts.bob, [0x09; 32].into(), 2000, 1000, 1500);
            set_sender(accounts.alice);
            let result = loan.withdraw_funds(500);
            assert_eq!(Err(LoanError::NotTheBorrower), result);
            set_sender(accounts.charlie);
            let result = loan.withdraw_funds(500);
            assert_eq!(Err(LoanError::NotTheBorrower), result);
            let contract_balance_after = ink::env::balance::<ink::env::DefaultEnvironment>();
            assert_eq!(1000, contract_balance_after);
        }

        #[ink::test]
        fn withdraw_fails_insufficient_funds() {
            let accounts = default_accounts();
            let mut loan = create_contract(accounts.bob, [0x09; 32].into(), 2000, 1000, 1500);
            set_sender(accounts.bob);
            let result = loan.withdraw_funds(1500);
            assert_eq!(Err(LoanError::InsufficientLoanBalance), result);
        }

        #[ink::test]
        fn repay_works() {
            let accounts = default_accounts();
            let mut loan = create_contract(accounts.bob, [0x09; 32].into(), 2000, 1000, 1500);
            let contract_balance_before = ink::env::balance::<ink::env::DefaultEnvironment>();
            let bob_balance_before = get_account_balance::<ink::env::DefaultEnvironment>(accounts.bob);
            assert_eq!(Ok(1000), bob_balance_before);
            assert_eq!(1000, contract_balance_before);
            set_sender(accounts.bob);
            let result = loan.withdraw_funds(500);
            assert_eq!(Ok(()), result);
            let repay_result = pay_with_call!(loan.repay(), 250);
            assert_eq!(Ok(()), repay_result);
            let contract_balance_after = ink::env::balance::<ink::env::DefaultEnvironment>();
            let bob_balance_after = get_account_balance::<ink::env::DefaultEnvironment>(accounts.bob);
            assert_eq!(Ok(1250), bob_balance_after);
            assert_eq!(750, contract_balance_after);
            assert_eq!(250, loan.borrowed_amount);
            assert_eq!(500, loan.available_amount);
        }

        #[ink::test]
        fn repay_fails_if_amount_is_zero() {
            let accounts = default_accounts();
            let mut loan = create_contract(accounts.bob, [0x09; 32].into(), 2000, 1000, 1500);
            let contract_balance_before = ink::env::balance::<ink::env::DefaultEnvironment>();
            let bob_balance_before = get_account_balance::<ink::env::DefaultEnvironment>(accounts.bob);
            assert_eq!(Ok(1000), bob_balance_before);
            assert_eq!(1000, contract_balance_before);
            set_sender(accounts.bob);
            let result = loan.withdraw_funds(500);
            assert_eq!(Ok(()), result);
            let repay_result = pay_with_call!(loan.repay(), 0);
            assert_eq!(Err(LoanError::RepayAmountMustBeHigherThanZero), repay_result);
        }
    }
}
