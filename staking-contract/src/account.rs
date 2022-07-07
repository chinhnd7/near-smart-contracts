use crate::*;

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize)]
pub struct Account {
    pub stake_balance: Balance,
    pub pre_reward: Balance,
    pub last_block_balance_change: BlockHeight,
    pub unstake_balance: Balance,
    pub unstake_start_timestamp: Timestamp,
    pub unstake_available_epoch: EpochHeight, 
}

// Timeline: t1 ------------ t2 ------------ now
// Balance: 100k       deposit thêm 200k