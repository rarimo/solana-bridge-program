use solana_program::pubkey::Pubkey;
use borsh::{BorshDeserialize, BorshSerialize};

pub const BRIDGE_ADMIN_SIZE: usize = 33;

pub const MAX_NETWORKS_SIZE: usize = 20;
pub const MAX_ADDRESS_SIZE: usize = 64;
pub const MAX_TOKEN_ID_SIZE: usize = 64;

pub const DEPOSIT_SIZE: usize = MAX_NETWORKS_SIZE + MAX_ADDRESS_SIZE + 1;
pub const WITHDRAW_SIZE: usize = MAX_NETWORKS_SIZE + MAX_ADDRESS_SIZE + 1;


#[repr(C)]
#[derive(BorshSerialize, BorshDeserialize, PartialEq, Debug, Clone)]
pub struct BridgeAdmin {
    pub admin: Pubkey,
    pub is_initialized: bool,
}

#[repr(C)]
#[derive(BorshSerialize, BorshDeserialize, PartialEq, Debug, Clone)]
pub struct Deposit {
    pub network: String,
    pub receiver_address: String,
    // original collection address
    pub address: Option<String>,
    // original token id
    pub token_id: Option<String>,
    pub is_initialized: bool,
}

#[repr(C)]
#[derive(BorshSerialize, BorshDeserialize, PartialEq, Debug, Clone)]
pub struct Withdraw {
    pub network: String,
    pub sender_address: String,
    pub token_id: Option<String>,
    pub is_initialized: bool,
}