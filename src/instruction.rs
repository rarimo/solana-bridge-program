use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    pubkey::Pubkey,
    instruction::{Instruction, AccountMeta},
    sysvar,
    entrypoint::ProgramResult,
};
use mpl_token_metadata::state::DataV2;
use crate::util;
use solana_program::secp256k1_recover::{SECP256K1_PUBLIC_KEY_LENGTH, SECP256K1_SIGNATURE_LENGTH};
use spl_associated_token_account::get_associated_token_address;

#[repr(C)]
#[derive(BorshSerialize, BorshDeserialize, PartialEq, Debug, Clone)]
pub struct InitializeAdminArgs {
    // ECDSA public key
    pub public_key: [u8; SECP256K1_PUBLIC_KEY_LENGTH],
    // Admin account seeds (also public)
    pub seeds: [u8; 32],
}

#[repr(C)]
#[derive(BorshSerialize, BorshDeserialize, PartialEq, Debug, Clone)]
pub struct TransferOwnershipArgs {
    // New ECDSA public key
    pub new_public_key: [u8; SECP256K1_PUBLIC_KEY_LENGTH],
    // Signature of new_public_key by old public key
    pub signature: [u8; SECP256K1_SIGNATURE_LENGTH],
    pub recovery_id: u8,
    // Admin account seeds
    pub seeds: [u8; 32],
}

#[repr(C)]
#[derive(BorshSerialize, BorshDeserialize, PartialEq, Debug, Clone)]
pub struct DepositNativeArgs {
    pub amount: u64,
    pub network_to: String,
    pub receiver_address: String,
    pub seeds: [u8; 32],
    pub bundle_data: Option<Vec<u8>>,
    pub bundle_seed: Option<[u8; 32]>,
}

#[repr(C)]
#[derive(BorshSerialize, BorshDeserialize, PartialEq, Debug, Clone)]
pub struct DepositFTArgs {
    pub amount: u64,
    pub network_to: String,
    pub receiver_address: String,
    pub seeds: [u8; 32],
    pub token_seed: Option<[u8; 32]>,
    pub bundle_data: Option<Vec<u8>>,
    pub bundle_seed: Option<[u8; 32]>,
}

#[repr(C)]
#[derive(BorshSerialize, BorshDeserialize, PartialEq, Debug, Clone)]
pub struct DepositNFTArgs {
    pub network_to: String,
    pub receiver_address: String,
    pub seeds: [u8; 32],
    pub token_seed: Option<[u8; 32]>,
    pub bundle_data: Option<Vec<u8>>,
    pub bundle_seed: Option<[u8; 32]>,
}

#[repr(C)]
#[derive(BorshSerialize, BorshDeserialize, PartialEq, Debug, Clone)]
pub struct SignedMetadata {
    pub name: String,
    pub symbol: String,
    pub uri: String,
    pub decimals: u8,
}

#[repr(C)]
#[derive(BorshSerialize, BorshDeserialize, PartialEq, Debug, Clone)]
pub struct WithdrawArgs {
    // Default: hash of tx | event_id | network_from
    pub origin: [u8; 32],
    pub amount: u64,
    // Signature for the Merkle root
    pub signature: [u8; SECP256K1_PUBLIC_KEY_LENGTH],
    pub recovery_id: u8,
    // Merkle path
    pub path: Vec<[u8; 32]>,
    pub seeds: [u8; 32],
    pub token_seed: Option<[u8; 32]>,
    pub signed_meta: Option<SignedMetadata>,
}

#[repr(C)]
#[derive(BorshSerialize, BorshDeserialize, PartialEq, Debug, Clone)]
pub struct MintCollectionArgs {
    pub data: SignedMetadata,
    pub seeds: [u8; 32],
    pub token_seed: [u8; 32],
}

#[repr(C)]
#[derive(BorshSerialize, BorshDeserialize, PartialEq, Debug, Clone)]
pub struct MintNFTArgs {
    pub data: DataV2,
    pub seeds: [u8; 32],
    pub verify: bool,
}

