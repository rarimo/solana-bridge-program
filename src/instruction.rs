use mpl_token_metadata::state::Data;
use borsh::{BorshDeserialize, BorshSerialize};

#[repr(C)]
#[derive(BorshSerialize, BorshDeserialize, PartialEq, Debug, Clone)]
pub struct DepositArgs {
    pub network_to: String,
    pub receiver_address: String,
    pub nonce: String,
}

pub struct WithdrawArgs {
    pub deposit_tx: String,
    pub network_from: String,
    pub sender_address: String,
    pub data: Data,
}


#[derive(BorshSerialize, BorshDeserialize, Clone)]
pub enum BridgeInstruction {
    /// Accounts expected by this instruction:
    ///
    ///   0. `[writable]` The BridgeAdmin account to initialize
    ///   1. `[]` System program
    ///   2. `[]` Token program id
    ///   3. `[]` Rent sysvar
    InitializeAdmin,

    /// Accounts expected by this instruction:
    ///
    ///   0. `[writable]` The BridgeAdmin account
    ///   1. `[signer]` Current admin account
    ///   2. `[]` New admin account
    ///
    TransferOwnership,

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
    Deposit(DepositArgs),

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
    Withdraw(WithdrawArgs),
}