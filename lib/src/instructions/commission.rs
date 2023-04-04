use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    entrypoint::ProgramResult,
    instruction::{AccountMeta, Instruction},
    pubkey::Pubkey,
    sysvar,
};
use solana_program::secp256k1_recover::SECP256K1_PUBLIC_KEY_LENGTH;
use crate::{CommissionToken, CommissionArgs};
use std::mem::size_of;

pub const MAX_TOKENS_COUNT: usize = 10;

pub const MAX_TOKEN_SIZE: usize = size_of::<CommissionToken>() + 32;
pub const MAX_ADMIN_SIZE: usize = MAX_TOKENS_COUNT * (MAX_TOKEN_SIZE + 8) + (8 as usize);

#[repr(C)]
#[derive(BorshSerialize, BorshDeserialize, PartialEq, Debug, Clone)]
pub struct CommissionTokenArg {
    pub token: CommissionToken,
    pub amount: u64,
}

#[repr(C)]
#[derive(BorshSerialize, BorshDeserialize, PartialEq, Debug, Clone)]
pub struct InitializeAdminArgs {
    pub acceptable_tokens: Vec<CommissionTokenArg>,
}

#[repr(C)]
#[derive(BorshSerialize, BorshDeserialize, PartialEq, Debug, Clone)]
pub struct FeeTokenArgs {
    pub origin: [u8; 32],
    pub signature: [u8; SECP256K1_PUBLIC_KEY_LENGTH],
    pub recovery_id: u8,
    pub path: Vec<[u8; 32]>,
    pub token: CommissionTokenArg,
}

#[derive(BorshSerialize, BorshDeserialize, Clone)]
pub enum CommissionInstruction {
    /// Initialize new CommissionAdmin that will store acceptable token
    ///
    /// Accounts expected by this instruction:
    ///
    ///   0. `[writable]` The CommissionAdmin account to initialize
    ///   1. `[writable]` The BridgeAdmin account
    ///   2. `[writable,signer]` The fee payer
    ///   3. `[]` System program
    ///   4. `[]` Rent sysvar
    InitializeAdmin(InitializeAdminArgs),

    /// Charge commission for deposit
    ///
    /// Accounts expected by this instruction:
    ///
    ///   0. `[writable]` The CommissionAdmin account
    ///   1. `[writable,signer]` The owner account
    ///   2. `[]` System program
    ///   3. `[]` Rent sysvar
    ///   4. `[]` SPL token program
    ///   7. `[]` Commission token owner associated account (Optional)
    ///   5. `[]` Commission token admin associated account (Optional)
    ///   6. `[]` Commission token mint account (Optional)
    ChargeCommission(CommissionArgs),

    /// Add new acceptable commission token
    ///
    /// Accounts expected by this instruction:
    ///
    ///   0. `[writable]` The CommissionAdmin account
    ///   1. `[]` The BridgeAdmin account
    AddFeeToken(FeeTokenArgs),

    /// Remove new acceptable commission token
    ///
    /// Accounts expected by this instruction:
    ///
    ///   0. `[writable]` The CommissionAdmin account
    ///   1. `[]` The BridgeAdmin account
    RemoveFeeToken(FeeTokenArgs),

    /// Update certain acceptable commission token
    ///
    /// Accounts expected by this instruction:
    ///
    ///   0. `[writable]` The CommissionAdmin account
    ///   1. `[]` The BridgeAdmin account
    UpdateFeeToken(FeeTokenArgs),
}