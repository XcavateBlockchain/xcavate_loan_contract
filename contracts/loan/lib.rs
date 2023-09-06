#![cfg_attr(not(feature = "std"), no_std, no_main)]

use ink::primitives::AccountId;
use sp_runtime::MultiAddress;

#[derive(scale::Encode)]
enum RuntimeCall {
    #[codec(index = 8)]
    CommunityLoanPool(CommunityLoanPoolCall),
}

#[derive(scale::Encode)]
enum CommunityLoanPoolCall {
    #[codec(index = 3)]
    DeleteLoan { loan_id: u32 },

    #[codec(index = 4)]
    UpdateLoan { loan_id: u32, amount: u128 },
}

#[openbrush::contract]
pub mod loan {

    use crate::{CommunityLoanPoolCall, RuntimeCall};

    use ink::storage::Mapping;
    use xcavate_lending_protocol::traits::loan::*;

    use openbrush::traits::DefaultEnv;

    type Id = u32;

    #[ink(storage)]
    //#[derive(Default, Storage)]
    pub struct LoanContract {
        //Mapping of the loans
        loan_info: Mapping<Id, LoanInfo>,
        //Identifier for the loan
        last_loan_id: Id,
        //AccountId of the community-loan-pool
        pallet_id: AccountId,
    }

    impl Loan for LoanContract {
        #[ink(message, payable)]
        fn create_loan(
            &mut self,
            lender: AccountId,
            borrower: AccountId,
            collection_id: u32,
            item_id: u32,
            collateral_price: Balance,
            available_amount: Balance,
        ) -> Result<(), LoanError> {
            if available_amount > Self::env().transferred_value() {
                return Err(LoanError::NotEnoughFundsProvided);
            }
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
                return Err(LoanError::LoanIdTaken);
            }
            self.loan_info.insert(&loan_id, &loan_info);
            Ok(())
        }

        #[ink(message)]
        fn delete_loan(&mut self, loan_id: Id) -> Result<(), LoanError> {
            let loan_info = self.loan_info.get(&loan_id).unwrap();
            let remaining_available_amount = loan_info.available_amount;
            if loan_info.lender != Self::env().caller() {
                return Err(LoanError::NoPermission);
            }
            if loan_info.borrowed_amount != 0 {
                return Err(LoanError::OngoingLoan);
            }
            if remaining_available_amount > 0 {
                // for testing purpose on polkadot.js we add the zeros due to the decimals
                <Self as DefaultEnv>::env()
                    .transfer(self.pallet_id, remaining_available_amount * 1000000000000);
            }
            self.loan_info.remove(&loan_id);
            Self::env()
                .call_runtime(&RuntimeCall::CommunityLoanPool(
                    CommunityLoanPoolCall::DeleteLoan { loan_id },
                ))
                .map_err(Into::into)
        }

        #[ink(message, payable)]
        fn update_loan(
            &mut self,
            loan_id: Id,
            additional_available_amount: Balance,
        ) -> Result<(), LoanError> {
            let mut loan_info = self.loan_info.get(&loan_id).unwrap();
            if additional_available_amount > Self::env().transferred_value() {
                return Err(LoanError::NotEnoughFundsProvided);
            }
            if loan_info.lender != Self::env().caller() {
                return Err(LoanError::NoPermission);
            }
            loan_info.available_amount += additional_available_amount;
            loan_info.timestamp = <Self as DefaultEnv>::env().block_timestamp();
            self.loan_info.insert(&loan_id, &loan_info);
            Ok(())
        }

        #[ink(message, payable)]
        fn charge_apy(
            &mut self,
            loan_id: Id,
            amount: Balance,
        ) -> Result<(), LoanError> {
            let mut loan_info = self.loan_info.get(&loan_id).unwrap();
            loan_info.borrowed_amount += amount;
            loan_info.timestamp = <Self as DefaultEnv>::env().block_timestamp();
            self.loan_info.insert(&loan_id, &loan_info);
            Ok(())
        }

