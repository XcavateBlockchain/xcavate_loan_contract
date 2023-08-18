#![cfg_attr(not(feature = "std"), no_std, no_main)]

#[openbrush::contract]
pub mod loan {

    use uniques_extension::Origin;
    use uniques_extension::*;

    use xcavate_lending_protocol::traits::loan::*;
    use ink::storage::Mapping;

    use openbrush::{
        //storage::Mapping,
        traits::{
            DefaultEnv,
        },
    };

    type Id = u128;

    #[ink(storage)]
    //#[derive(Default, Storage)]
    pub struct LoanContract {
        loan_info: Mapping<Id, LoanInfo>,
        last_loan_id: Id,
    }

    impl Loan for LoanContract {

        #[ink(message)]
        fn create_loan(&mut self, borrower: AccountId, collection_id: u32, item_id: u32, collateral_price: Balance, available_amount: Balance) -> Result<(), LoanError> {

            if available_amount > Self::env().transferred_value() {
                return Err(LoanError::NotEnoughFundsProvided)
            }
            let lender = <Self as DefaultEnv>::env().caller();
            let timestamp = <Self as DefaultEnv>::env().block_timestamp();
            let borrowed_amount = 0;
            let loan_info = LoanInfo {
                lender,
                borrower, 
                collection_id, 
                item_id, 
                collateral_price,
                available_amount, 
                borrowed_amount,
                timestamp,
            };

            let loan_id = self._get_next_loan_id_and_increase();
            if self.loan_info.get(&loan_id).is_some() {
                return Err(LoanError::LoanIdTaken)
            }
            self.loan_info.insert(&loan_id, &loan_info);
            Ok(())
        }

        #[ink(message)]
        fn delete_loan(&mut self, loan_id: Id) -> Result<(), LoanError> {
            let loan_info = self.loan_info.get(&loan_id).unwrap();
            if loan_info.lender != Self::env().caller() {
                return Err(LoanError::NoPermission)
            }
            if loan_info.borrowed_amount == 0 {
                return Err(LoanError::OngoingLoan)
            }
            let collection = loan_info.collection_id;       
            let item = loan_info.item_id;
            let contract = Self::env().account_id();
            UniquesExtension::burn(
                Origin::Address,
                collection,
                item,
                Some(contract),
            );
            self.loan_info.remove(&loan_id);
            Ok(())
        }

        #[ink(message)]
        fn update_loan(&mut self, loan_id: Id, additional_available_amount: Balance) -> Result<(), LoanError>
        {
            let mut loan_info = self.loan_info.get(&loan_id).unwrap();
            if additional_available_amount > Self::env().transferred_value() {
                return Err(LoanError::NotEnoughFundsProvided)
            } 
            if loan_info.lender != Self::env().caller() {
                return Err(LoanError::NoPermission)
            }
            loan_info.available_amount += additional_available_amount;
            loan_info.timestamp = <Self as DefaultEnv>::env().block_timestamp();
            self.loan_info.insert(&loan_id, &loan_info);
            Ok(())
        }


        #[ink(message, payable)]
        fn repay(&mut self, loan_id: Id) -> Result<(), LoanError>
        {
            let mut loan_info = self.loan_info.get(&loan_id).unwrap();
            let repay_amount = <Self as DefaultEnv>::env().transferred_value();
            if repay_amount == 0 {
                return Err(LoanError::RepayAmountMustBeHigherThanZero)
            }
            loan_info.borrowed_amount -= repay_amount;
            self.loan_info.insert(&loan_id, &loan_info);
            Ok(())
        }
    

        #[ink(message)]
        fn withdraw_funds(&mut self,loan_id: Id, amount: Balance) -> Result<(), LoanError> {

            let loan_info_option = self.loan_info.get(&loan_id);
            if loan_info_option.is_none() {
                return Err(LoanError::NonExistingLoanId)
            }
            let mut loan_info = loan_info_option.unwrap();
            if amount >= Self::env().balance() {
                return Err(LoanError::InsufficientLoanBalance)
            }
            if amount >= loan_info.available_amount {
                return Err(LoanError::InsufficientLoanBalance)
            }
            if loan_info.borrower != <Self as DefaultEnv>::env().caller() {
                return Err(LoanError::NotTheBorrower)
            }
            <Self as DefaultEnv>::env().transfer(loan_info.borrower, amount);
            loan_info.borrowed_amount += amount;
            loan_info.available_amount -= amount;
            self.loan_info.insert(&loan_id, &loan_info);
            Ok(())
        }

