use near_sdk::collections::LookupMap;
use near_sdk::{env, AccountId, Balance, BlockHeight, EpochHeight, PanicOnDefault, BorshStorageKey, near_bindgen, Promise, json_types::U128};
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::serde::{Deserialize, Serialize};

use crate::config::*;
use crate::account::*;
use crate::util::*;
use crate::internal::*;
use crate::enumeration::*;
use crate::core_impl::*;

mod config;
mod account;
mod util;
mod internal;
mod enumeration;
mod core_impl;

#[derive(BorshDeserialize, BorshSerialize, BorshStorageKey)]
pub enum StorageKey {
    AccountKey
}

#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
#[near_bindgen]
pub struct StakingContract {
    pub owner_id: AccountId,
    pub ft_contract_id: AccountId,
    pub config: ConfigForReward, // cấu hình công thức trả thưởng cho user
    pub total_stake_balance: Balance,
    pub total_paid_reward_balance: Balance,
    pub total_staker: Balance,
    pub pre_reward: Balance,
    pub last_block_balance_change: BlockHeight,
    pub accounts: LookupMap<AccountId, UpgradableAccount>, // thông tin chi tiết của account map theo account id
    pub paused: bool, // nếu hết token không thể trả cho user, pause contract, user sẽ không deposit thêm và reward cũng không trả thêm nữa
    pub pause_in_block: BlockHeight,
}

#[near_bindgen]
impl StakingContract {

    #[init]
    pub fn new_default_config(owner_id: AccountId, ft_contract_id: AccountId) -> Self {
        Self::new(owner_id, ft_contract_id, ConfigForReward::default())
    }

    #[init]
    pub fn new(owner_id: AccountId, ft_contract_id: AccountId, config: ConfigForReward) -> Self {
        StakingContract {
            owner_id,
            ft_contract_id,
            config,
            total_stake_balance: 0,
            total_paid_reward_balance: 0,
            total_staker: 0,
            pre_reward: 0,
            last_block_balance_change: env::block_height(),
            accounts: LookupMap::new(StorageKey::AccountKey),
            paused: false,
            pause_in_block: 0,
        }
    }

    // để 1 fn deposit được thì sẽ phải có macro này
    #[payable]
    pub fn storage_deposit(&mut self, account_id: Option<AccountId>) {
        assert_at_least_one_yocto();
        let account = account_id.unwrap_or_else(|| env::predecessor_account_id());
        let account_stake: Option<UpgradableAccount> = self.accounts.get(&account);

        if account_stake.is_some() {
            // refund toàn bộ token deposit
            refund_deposit(0);
        } else {
            // Tạo account mới
            let before_storage_useage: u64 = env::storage_usage();
            self.internal_register_account(account.clone());
            let after_storage_usage: u64 = env::storage_usage();

            // Refund lại token deposit còn thừa
            refund_deposit(after_storage_usage - before_storage_useage);
        }
    }

    pub fn storage_balance_of(&mut self, account_id: AccountId) -> U128 {
        let account: Option<UpgradableAccount> = self.accounts.get(&account_id);

        if account.is_some() {
            U128(1)
        } else {
            U128(0)
        }
    }

    pub fn is_pause(&self) -> bool {
        self.paused
    }
}

#[cfg(test)]
mod tests {
    use crate::config::*;

    use super::*;
    use near_sdk::{testing_env, MockedBlockchain};
    use near_sdk::test_utils::{ VMContextBuilder, accounts};

    fn get_context(is_view: bool) -> VMContextBuilder {
        let mut builder = VMContextBuilder::new();
        builder.current_account_id(accounts(0))
        .signer_account_id(accounts(0))
        .predecessor_account_id(accounts(0))
        .is_view(is_view);

        builder
    }

    #[test]
    fn test_init_contract() {
        let context = get_context(false);
        testing_env!(context.build());

        let config: ConfigForReward = ConfigForReward {
            reward_numerator: 500,
            reward_denumerator: 100000,
        };

        let contract = StakingContract::new(AccountId::new_unchecked(accounts(1).to_string()), AccountId::new_unchecked("ft_contract".to_string()), config);

        assert_eq!(contract.owner_id, AccountId::new_unchecked(accounts(1).to_string()));
        assert_eq!(contract.ft_contract_id, AccountId::new_unchecked("ft_contract".to_string()));
        assert_eq!(config.reward_numerator, contract.config.reward_numerator);
        assert_eq!(contract.paused, false);
    }
}