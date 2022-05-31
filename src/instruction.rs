use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::pubkey::Pubkey;
use solana_program::{
    instruction::{Instruction, AccountMeta},
    sysvar,
};
use solana_program::entrypoint::ProgramResult;
use crate::state::{MAX_ADDRESS_SIZE, MAX_NETWORKS_SIZE};
use crate::error::BridgeError;
use solana_program::program_option::COption;

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
    pub token_id: Option<String>,
    pub seeds: [u8; 32],
    pub nonce: [u8; 32],
}

impl DepositArgs {
    pub fn validate(&self) -> ProgramResult {
        if self.receiver_address.as_bytes().len() > MAX_ADDRESS_SIZE || self.network_to.as_bytes().len() > MAX_NETWORKS_SIZE {
            return Err(BridgeError::WrongArgsSize.into());
        }

        if let Some(token_id) = &self.token_id {
            if token_id.as_bytes().len() > MAX_ADDRESS_SIZE {
                return Err(BridgeError::WrongArgsSize.into());
            }
        }

        Ok(())
    }
}

#[repr(C)]
#[derive(BorshSerialize, BorshDeserialize, PartialEq, Debug, Clone)]
pub struct WithdrawArgs {
    pub deposit_tx: String,
    pub network_from: String,
    pub sender_address: String,
    pub token_id: Option<String>,
    pub seeds: [u8; 32],
}

impl WithdrawArgs {
    pub fn validate(&self) -> ProgramResult {
        if self.sender_address.as_bytes().len() > MAX_ADDRESS_SIZE || self.network_from.as_bytes().len() > MAX_NETWORKS_SIZE {
            return Err(BridgeError::WrongArgsSize.into());
        }

        if let Some(token_id) = &self.token_id {
            if token_id.as_bytes().len() > MAX_ADDRESS_SIZE {
                return Err(BridgeError::WrongArgsSize.into());
            }
        }

        Ok(())
    }
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
    ///   3. `[writable]` The bridge token account
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
    ///   1. `[]` The token mint account
    ///   2. `[writable]` The owner token associated account
    ///   3. `[writable]` The bridge token account
    ///   4. `[writable]` The new Withdraw account
    ///   5. `[signer]` The admin account
    ///   6. `[]` Token program id
    ///   7. `[]` Rent sysvar
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


pub fn deposit_metaplex(
    program_id: Pubkey,
    bridge_admin: Pubkey,
    mint: Pubkey,
    owner_associated: Pubkey,
    bridge_associated: Pubkey,
    deposit: Pubkey,
    owner: Pubkey,
    seeds: [u8; 32],
    network_to: String,
    receiver_address: String,
    token_id: Option<String>,
    nonce: [u8; 32],
) -> Instruction {
    Instruction {
        program_id,
        accounts: vec![
            AccountMeta::new_readonly(bridge_admin, false),
            AccountMeta::new_readonly(mint, false),
            AccountMeta::new(owner_associated, false),
            AccountMeta::new(bridge_associated, false),
            AccountMeta::new(deposit, false),
            AccountMeta::new_readonly(owner, true),
            AccountMeta::new_readonly(spl_token::id(), false),
            AccountMeta::new_readonly(sysvar::rent::id(), false),
        ],
        data: BridgeInstruction::DepositMetaplex(DepositArgs {
            network_to,
            receiver_address,
            seeds,
            token_id,
            nonce,
        }).try_to_vec().unwrap(),
    }
}

pub fn withdraw_metaplex(
    program_id: Pubkey,
    bridge_admin: Pubkey,
    mint: Pubkey,
    owner_associated: Pubkey,
    bridge_associated: Pubkey,
    withdraw: Pubkey,
    admin: Pubkey,
    seeds: [u8; 32],
    deposit_tx: String,
    network_from: String,
    sender_address: String,
    token_id: Option<String>,
) -> Instruction {
    Instruction {
        program_id,
        accounts: vec![
            AccountMeta::new_readonly(bridge_admin, false),
            AccountMeta::new_readonly(mint, false),
            AccountMeta::new(owner_associated, false),
            AccountMeta::new(bridge_associated, false),
            AccountMeta::new(withdraw, false),
            AccountMeta::new_readonly(admin, true),
            AccountMeta::new_readonly(spl_token::id(), false),
            AccountMeta::new_readonly(sysvar::rent::id(), false),
        ],
        data: BridgeInstruction::WithdrawMetaplex(WithdrawArgs {
            deposit_tx,
            network_from,
            seeds,
            token_id,
            sender_address,
        }).try_to_vec().unwrap(),
    }
}