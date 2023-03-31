use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    entrypoint::ProgramResult,
    instruction::{AccountMeta, Instruction},
    pubkey::Pubkey,
    sysvar,
};
use crate::state::CommissionToken;

#[repr(C)]
#[derive(BorshSerialize, BorshDeserialize, PartialEq, Debug, Clone)]
pub struct InitializeAdminArgs {
    pub acceptable_tokens: Vec<CommissionToken>,
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
    ChargeCommission(lib::CommissionArgs),
}