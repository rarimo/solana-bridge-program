use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::pubkey::Pubkey;

pub const COMMISSION_ADMIN_PDA_SEED: &str = "commission_admin";

#[repr(C)]
#[derive(BorshSerialize, BorshDeserialize, PartialEq, Debug, Clone)]
pub enum TokenType {
    Native,
    NFT,
    FT,
}

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