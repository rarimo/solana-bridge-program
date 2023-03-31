use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::pubkey::Pubkey;
use crate::state::TokenType;

#[repr(C)]
#[derive(BorshSerialize, BorshDeserialize, PartialEq, Debug, Clone)]
pub enum CommissionToken {
    Native,
    FT(Pubkey),
    NFT(Pubkey),
}

#[repr(C)]
#[derive(BorshSerialize, BorshDeserialize, PartialEq, Debug, Clone)]
pub struct CommissionArgs {
    pub token: CommissionToken,
    pub deposit_token: TokenType,
    pub deposit_token_amount: u64,
}