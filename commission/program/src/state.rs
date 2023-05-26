use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::pubkey::Pubkey;
use lib::instructions::commission::{MAX_TOKENS_COUNT, MAX_TOKEN_SIZE};
use std::mem::size_of;

pub const MAX_ADMIN_SIZE: usize = MAX_TOKENS_COUNT * (MAX_TOKEN_SIZE + 8) + (32 as usize) + (8 as usize);

#[repr(C)]
#[derive(BorshSerialize, BorshDeserialize, PartialEq, Debug, Clone)]
pub enum OperationType {
    AddToken,
    RemoveToken,
    UpdateToken,
    WithdrawToken,
}

impl std::convert::Into<u8> for OperationType {
    fn into(self) -> u8 {
        match self {
            OperationType::AddToken => 0,
            OperationType::RemoveToken => 1,
            OperationType::UpdateToken => 2,
            OperationType::WithdrawToken => 3,
        }
    }
}

#[repr(C)]
#[derive(BorshSerialize, BorshDeserialize, PartialEq, Debug, Clone)]
pub struct CommissionToken {
    pub token: lib::CommissionToken,
    pub amount: u64,
}

impl CommissionToken {
    pub fn from(value: &lib::instructions::commission::CommissionTokenArg) -> Self {
        CommissionToken {
            token: value.token.clone(),
            amount: value.amount,
        }
    }
}

#[repr(C)]
#[derive(BorshSerialize, BorshDeserialize, PartialEq, Debug, Clone)]
pub struct CommissionAdmin {
    pub acceptable_tokens: Vec<CommissionToken>,
    pub add_token_nonce: u64,
    pub update_token_nonce: u64,
    pub remove_token_nonce: u64,
    pub withdraw_token_nonce: u64,
    pub is_initialized: bool,
}