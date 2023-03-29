use std::mem::size_of;

use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::pubkey::Pubkey;
use solana_program::secp256k1_recover::SECP256K1_PUBLIC_KEY_LENGTH;

pub const MAX_NETWORKS_SIZE: usize = 20;
pub const MAX_ADDRESS_SIZE: usize = 100;
pub const MAX_TOKEN_ID_SIZE: usize = 100;
pub const MAX_TX_SIZE: usize = 100;

pub const BRIDGE_ADMIN_SIZE: usize = SECP256K1_PUBLIC_KEY_LENGTH + 1;
pub const WITHDRAW_SIZE: usize = size_of::<TokenType>() + (32 as usize) + (8 as usize) + MAX_NETWORKS_SIZE + MAX_ADDRESS_SIZE + 1;


#[repr(C)]
#[derive(BorshSerialize, BorshDeserialize, PartialEq, Debug, Clone)]
pub enum TokenType {
    Native,
    NFT,
    FT,
}

#[repr(C)]
#[derive(BorshSerialize, BorshDeserialize, PartialEq, Debug, Clone)]
pub struct BridgeAdmin {
    pub public_key: [u8; SECP256K1_PUBLIC_KEY_LENGTH],
    pub is_initialized: bool,
}

#[repr(C)]
#[derive(BorshSerialize, BorshDeserialize, PartialEq, Debug, Clone)]
pub struct Withdraw {
    pub token_type: TokenType,
    pub mint: Option<Pubkey>,
    pub amount: u64,
    // Hash of deposit tx info. See spec in core for more information.
    pub origin: [u8; 32],
    pub receiver_address: Pubkey,
    pub is_initialized: bool,
}