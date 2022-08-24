use solana_program::pubkey::Pubkey;
use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::secp256k1_recover::SECP256K1_PUBLIC_KEY_LENGTH;

pub const BRIDGE_ADMIN_SIZE: usize = 33;

pub const MAX_NETWORKS_SIZE: usize = 20;
pub const MAX_ADDRESS_SIZE: usize = 64;
pub const MAX_TOKEN_ID_SIZE: usize = 100;
pub const MAX_TX_SIZE: usize = 100;

pub const DEPOSIT_SIZE: usize = MAX_NETWORKS_SIZE + 2 * MAX_ADDRESS_SIZE + MAX_TOKEN_ID_SIZE + 1;
pub const WITHDRAW_SIZE: usize = MAX_NETWORKS_SIZE + MAX_ADDRESS_SIZE + MAX_TOKEN_ID_SIZE + 1;


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
pub struct Deposit {
    pub token_type: TokenType,
    // None for native token
    pub mint: Option<Pubkey>,
    pub amount: u64,
    // Network to
    pub network: String,
    pub receiver_address: String,
    pub is_initialized: bool,
}

#[repr(C)]
#[derive(BorshSerialize, BorshDeserialize, PartialEq, Debug, Clone)]
pub struct Withdraw {
    pub token_type: TokenType,
    pub mint: Option<Pubkey>,
    pub amount: u64,
    // Network from
    pub network: String,
    pub receiver_address: Pubkey,
    pub is_initialized: bool,
}