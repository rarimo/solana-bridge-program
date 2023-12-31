use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    entrypoint::ProgramResult,
    instruction::{AccountMeta, Instruction},
    pubkey::Pubkey,
    sysvar,
};
use solana_program::secp256k1_recover::SECP256K1_PUBLIC_KEY_LENGTH;
use crate::{CommissionToken, CommissionArgs, TokenType};
use std::mem::size_of;
use spl_associated_token_account::get_associated_token_address;

pub const MAX_TOKENS_COUNT: usize = 10;
pub const MAX_TOKEN_SIZE: usize = size_of::<CommissionToken>() + 32;

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
    pub signature: [u8; SECP256K1_PUBLIC_KEY_LENGTH],
    pub recovery_id: u8,
    pub path: Vec<[u8; 32]>,
    pub token: CommissionTokenArg,
}

#[repr(C)]
#[derive(BorshSerialize, BorshDeserialize, PartialEq, Debug, Clone)]
pub struct WithdrawArgs {
    pub signature: [u8; SECP256K1_PUBLIC_KEY_LENGTH],
    pub recovery_id: u8,
    pub path: Vec<[u8; 32]>,
    pub token: CommissionTokenArg,
    pub withdraw_amount: u64,
}

#[derive(BorshSerialize, BorshDeserialize, Clone)]
pub enum CommissionInstruction {
    /// Initialize new CommissionAdmin that will store acceptable token
    ///
    /// Accounts expected by this instruction:
    ///
    ///   0. `[writable]` The CommissionAdmin account to initialize
    ///   1. `[]` The BridgeAdmin account
    ///   2. `[writable,signer]` The fee payer
    ///   3. `[]` System program
    ///   4. `[]` Rent sysvar
    InitializeAdmin(InitializeAdminArgs),

    /// Charge commission for deposit
    ///
    /// Accounts expected by this instruction:
    ///
    ///   0. `[writable]` The CommissionAdmin account
    ///   1. `[]` The BridgeAdmin account
    ///   2. `[writable,signer]` The owner account
    ///   3. `[]` System program
    ///   4. `[]` Rent sysvar
    ///   5. `[]` SPL token program
    ///   6. `[writable]` Commission token owner associated account (Optional)
    ///   7. `[writable]` Commission token admin associated account (Optional)
    ///   8. `[]` Commission token mint account (Optional)
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

    /// Withdraw collected tokens from contract
    ///
    /// Accounts expected by this instruction:
    ///
    ///   0. `[]` The CommissionAdmin account
    ///   1. `[]` The BridgeAdmin account
    ///   2. `[writable, signer]` The receiver account (also fee payer)
    ///   3. `[]` System program
    ///   4. `[]` Rent sysvar
    ///   5. `[]` SPL token program
    ///   6. `[]` Commission token receiver associated account (Optional)
    ///   7. `[]` Commission token admin associated account (Optional)
    ///   8. `[]` Commission token mint account (Optional)
    Withdraw(WithdrawArgs),
}

pub fn charge_commission_native(
    program_id: Pubkey,
    commission_admin: Pubkey,
    bridge_admin: Pubkey,
    owner: Pubkey,
    token: CommissionToken,
    deposit_token: TokenType,
    deposit_token_amount: u64,
) -> Instruction {
    Instruction {
        program_id,
        accounts: vec![
            AccountMeta::new(commission_admin, false),
            AccountMeta::new_readonly(bridge_admin, false),
            AccountMeta::new(owner, true),
            AccountMeta::new_readonly(solana_program::system_program::id(), false),
            AccountMeta::new_readonly(sysvar::rent::id(), false),
            AccountMeta::new_readonly(spl_token::id(), false),
        ],
        data: CommissionInstruction::ChargeCommission(CommissionArgs {
            token,
            deposit_token,
            deposit_token_amount,
        }).try_to_vec().unwrap(),
    }
}

pub fn charge_commission_ft(
    program_id: Pubkey,
    commission_admin: Pubkey,
    bridge_admin: Pubkey,
    owner: Pubkey,
    mint: Pubkey,
    token: CommissionToken,
    deposit_token: TokenType,
    deposit_token_amount: u64,
) -> Instruction {
    let owner_associated = get_associated_token_address(&owner, &mint);
    let commission_associated = get_associated_token_address(&commission_admin, &mint);

    Instruction {
        program_id,
        accounts: vec![
            AccountMeta::new(commission_admin, false),
            AccountMeta::new_readonly(bridge_admin, false),
            AccountMeta::new(owner, true),
            AccountMeta::new_readonly(solana_program::system_program::id(), false),
            AccountMeta::new_readonly(sysvar::rent::id(), false),
            AccountMeta::new_readonly(spl_token::id(), false),
            AccountMeta::new(owner_associated, false),
            AccountMeta::new(commission_associated, false),
            AccountMeta::new_readonly(mint, false),
        ],
        data: CommissionInstruction::ChargeCommission(CommissionArgs {
            token,
            deposit_token,
            deposit_token_amount,
        }).try_to_vec().unwrap(),
    }
}