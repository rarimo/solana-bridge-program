use std::str::FromStr;
use solana_program::{pubkey, pubkey::Pubkey};
use borsh::{BorshDeserialize, BorshSerialize};
use shank::ShankAccount;

#[repr(C)]
#[derive(BorshSerialize, BorshDeserialize, PartialEq, Debug, Clone, ShankAccount)]
pub struct BridgeAdmin {
    pub admin: Pubkey,
    pub is_initialized: bool,
}

#[repr(C)]
#[derive(BorshSerialize, BorshDeserialize, PartialEq, Debug, Clone, ShankAccount)]
pub struct Deposit {
    pub network: String,
    pub receiver_address: String,
    pub is_initialized: bool,
}

#[repr(C)]
#[derive(BorshSerialize, BorshDeserialize, PartialEq, Debug, Clone, ShankAccount)]
pub struct Withdraw {
    pub network: String,
    pub sender_address: String,
    pub is_initialized: bool,
}