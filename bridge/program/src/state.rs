use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::pubkey::Pubkey;
use solana_program::secp256k1_recover::SECP256K1_PUBLIC_KEY_LENGTH;

#[repr(C)]
#[derive(BorshSerialize, BorshDeserialize, PartialEq, Debug, Clone)]
pub struct BridgeAdmin {
    pub public_key: [u8; SECP256K1_PUBLIC_KEY_LENGTH],
    pub commission_program: Pubkey,
    pub is_initialized: bool,
}

#[repr(C)]
#[derive(BorshSerialize, BorshDeserialize, PartialEq, Debug, Clone)]
pub struct Withdraw {
    pub token_type: lib::TokenType,
    pub mint: Option<Pubkey>,
    pub amount: u64,
    // Hash of deposit tx info. See spec in core for more information.
    pub origin: [u8; 32],
    pub receiver_address: Pubkey,
    pub is_initialized: bool,
}