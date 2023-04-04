use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::pubkey::Pubkey;

#[repr(C)]
#[derive(BorshSerialize, BorshDeserialize, PartialEq, Debug, Clone)]
pub struct CommissionToken {
    pub token: lib::CommissionToken,
    pub amount: u64,
}

impl CommissionToken {
    pub fn from(value: &lib::instructions::commission::CommissionTokenArg) -> Self {
        CommissionToken{
            token: value.token.clone(),
            amount: value.amount,
        }
    }
}

#[repr(C)]
#[derive(BorshSerialize, BorshDeserialize, PartialEq, Debug, Clone)]
pub struct CommissionAdmin {
    pub acceptable_tokens: Vec<CommissionToken>,
    pub is_initialized: bool,
}