#[derive(BorshSerialize, BorshDeserialize, Clone)]
pub enum BridgeInstruction {
    /// Initialize new BridgeAdmin that will store ECDSA publick key
    ///
    /// Accounts expected by this instruction:
    ///
    ///   0. `[writable]` The BridgeAdmin account to initialize
    ///   1. `[writable,signer]` The fee payer
    ///   2. `[]` System program
    ///   3. `[]` Rent sysvar
    InitializeAdmin(InitializeAdminArgs),

    /// Change admin in BridgeAdmin.
    ///
    /// Accounts expected by this instruction:
    ///
    ///   0. `[writable]` The BridgeAdmin account
    ///
    TransferOwnership(TransferOwnershipArgs),

    /// Make SOL deposit on bridge.
    ///
    /// Accounts expected by this instruction:
    ///
    ///   0. `[writable]` The BridgeAdmin account
    ///   1. `[writable,signer]` The owner account
    ///   2. `[]` System program
    ///   3. `[]` Rent sysvar
    DepositNative(DepositNativeArgs),

    /// Make FT deposit on bridge.
    ///
    /// Accounts expected by this instruction:
    ///
    ///   0. `[]` The BridgeAdmin account
    ///   1. `[writable]` The token mint account
    ///   2. `[writable]` The owner token associated account
    ///   3. `[writable]` The bridge token account
    ///   4. `[writable,signer]` The token owner account
    ///   5. `[]` Token program id
    ///   6. `[]` System program
    ///   7. `[]` Rent sysvar
    ///   8. `[]` Associated token program
    DepositFT(DepositFTArgs),

    /// Make NFT deposit on bridge.
    ///
    /// Accounts expected by this instruction:
    ///
    ///   0. `[]` The BridgeAdmin account
    ///   1. `[writable]` The token mint account
    ///   2. `[writable]` The owner token associated account
    ///   3. `[writable]` The bridge token account
    ///   4. `[writable,signer]` The token owner account
    ///   5. `[]` Token program id
    ///   6. `[]` System program
    ///   7. `[]` Rent sysvar
    ///   8. `[]` Associated token program
    DepositNFT(DepositNFTArgs),

    /// Make NFT withdraw from bridge.
    ///
    /// Accounts expected by this instruction:
    ///
    ///   0. `[writable]` The BridgeAdmin account
    ///   1. `[writable,signer]` The owner account
    ///   2. `[writable]` The new Withdraw account
    ///   3. `[]` System program
    ///   4. `[]` Rent sysvar
    WithdrawNative(WithdrawArgs),

    /// Make FT withdraw from bridge.
    ///
    /// Accounts expected by this instruction:
    ///
    ///   0. `[]` The BridgeAdmin account
    ///   1. `[writable]` The token mint account
    ///   2. `[writable]` The token metadata account
    ///   3. `[writable,signer]` The owner account
    ///   4. `[writable]` The owner token associated account
    ///   5. `[writable]` The bridge token account
    ///   6. `[writable]` The new Withdraw account
    ///   7. `[]` Token program id
    ///   8. `[]` System program
    ///   9. `[]` Rent sysvar
    ///   10. `[]` Metadata program
    ///   11. `[]` Associated token program
    WithdrawFT(WithdrawArgs),

    /// Make NFT withdraw from bridge.
    ///
    /// Accounts expected by this instruction:
    ///
    ///   0. `[]` The BridgeAdmin account
    ///   1. `[writable]` The token mint account
    ///   2. `[writable]` The token metadata account
    ///   3. `[writable,signer]` The owner account
    ///   4. `[writable]` The owner token associated account
    ///   5. `[writable]` The bridge token account
    ///   6. `[writable]` The new Withdraw account
    ///   7. `[]` Token program id
    ///   8. `[]` System program
    ///   9. `[]` Rent sysvar
    ///   10. `[]` Metadata program
    ///   11. `[]` Associated token program
    WithdrawNFT(WithdrawArgs),

