use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    entrypoint::ProgramResult,
    instruction::{AccountMeta, Instruction},
    pubkey::Pubkey,
    sysvar,
};
use solana_program::secp256k1_recover::{SECP256K1_PUBLIC_KEY_LENGTH, SECP256K1_SIGNATURE_LENGTH};
use std::mem::size_of;



#[repr(C)]
#[derive(BorshSerialize, BorshDeserialize, PartialEq, Debug, Clone)]
pub struct InitializeAdminArgs {
    // ECDSA public key
    pub public_key: [u8; SECP256K1_PUBLIC_KEY_LENGTH],
    pub contract: Pubkey,
}

#[repr(C)]
#[derive(BorshSerialize, BorshDeserialize, PartialEq, Debug, Clone)]
pub struct TransferOwnershipArgs {
    // New ECDSA public key
    pub new_public_key: [u8; SECP256K1_PUBLIC_KEY_LENGTH],
    // Signature of new_public_key by old public key
    pub signature: [u8; SECP256K1_SIGNATURE_LENGTH],
    pub recovery_id: u8,
}

#[repr(C)]
#[derive(BorshSerialize, BorshDeserialize, PartialEq, Debug, Clone)]
pub struct UpgradeArgs {
    pub signature: [u8; SECP256K1_PUBLIC_KEY_LENGTH],
    pub recovery_id: u8,
}

#[derive(BorshSerialize, BorshDeserialize, Clone)]
pub enum UpgradeInstruction {
    /// Initialize new UpgradeAdmin that will store acceptable token
    ///
    /// Accounts expected by this instruction:
    ///
    ///   0. `[writable]` The UpgradeAdmin account to initialize
    ///   1. `[writable,signer]` The fee payer
    ///   2. `[]` System program
    ///   3. `[]` Rent sysvar
    InitializeAdmin(InitializeAdminArgs),

    /// Change pubkey in UpgradeAdmin.
    ///
    /// Accounts expected by this instruction:
    ///
    ///   0. `[writable]` The UpgradeAdmin account
    TransferOwnership(TransferOwnershipArgs),


    /// Upgrade contract
    ///
    /// Accounts expected by this instruction:
    ///
    ///   0. `[writable]` The UpgradeAdmin account
    ///   1. `[writable]` The ProgramData account.
    ///   2. `[writable]` The Program account corresponding to stores address in UpgradeAdmin.
    ///   3. `[writable]` The Buffer account where the program data has been
    ///      written.  The buffer account's authority must match the program's
    ///      authority
    ///   4. `[writable]` The spill account.
    ///   5. `[]` Rent sysvar.
    ///   6. `[]` Clock sysvar.
    Upgrade(UpgradeArgs),
}