use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::pubkey::Pubkey;
use solana_program::{
    instruction::{Instruction, AccountMeta},
    sysvar,
};

#[repr(C)]
#[derive(BorshSerialize, BorshDeserialize, PartialEq, Debug, Clone)]
pub struct InitializeAdminArgs {
    pub admin: Pubkey,
    pub seeds: [u8; 32],
}

#[repr(C)]
#[derive(BorshSerialize, BorshDeserialize, PartialEq, Debug, Clone)]
pub struct TransferOwnershipArgs {
    pub new_admin: Pubkey,
    pub seeds: [u8; 32],
}

#[repr(C)]
#[derive(BorshSerialize, BorshDeserialize, PartialEq, Debug, Clone)]
pub struct DepositArgs {
    pub network_to: String,
    pub receiver_address: String,
    pub seeds: [u8; 32],
    pub nonce: [u8; 32],
}

#[repr(C)]
#[derive(BorshSerialize, BorshDeserialize, PartialEq, Debug, Clone)]
pub struct WithdrawArgs {
    pub deposit_tx: String,
    pub network_from: String,
    pub sender_address: String,
    pub seeds: [u8; 32],
}


#[derive(BorshSerialize, BorshDeserialize, Clone)]
pub enum BridgeInstruction {
    /// Initialize new BridgeAdmin that will manage contract operations.
    ///
    /// The `InitializeAdmin` instruction requires no signers and MUST be
    /// included within the same Transaction as the system program's
    /// `CreateAccount` instruction that creates the account being initialized.
    /// Otherwise another party can acquire ownership of the uninitialized
    /// account.
    ///
    /// Accounts expected by this instruction:
    ///
    ///   0. `[writable]` The BridgeAdmin account to initialize
    ///   1. `[]` Rent sysvar
    InitializeAdmin(InitializeAdminArgs),

    /// Change admin in BridgeAdmin.
    ///
    /// Accounts expected by this instruction:
    ///
    ///   0. `[writable]` The BridgeAdmin account
    ///   1. `[signer]` Current admin account
    ///
    TransferOwnership(TransferOwnershipArgs),

    /// Make token deposit on bridge.
    ///
    /// The `DepositMetaplex` MUST be included within the same Transaction as the system program's
    /// `CreateAccount` instruction for all new accounts.
    /// Otherwise another party can acquire ownership of the uninitialized
    /// account.
    ///
    /// Accounts expected by this instruction:
    ///
    ///   0. `[]` The BridgeAdmin account
    ///   1. `[]` The token mint account
    ///   2. `[writable]` The owner token associated account
    ///   3. `[writable]` The program token account
    ///   4. `[writable]` The new Deposit account
    ///   5. `[signer]` The token owner account
    ///   6. `[]` Token program id
    ///   7. `[]` Rent sysvar
    DepositMetaplex(DepositArgs),

    /// Make token withdraw from bridge.
    /// Contract will transfer existing token or mint and trnasfer the new on
    ///
    /// The `WithdrawMetaplex` MUST be included within the same Transaction as the system program's
    /// `CreateAccount` instruction for all new accounts.
    /// Otherwise another party can acquire ownership of the uninitialized
    /// account.
    ///
    /// Accounts expected by this instruction:
    ///
    ///   0. `[]` The BridgeAdmin account
    ///   1. `[writable]` The token mint account
    ///   2. `[writable]` The token metadata account
    ///   2. `[writable]` The owner token associated account
    ///   3. `[writable]` The program token account
    ///   4. `[writable]` The new Withdraw account
    ///   5. `[signer]` The admin account
    ///   7. `[]` Token program id
    ///   8. `[]` Rent sysvar
    WithdrawMetaplex(WithdrawArgs),
}

pub fn initialize_admin(
    program_id: Pubkey,
    bridge_admin: Pubkey,
    admin: Pubkey,
    seeds: [u8; 32],
) -> Instruction {
    Instruction {
        program_id,
        accounts: vec![
            AccountMeta::new(bridge_admin, false),
            AccountMeta::new_readonly(sysvar::rent::id(), false),
        ],
        data: BridgeInstruction::InitializeAdmin(InitializeAdminArgs {
            admin,
            seeds,
        }).try_to_vec().unwrap(),
    }
}

pub fn transfer_ownership(
    program_id: Pubkey,
    bridge_admin: Pubkey,
    admin: Pubkey,
    new_admin: Pubkey,
    seeds: [u8; 32],
) -> Instruction {
    Instruction {
        program_id,
        accounts: vec![
            AccountMeta::new(bridge_admin, false),
            AccountMeta::new_readonly(admin, true),
        ],
        data: BridgeInstruction::TransferOwnership(TransferOwnershipArgs {
            new_admin,
            seeds,
        }).try_to_vec().unwrap(),
    }
}