    /// Create collection NFT owned by brisge
    /// Accounts expected by this instruction:
    ///
    ///   0. `[writable]` The BridgeAdmin account
    ///   1. `[writable,signed]` The token mint account
    ///   2. `[writable]` The bridge token account
    ///   3. `[writable]` The new metadata account
    ///   4. `[writable,signer]` The payer account
    ///   5. `[]` Token program id
    ///   6. `[]` Token metadata program id
    ///   7. `[]` Rent sysvar
    ///   8. `[]` System program
    ///   9. `[]` Associated token program
    MintCollection(MintCollectionArgs),
}

/*
pub fn initialize_admin(
    program_id: Pubkey,
    bridge_admin: Pubkey,
    fee_payer: Pubkey,
    public_key: [u8; SECP256K1_PUBLIC_KEY_LENGTH],
    seeds: [u8; 32],
) -> Instruction {
    Instruction {
        program_id,
        accounts: vec![
            AccountMeta::new(bridge_admin, false),
            AccountMeta::new(fee_payer, true),
            AccountMeta::new_readonly(sysvar::rent::id(), false),
        ],
        data: BridgeInstruction::InitializeAdmin(InitializeAdminArgs {
            public_key,
            seeds,
        }).try_to_vec().unwrap(),
    }
}

pub fn transfer_ownership(
    program_id: Pubkey,
    bridge_admin: Pubkey,
    signature: [u8; SECP256K1_SIGNATURE_LENGTH],
    new_public_key: [u8; SECP256K1_PUBLIC_KEY_LENGTH],
    seeds: [u8; 32],
) -> Instruction {
    Instruction {
        program_id,
        accounts: vec![
            AccountMeta::new(bridge_admin, false),
        ],
        data: BridgeInstruction::TransferOwnership(TransferOwnershipArgs {
            signature,
            new_public_key,
            seeds,
        }).try_to_vec().unwrap(),
    }
}


pub fn deposit_native(
    program_id: Pubkey,
    bridge_admin: Pubkey,
    deposit: Pubkey,
    owner: Pubkey,
    seeds: [u8; 32],
    network_to: String,
    amount: u64,
    receiver_address: String,
    nonce: [u8; 32],
) -> Instruction {
    Instruction {
        program_id,
        accounts: vec![
            AccountMeta::new_readonly(bridge_admin, false),
            AccountMeta::new(deposit, false),
            AccountMeta::new(owner, true),
            AccountMeta::new_readonly(spl_token::id(), false),
            AccountMeta::new_readonly(solana_program::system_program::id(), false),
            AccountMeta::new_readonly(sysvar::rent::id(), false),
        ],
        data: BridgeInstruction::DepositNative(DepositNativeArgs {
            amount,
            network_to,
            receiver_address,
            seeds,
            nonce,
        }).try_to_vec().unwrap(),
    }
}

pub fn deposit_ft(
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
    amount: u64,
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
            AccountMeta::new(owner, true),
            AccountMeta::new_readonly(spl_token::id(), false),
            AccountMeta::new_readonly(solana_program::system_program::id(), false),
            AccountMeta::new_readonly(sysvar::rent::id(), false),
            AccountMeta::new_readonly(spl_associated_token_account::id(), false),
        ],
        data: BridgeInstruction::DepositFT(DepositFTArgs {
            amount,
            network_to,
            receiver_address,
            seeds,
            nonce,
        }).try_to_vec().unwrap(),
    }
}

pub fn deposit_nft(
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
            AccountMeta::new(owner, true),
            AccountMeta::new_readonly(spl_token::id(), false),
            AccountMeta::new_readonly(solana_program::system_program::id(), false),
            AccountMeta::new_readonly(sysvar::rent::id(), false),
            AccountMeta::new_readonly(spl_associated_token_account::id(), false),
        ],
        data: BridgeInstruction::DepositNFT(DepositNFTArgs {
            network_to,
            receiver_address,
            seeds,
            nonce,
        }).try_to_vec().unwrap(),
    }
}

pub fn withdraw_native(
    program_id: Pubkey,
    bridge_admin: Pubkey,
    owner: Pubkey,
    withdraw: Pubkey,
    seeds: [u8; 32],
    content: SignedContent,
    signature: [u8; SECP256K1_SIGNATURE_LENGTH],
    path: Vec<[u8; 32]>,
    root: [u8; 32],
) -> Instruction {
    Instruction {
        program_id,
        accounts: vec![
            AccountMeta::new_readonly(bridge_admin, false),
            AccountMeta::new(owner, true),
            AccountMeta::new(withdraw, false),
            AccountMeta::new_readonly(spl_token::id(), false),
            AccountMeta::new_readonly(solana_program::system_program::id(), false),
            AccountMeta::new_readonly(sysvar::rent::id(), false),
        ],
        data: BridgeInstruction::WithdrawNative(WithdrawArgs {
            content,
            signature,
            path,
            root,
            seeds,
        }).try_to_vec().unwrap(),
    }
}

pub fn withdraw_ft(
    program_id: Pubkey,
    bridge_admin: Pubkey,
    mint: Pubkey,
    owner: Pubkey,
    owner_associated: Pubkey,
    bridge_associated: Pubkey,
    withdraw: Pubkey,
    seeds: [u8; 32],
    content: SignedContent,
    signature: [u8; SECP256K1_SIGNATURE_LENGTH],
    path: Vec<[u8; 32]>,
    root: [u8; 32],
) -> Instruction {
    Instruction {
        program_id,
        accounts: vec![
            AccountMeta::new_readonly(bridge_admin, false),
            AccountMeta::new_readonly(mint, false),
            AccountMeta::new(owner, true),
            AccountMeta::new(owner_associated, false),
            AccountMeta::new(bridge_associated, false),
            AccountMeta::new(withdraw, false),
            AccountMeta::new_readonly(spl_token::id(), false),
            AccountMeta::new_readonly(solana_program::system_program::id(), false),
            AccountMeta::new_readonly(sysvar::rent::id(), false),
            AccountMeta::new_readonly(spl_associated_token_account::id(), false),
        ],
        data: BridgeInstruction::WithdrawFT(WithdrawArgs {
            content,
            signature,
            path,
            root,
            seeds,
        }).try_to_vec().unwrap(),
    }
}

pub fn withdraw_nft(
    program_id: Pubkey,
    bridge_admin: Pubkey,
    mint: Pubkey,
    owner: Pubkey,
    owner_associated: Pubkey,
    bridge_associated: Pubkey,
    withdraw: Pubkey,
    seeds: [u8; 32],
    content: SignedContent,
    signature: [u8; SECP256K1_SIGNATURE_LENGTH],
    path: Vec<[u8; 32]>,
    root: [u8; 32],
) -> Instruction {
    Instruction {
        program_id,
        accounts: vec![
            AccountMeta::new_readonly(bridge_admin, false),
            AccountMeta::new_readonly(mint, false),
            AccountMeta::new(owner, true),
            AccountMeta::new(owner_associated, false),
            AccountMeta::new(bridge_associated, false),
            AccountMeta::new(withdraw, false),
            AccountMeta::new_readonly(spl_token::id(), false),
            AccountMeta::new_readonly(solana_program::system_program::id(), false),
            AccountMeta::new_readonly(sysvar::rent::id(), false),
            AccountMeta::new_readonly(spl_associated_token_account::id(), false),
        ],
        data: BridgeInstruction::WithdrawNFT(WithdrawArgs {
            content,
            signature,
            path,
            root,
            seeds,
        }).try_to_vec().unwrap(),
    }
}*/