        #[ink(message)]
        fn get_loan_info(&self, loan_id: Id) -> LoanInfo{
            let mut loan_info = self.loan_info.get(&loan_id).unwrap_or_else(|| {
                panic!("loan_id doesn't exist");
            });
            loan_info
        }
    }

    impl LoanContract {
        /// Constructor that initializes loan information for the contract
        #[ink(constructor, payable)]
        pub fn new() -> Self {
            let loan_info = Mapping::default();
            let last_loan_id = 0;

            LoanContract{
                loan_info,
                last_loan_id,
            }
        }

        /// Internal function to return the id of a new loan and to increase it in the storage
        fn _get_next_loan_id_and_increase(&mut self) -> u128 {
            let mut loan_id = self.last_loan_id;
            loan_id += 1;
            self.last_loan_id = loan_id;
            loan_id
        }
    }


    #[cfg(test)]
    mod tests {
        use super::*;
        use ink::env::pay_with_call;
        use ink::env::test::*;

        fn create_contract() -> LoanContract{
            let accounts = default_accounts();
            set_sender(accounts.alice);
            set_balance(contract_id(), 1000);
            LoanContract::new()
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
        fn create_loan_works() {
            let accounts = default_accounts();
            let mut loan = create_contract();
            let result = pay_with_call!(loan.create_loan(accounts.bob, 0, 0, 2000, 1000), 1000);
            assert_eq!(result, Ok(()));
        }

        #[ink::test]
        fn create_loan_fails_if_not_enough_funds_transferred() {
            let accounts = default_accounts();
            let mut loan = create_contract();
            let result = pay_with_call!(loan.create_loan(accounts.bob, 0, 0, 2000, 1000), 900);
            assert_eq!(result, Err(LoanError::NotEnoughFundsProvided));
        }

        #[ink::test]
        fn increase_loan_id_works() {
            let accounts = default_accounts();
            let mut loan = create_contract();
            let result = pay_with_call!(loan.create_loan(accounts.bob, 0, 0, 2000, 1000), 1000);
            assert_eq!(result, Ok(()));
            loan.get_loan_info(1);
            let result = pay_with_call!(loan.create_loan(accounts.bob, 0, 0, 2000, 1000), 1000);
            assert_eq!(result, Ok(()));
            loan.get_loan_info(2);
        }

        #[ink::test]
        fn withdraw_works() {
            let accounts = default_accounts();
            let mut loan = create_contract();
            let result = pay_with_call!(loan.create_loan(accounts.bob, 0, 0, 2000, 1000), 1000);
            assert_eq!(result, Ok(()));
            let contract_balance_before = ink::env::balance::<ink::env::DefaultEnvironment>();
            let bob_balance_before = get_account_balance::<ink::env::DefaultEnvironment>(accounts.bob);
            assert_eq!(Ok(1000), bob_balance_before);
            assert_eq!(2000, contract_balance_before);
            set_sender(accounts.bob);
            let result = loan.withdraw_funds(1, 500);
            assert_eq!(Ok(()), result);
            let contract_balance_after = ink::env::balance::<ink::env::DefaultEnvironment>();
            let bob_balance_after = get_account_balance::<ink::env::DefaultEnvironment>(accounts.bob);
            assert_eq!(Ok(1500), bob_balance_after);
            assert_eq!(1500, contract_balance_after);
            let loan_info = loan.get_loan_info(1);
            assert_eq!(500, loan_info.borrowed_amount);
            assert_eq!(500, loan_info.available_amount);
        }

        #[ink::test]
        fn withdraw_fails_non_existing_loanid() {
            let accounts = default_accounts();
            let mut loan = create_contract();
            let result = pay_with_call!(loan.create_loan(accounts.bob, 0, 0, 2000, 1000), 1000);
            assert_eq!(result, Ok(()));
            let contract_balance_before = ink::env::balance::<ink::env::DefaultEnvironment>();
            let bob_balance_before = get_account_balance::<ink::env::DefaultEnvironment>(accounts.bob);
            assert_eq!(Ok(1000), bob_balance_before);
            assert_eq!(2000, contract_balance_before);
            set_sender(accounts.bob);
            let result = loan.withdraw_funds(3, 500);
            assert_eq!(Err(LoanError::NonExistingLoanId), result);
        }

        #[ink::test]
        fn withdraw_fails_if_someone_but_the_borrower_calls() {
            let accounts = default_accounts();
            let mut loan = create_contract();
            let result = pay_with_call!(loan.create_loan(accounts.alice, 0, 0, 2000, 1000), 1000);
            set_sender(accounts.bob);
            let result = loan.withdraw_funds(1,500);
            assert_eq!(Err(LoanError::NotTheBorrower), result);
            set_sender(accounts.charlie);
            let result = loan.withdraw_funds(1,500);
            assert_eq!(Err(LoanError::NotTheBorrower), result);
            let contract_balance_after = ink::env::balance::<ink::env::DefaultEnvironment>();
            assert_eq!(2000, contract_balance_after);
        }

        #[ink::test]
        fn withdraw_fails_insufficient_funds() {
            let accounts = default_accounts();
            let mut loan = create_contract();
            let result = pay_with_call!(loan.create_loan(accounts.alice, 0, 0, 2000, 1000), 1000);
            set_sender(accounts.bob);
            let result = loan.withdraw_funds(1, 1500);
            assert_eq!(Err(LoanError::InsufficientLoanBalance), result);
        }

        #[ink::test]
        fn repay_works() {
            let accounts = default_accounts();
            let mut loan = create_contract();
            let result = pay_with_call!(loan.create_loan(accounts.bob, 0, 0, 2000, 1000), 1000);
            let contract_balance_before = ink::env::balance::<ink::env::DefaultEnvironment>();
            let bob_balance_before = get_account_balance::<ink::env::DefaultEnvironment>(accounts.bob);
            assert_eq!(Ok(1000), bob_balance_before);
            assert_eq!(2000, contract_balance_before);
            set_sender(accounts.bob);
            let result = loan.withdraw_funds(1, 500);
            assert_eq!(Ok(()), result);
            let repay_result = pay_with_call!(loan.repay(1), 250);
            assert_eq!(Ok(()), repay_result);
            let contract_balance_after = ink::env::balance::<ink::env::DefaultEnvironment>();
            let bob_balance_after = get_account_balance::<ink::env::DefaultEnvironment>(accounts.bob);
            assert_eq!(Ok(1250), bob_balance_after);
            assert_eq!(1750, contract_balance_after);
            let loan_info = loan.get_loan_info(1);
            assert_eq!(250, loan_info.borrowed_amount);
            assert_eq!(500, loan_info.available_amount);
        }

        #[ink::test]
        fn repay_fails_if_amount_is_zero() {
            let accounts = default_accounts();
            let mut loan = create_contract();
            let result = pay_with_call!(loan.create_loan(accounts.bob, 0, 0, 2000, 1000), 1000);
            let contract_balance_before = ink::env::balance::<ink::env::DefaultEnvironment>();
            let bob_balance_before = get_account_balance::<ink::env::DefaultEnvironment>(accounts.bob);
            assert_eq!(Ok(1000), bob_balance_before);
            assert_eq!(2000, contract_balance_before);
            set_sender(accounts.bob);
            let result = loan.withdraw_funds(1, 500);
            assert_eq!(Ok(()), result);
            let repay_result = pay_with_call!(loan.repay(1), 0);
            assert_eq!(Err(LoanError::RepayAmountMustBeHigherThanZero), repay_result);
        }
    }
}
