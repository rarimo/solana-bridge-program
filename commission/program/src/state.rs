use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::pubkey::Pubkey;
use std::mem::size_of;

pub const MAX_TOKENS_COUNT: usize = 10;

pub const MAX_TOKEN_SIZE: usize = size_of::<lib::CommissionToken>() + 32;
pub const MAX_ADMIN_SIZE: usize = MAX_TOKENS_COUNT * (MAX_TOKEN_SIZE + 8) + (8 as usize);


#[repr(C)]
#[derive(BorshSerialize, BorshDeserialize, PartialEq, Debug, Clone)]
pub struct CommissionToken {
    pub token: lib::CommissionToken,
    pub amount: u64,
}

#[repr(C)]
#[derive(BorshSerialize, BorshDeserialize, PartialEq, Debug, Clone)]
pub struct CommissionAdmin {
    pub acceptable_tokens: Vec<CommissionToken>,
    pub is_initialized: bool,
}