        #[ink(message, payable)]
        fn repay(&mut self, loan_id: Id, repay_amount: Balance) -> Result<(), LoanError> {
            let mut loan_info = self.loan_info.get(&loan_id).unwrap();
            if repay_amount == 0 {
                return Err(LoanError::RepayAmountMustBeHigherThanZero);
            }
            if repay_amount > Self::env().transferred_value() {
                return Err(LoanError::NotEnoughFundsProvided);
            }
            <Self as DefaultEnv>::env().transfer(self.pallet_id, Self::env().transferred_value());
            loan_info.borrowed_amount -= repay_amount;
            self.loan_info.insert(&loan_id, &loan_info);
            Self::env()
            .call_runtime(&RuntimeCall::CommunityLoanPool(
                CommunityLoanPoolCall::UpdateLoan { loan_id, amount: repay_amount },
            ))
            .map_err(Into::into)
        }

        #[ink(message)]
        fn withdraw_funds(&mut self, loan_id: Id, amount: u128) -> Result<(), LoanError> {
            let loan_info_option = self.loan_info.get(&loan_id);
            if loan_info_option.is_none() {
                return Err(LoanError::NonExistingLoanId);
            }
            let mut loan_info = loan_info_option.unwrap();
            if amount > Self::env().balance() {
                return Err(LoanError::InsufficientLoanBalance);
            }
            if amount > loan_info.available_amount {
                return Err(LoanError::InsufficientLoanBalance);
            }
            if loan_info.borrower != <Self as DefaultEnv>::env().caller() {
                return Err(LoanError::NotTheBorrower);
            }
            // for testing purpose on polkadot.js we add the zeros due to the decimals
            <Self as DefaultEnv>::env().transfer(loan_info.borrower, amount * 1000000000000);
            loan_info.borrowed_amount += amount;
            loan_info.available_amount -= amount;
            self.loan_info.insert(&loan_id, &loan_info);
            Ok(())
        }

