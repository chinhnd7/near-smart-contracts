use near_sdk::collections::LookupMap;
use near_sdk::{env, AccountId, Balance, BlockHeight, EpochHeight, PanicOnDefault, BorshStorageKey, near_bindgen};
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::serde::{Deserialize, Serialize};

use crate::config::*;
use crate::account::*;
mod config;
mod account;

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
    pub accounts: LookupMap<AccountId, Account>, // thông tin chi tiết của account map theo account id
    pub paused: bool // nếu hết token không thể trả cho user, pause contract, user sẽ không deposit thêm và reward cũng không trả thêm nữa
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
            paused: false
        }
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
        builder.current_account_id(account_id(0))
        .signer_account_id(accounts(0))
        .predecessor_account_id(accounts(0))
        .is_view(is_view);

        builder
    }

    fn test_init_contract() {
        let context = get_context(false);
        testing_env!(context.build());

        let config: ConfigForReward = ConfigForReward {
            reward_numerator: 500,
            reward_denumerator: 100000,
        };

        let contract = StakingContract::new(accounts(1).to_string(), "ft_contract".to_string(), config);

        assert_eq!(contract.owner_id, accounts(1).to_string());
        assert_eq!(contract.ft_contract_id, "ft_contract".to_string());
        assert_eq!(config.reward_numerator, contract.config.reward_numerator);
        assert_eq!(contract.paused, false);
    }
}
