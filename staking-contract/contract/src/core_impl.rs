use near_sdk::{Gas, env, PromiseOrValue, ext_contract, PromiseResult};

use crate::*;

pub const DEPOSIT_ONE_YOCTOR: Balance = 1;
pub const NO_DEPOSIT: Balance = 0;
pub const FT_TRANSFER_GAS: Gas = Gas(10_000_000_000_000);
pub const WITHDRAW_CALLBACK_GAS: Gas = Gas(10_000_000_000_000);
pub const HARVEST_CALLBACK_GAS: Gas = Gas(10_000_000_000_00);

pub trait FungibleTokenReceiver {
    fn ft_on_transfer(&mut self, sender_id: AccountId, amount: U128, msg: String) -> PromiseOrValue<U128>;
}

#[ext_contract(ext_ft_contract)]
pub trait FungibleToken {
    fn ft_transfer(&mut self, receiver_id: AccountId, amount: U128, memo: Option<String>);
}

#[ext_contract(ext_self)]
pub trait ExtStakingContract {
    fn ft_transfer_callback(&mut self, amount: U128, account_id: AccountId);
    fn ft_withdraw_callback(&mut self, account_id: AccountId, old_account: Account);
}

impl FungibleTokenReceiver for StakingContract {
    fn ft_on_transfer(&mut self, sender_id: AccountId, amount:U128, msg: String) -> PromiseOrValue<U128> {
        self.internal_deposit_and_stake(sender_id, amount.0);

        PromiseOrValue::Value(U128(0))
    }
}

#[near_bindgen]
impl StakingContract {

    #[payable]
    pub fn unstake(&mut self, amount: U128) {
        assert_one_yocto();
        let account_id: AccountId = env::predecessor_account_id();

        self.internal_unstake(account_id, amount.0);
    }

    #[payable]
    pub fn withdraw(&mut self) -> Promise {
        assert_one_yocto();
        let account_id: AccountId = env::predecessor_account_id();
        let old_account: Account = self.internal_withdraw(account_id.clone());

        // handle transfer withdraw
        ext_ft_contract::ext(self.ft_contract_id.clone())
            .with_attached_deposit(DEPOSIT_ONE_YOCTOR)
            .with_static_gas(FT_TRANSFER_GAS)
            .ft_transfer(
                account_id.clone(), 
                U128(old_account.unstake_balance), 
                Some(String::from("Staking contract withdraw"))
            ).then(
                ext_self::ext(env::current_account_id())
                .with_attached_deposit(NO_DEPOSIT)
                .with_static_gas(WITHDRAW_CALLBACK_GAS)
                .ft_withdraw_callback(
                    account_id.clone(), 
                    old_account
                )
            )
    }

    #[payable]
    pub fn harvest(&mut self) -> Promise {
        assert_one_yocto();
        let account_id: AccountId = env::predecessor_account_id();
        let upgradable_account = self.accounts.get(&account_id).unwrap();
        let account = Account::from(upgradable_account);

        let new_reward: Balance = self.internal_calculate_account_reward(&account);
        let current_reward: Balance = account.pre_reward + new_reward;

        assert!(current_reward > 0, "ERR_REWARD_EQUAL_ZERO");

        // Cross contract call
        ext_ft_contract::ext(self.ft_contract_id.clone())
            .with_attached_deposit(DEPOSIT_ONE_YOCTOR)
            .with_static_gas(FT_TRANSFER_GAS)
            .ft_transfer(
                account_id.clone(), 
                U128(current_reward), 
                Some("Staking contract harvest".to_string())
            ).then(
                ext_self::ext(env::current_account_id())
                .with_attached_deposit(NO_DEPOSIT)
                .with_static_gas(HARVEST_CALLBACK_GAS)
                .ft_transfer_callback(U128(current_reward), account_id.clone())
            )
    }

    #[private]
    pub fn ft_transfer_callback(&mut self, amount: U128, account_id: AccountId) -> U128 {
        assert_eq!(env::promise_results_count(), 1, "ERR_TOO_MANY_RESULTS");
        match env::promise_result(0) {
            PromiseResult::NotReady => unreachable!(),
            PromiseResult::Successful(_value) => {
                let upgradable_account: UpgradableAccount = self.accounts.get(&account_id).unwrap();
                let mut account: Account = Account::from(upgradable_account);

                // update account data
                account.pre_reward = 0;
                account.last_block_balance_change = env::block_height();

                self.accounts.insert(&account_id, &UpgradableAccount::from(account));
                self.total_paid_reward_balance += amount.0;

                amount
            },
            PromiseResult::Failed => env::panic_str("ERR_CALL_FAILED"),
        }
    }

    #[private]
    pub fn ft_withdraw_callback(&mut self, account_id: AccountId, old_account: Account) -> U128 {
        assert_eq!(env::promise_results_count(), 1, "ERR_TOO_MANY_RESULTS");
        match env::promise_result(0) {
            PromiseResult::NotReady => unreachable!(),
            PromiseResult::Successful(_value) => {
                U128(old_account.unstake_balance)
            },
            PromiseResult::Failed => {
                // Handle rollback data
                self.accounts.insert(&account_id, &UpgradableAccount::from(old_account));
                U128(0)
            },
        }
    }
}