        #[ink(message)]
        fn get_loan_info(&self, loan_id: Id) -> LoanInfo {
            let loan_info = self.loan_info.get(&loan_id).unwrap_or_else(|| {
                panic!("loan_id doesn't exist");
            });
            loan_info
        }
    }

    impl LoanContract {
        /// Constructor that initializes loan information for the contract
        #[ink(constructor, payable)]
        pub fn new(pallet_id: AccountId) -> Self {
            let loan_info = Mapping::default();
            let last_loan_id = 0;

            LoanContract {
                loan_info,
                last_loan_id,
                pallet_id,
            }
        }

        /// Internal function to return the id of a new loan and to increase it in the storage
        fn _get_next_loan_id_and_increase(&mut self) -> u32 {
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

        fn create_contract() -> LoanContract {
            let accounts = default_accounts();
            set_sender(accounts.alice);
            set_balance(contract_id(), 1000);
            LoanContract::new(accounts.frank)
        }

        fn contract_id() -> AccountId {
            ink::env::test::callee::<ink::env::DefaultEnvironment>()
        }
        fn default_accounts() -> ink::env::test::DefaultAccounts<ink::env::DefaultEnvironment> {
            ink::env::test::default_accounts::<ink::env::DefaultEnvironment>()
        }
        fn set_sender(sender: AccountId) {
            ink::env::test::set_caller::<ink::env::DefaultEnvironment>(sender);
        }
        fn set_balance(account_id: AccountId, balance: Balance) {
            ink::env::test::set_account_balance::<ink::env::DefaultEnvironment>(account_id, balance)
        }

        #[ink::test]
        fn create_loan_works() {
            let accounts = default_accounts();
            let mut loan = create_contract();
            let result = pay_with_call!(
                loan.create_loan(accounts.alice, accounts.bob, 0, 0, 2000, 1000),
                1000
            );
            assert_eq!(result, Ok(()));
        }

        #[ink::test]
        fn create_loan_fails_if_not_enough_funds_transferred() {
            let accounts = default_accounts();
            let mut loan = create_contract();
            let result = pay_with_call!(
                loan.create_loan(accounts.alice, accounts.bob, 0, 0, 2000, 1000),
                900
            );
            assert_eq!(result, Err(LoanError::NotEnoughFundsProvided));
        }

        #[ink::test]
        fn increase_loan_id_works() {
            let accounts = default_accounts();
            let mut loan = create_contract();
            let result = pay_with_call!(
                loan.create_loan(accounts.alice, accounts.bob, 0, 0, 2000, 1000),
                1000
            );
            assert_eq!(result, Ok(()));
            loan.get_loan_info(1);
            let result = pay_with_call!(
                loan.create_loan(accounts.alice, accounts.bob, 0, 0, 2000, 1000),
                1000
            );
            assert_eq!(result, Ok(()));
            loan.get_loan_info(2);
        }

        #[ink::test]
        fn withdraw_works() {
            let accounts = default_accounts();
            let mut loan = create_contract();
            let result = pay_with_call!(
                loan.create_loan(accounts.alice, accounts.bob, 0, 0, 2000, 1000),
                1000
            );
            assert_eq!(result, Ok(()));
            let contract_balance_before = ink::env::balance::<ink::env::DefaultEnvironment>();
            let bob_balance_before =
                get_account_balance::<ink::env::DefaultEnvironment>(accounts.bob);
            assert_eq!(Ok(1000), bob_balance_before);
            assert_eq!(2000, contract_balance_before);
            set_sender(accounts.bob);
            let result = loan.withdraw_funds(1, 500);
            assert_eq!(Ok(()), result);
            let contract_balance_after = ink::env::balance::<ink::env::DefaultEnvironment>();
            let bob_balance_after =
                get_account_balance::<ink::env::DefaultEnvironment>(accounts.bob);
            assert_eq!(Ok(1500), bob_balance_after);
            assert_eq!(1500, contract_balance_after);
            let loan_info = loan.get_loan_info(1);
            assert_eq!(500, loan_info.borrowed_amount);
            assert_eq!(500, loan_info.available_amount);
            let result = loan.withdraw_funds(1, 500);
            let loan_info = loan.get_loan_info(1);
            assert_eq!(1000, loan_info.borrowed_amount);
            assert_eq!(0, loan_info.available_amount);
        }

        #[ink::test]
        fn withdraw_fails_non_existing_loanid() {
            let accounts = default_accounts();
            let mut loan = create_contract();
            let result = pay_with_call!(
                loan.create_loan(accounts.alice, accounts.bob, 0, 0, 2000, 1000),
                1000
            );
            assert_eq!(result, Ok(()));
            let contract_balance_before = ink::env::balance::<ink::env::DefaultEnvironment>();
            let bob_balance_before =
                get_account_balance::<ink::env::DefaultEnvironment>(accounts.bob);
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
            pay_with_call!(
                loan.create_loan(accounts.bob, accounts.alice, 0, 0, 2000, 1000),
                1000
            );
            set_sender(accounts.bob);
            let result = loan.withdraw_funds(1, 500);
            assert_eq!(Err(LoanError::NotTheBorrower), result);
            set_sender(accounts.charlie);
            let result = loan.withdraw_funds(1, 500);
            assert_eq!(Err(LoanError::NotTheBorrower), result);
            let contract_balance_after = ink::env::balance::<ink::env::DefaultEnvironment>();
            assert_eq!(2000, contract_balance_after);
        }

        #[ink::test]
        fn withdraw_fails_insufficient_funds() {
            let accounts = default_accounts();
            let mut loan = create_contract();
            pay_with_call!(
                loan.create_loan(accounts.bob, accounts.alice, 0, 0, 2000, 1000),
                1000
            );
            set_sender(accounts.bob);
            let result = loan.withdraw_funds(1, 1500);
            assert_eq!(Err(LoanError::InsufficientLoanBalance), result);
        }

        #[ink::test]
        fn repay_works() {
            let accounts = default_accounts();
            let mut loan = create_contract();
            pay_with_call!(
                loan.create_loan(accounts.alice, accounts.bob, 0, 0, 2000, 1000),
                1000
            );
            let contract_balance_before = ink::env::balance::<ink::env::DefaultEnvironment>();
            let bob_balance_before =
                get_account_balance::<ink::env::DefaultEnvironment>(accounts.bob);
            let pallet_balance_before =
                get_account_balance::<ink::env::DefaultEnvironment>(accounts.frank);
            assert_eq!(Ok(1000), bob_balance_before);
            assert_eq!(2000, contract_balance_before);
            assert_eq!(Ok(0), pallet_balance_before);
            set_sender(accounts.bob);
            let result = loan.withdraw_funds(1, 500);
            assert_eq!(Ok(()), result);
            let repay_result = pay_with_call!(loan.repay(1, 250), 250);
            assert_eq!(Ok(()), repay_result);
            let contract_balance_after = ink::env::balance::<ink::env::DefaultEnvironment>();
            let bob_balance_after =
                get_account_balance::<ink::env::DefaultEnvironment>(accounts.bob);
            let pallet_balance_after =
                get_account_balance::<ink::env::DefaultEnvironment>(accounts.frank);
            assert_eq!(Ok(1250), bob_balance_after);
            assert_eq!(1500, contract_balance_after);
            assert_eq!(Ok(250), pallet_balance_after);
            let loan_info = loan.get_loan_info(1);
            assert_eq!(250, loan_info.borrowed_amount);
            assert_eq!(500, loan_info.available_amount);
        }

        #[ink::test]
        fn repay_fails_if_amount_is_zero() {
            let accounts = default_accounts();
            let mut loan = create_contract();
            let result = pay_with_call!(
                loan.create_loan(accounts.alice, accounts.bob, 0, 0, 2000, 1000),
                1000
            );
            let contract_balance_before = ink::env::balance::<ink::env::DefaultEnvironment>();
            let bob_balance_before =
                get_account_balance::<ink::env::DefaultEnvironment>(accounts.bob);
            assert_eq!(Ok(1000), bob_balance_before);
            assert_eq!(2000, contract_balance_before);
            set_sender(accounts.bob);
            let result = loan.withdraw_funds(1, 500);
            assert_eq!(Ok(()), result);
            let repay_result = pay_with_call!(loan.repay(1, 0), 0);
            assert_eq!(
                Err(LoanError::RepayAmountMustBeHigherThanZero),
                repay_result
            );
        }

        #[ink::test]
        fn repay_fails_not_enough_funds_provided() {
            let accounts = default_accounts();
            let mut loan = create_contract();
            let result = pay_with_call!(
                loan.create_loan(accounts.alice, accounts.bob, 0, 0, 2000, 1000),
                1000
            );
            let contract_balance_before = ink::env::balance::<ink::env::DefaultEnvironment>();
            let bob_balance_before =
                get_account_balance::<ink::env::DefaultEnvironment>(accounts.bob);
            assert_eq!(Ok(1000), bob_balance_before);
            assert_eq!(2000, contract_balance_before);
            set_sender(accounts.bob);
            let result = loan.withdraw_funds(1, 500);
            assert_eq!(Ok(()), result);
            let repay_result = pay_with_call!(loan.repay(1, 500), 400);
            assert_eq!(Err(LoanError::NotEnoughFundsProvided), repay_result);
        }

        #[ink::test]
        fn charge_apy_works() {
            let accounts = default_accounts();
            let mut loan = create_contract();
            pay_with_call!(
                loan.create_loan(accounts.alice, accounts.bob, 0, 0, 2000, 1000),
                1000
            );
            let loan_info_before = loan.get_loan_info(1);
            assert_eq!(0, loan_info_before.borrowed_amount);
            let result = loan.charge_apy(1, 100);
            assert_eq!(Ok(()), result);
            let loan_info_after = loan.get_loan_info(1);
            assert_eq!(100, loan_info_after.borrowed_amount);
        }
    }
}
