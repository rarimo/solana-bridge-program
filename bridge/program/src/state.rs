use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::pubkey::Pubkey;
use solana_program::secp256k1_recover::SECP256K1_PUBLIC_KEY_LENGTH;
use lib::TokenType;
use lib::instructions::bridge::{MAX_NETWORKS_SIZE, MAX_ADDRESS_SIZE};
use std::mem::size_of;

pub const BRIDGE_ADMIN_SIZE: usize = SECP256K1_PUBLIC_KEY_LENGTH + (32 as usize) + 1;
pub const WITHDRAW_SIZE: usize = size_of::<TokenType>() + (32 as usize) + (8 as usize) + MAX_NETWORKS_SIZE + MAX_ADDRESS_SIZE + 1;

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