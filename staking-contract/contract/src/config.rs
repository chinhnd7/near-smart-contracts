use crate::*;

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone, Copy)]
#[serde(crate = "near_sdk::serde")]
pub struct ConfigForReward {
    pub reward_numerator: u32,
    pub reward_denumerator: u64,
}

impl ConfigForReward {
    
}

impl  Default for ConfigForReward {
    fn default() -> Self {
        // APR 15% - 18%
        Self { reward_numerator: 715, reward_denumerator: 100000000000 } // reward per block
    }
}
// APR 15% = (token * 715 / 100000000000) * total block