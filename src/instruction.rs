use mpl_token_metadata::state::Data;
use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::pubkey::Pubkey;

#[repr(C)]
#[derive(BorshSerialize, BorshDeserialize, PartialEq, Debug, Clone)]
pub struct InitBridgeStateArgs {
    pub admin: Pubkey,
}

#[repr(C)]
#[derive(BorshSerialize, BorshDeserialize, PartialEq, Debug, Clone)]
pub struct TransferOwnershipArgs {
    pub new_admin: Pubkey,
}

#[repr(C)]
#[derive(BorshSerialize, BorshDeserialize, PartialEq, Debug, Clone)]
pub struct DepositArgs {
    pub network_to: String,
    pub receiver_address: String,
    pub nonce: String,
}

#[repr(C)]
#[derive(BorshSerialize, BorshDeserialize, PartialEq, Debug, Clone)]
pub struct WithdrawArgs {
    pub deposit_tx: String,
    pub network_from: String,
    pub sender_address: String,
    pub data: Data,
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
    InitializeAdmin(InitBridgeStateArgs),

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
    ///   6. `[]` System program
    ///   7. `[]` Token program id
    ///   8. `[]` Rent sysvar
    DepositMetaplex(DepositArgs),

   /// Make token withdraw from bridge.
   /// Contract will transfer existing token or mint and trnasfer the new one.
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
   ///   6. `[]` System program
   ///   7. `[]` Token program id
   ///   8. `[]` Rent sysvar
    WithdrawMetaplex(WithdrawArgs),
}