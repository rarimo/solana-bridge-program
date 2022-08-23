//! Error types

use num_derive::FromPrimitive;
use solana_program::{decode_error::DecodeError, msg, program_error::{PrintProgramError, ProgramError}};
use thiserror::Error;

/// Errors that may be returned by the Token program.
#[derive(Clone, Debug, Eq, Error, FromPrimitive, PartialEq)]
pub enum BridgeError {
    /// 0 The account cannot be initialized because it is already being used.
    #[error("Already in use")]
    AlreadyInUse,
    /// 1 Lamport balance below rent-exempt threshold.
    #[error("Lamport balance below rent-exempt threshold")]
    NotRentExempt,
    /// 2 The account hasn't been initialized
    #[error("Not initialized")]
    NotInitialized,
    /// 3 Admin signature was not provided
    #[error("No signature")]
    UnsignedAdmin,
    /// 4 Wrong admin account
    #[error("Wrong admin")]
    WrongAdmin,
    /// 5 Created account data length is wrong
    #[error("Wrong data len")]
    WrongDataLen,
    /// 6 Wrong seeds for admin account
    #[error("Wrong seeds")]
    WrongSeeds,
    /// 7 Wrong nonce for deposit account
    #[error("Wrong nonce")]
    WrongNonce,
    /// 8 Wrong token account
    #[error("Wrong token account")]
    WrongTokenAccount,
    /// 9 Wrong token mint account
    #[error("Wrong token metadata account")]
    WrongMetadataAccount,
    /// 10 Wrong arguments size
    #[error("Wrong arguments size")]
    WrongArgsSize,
    /// 11 Wrong mint account key
    #[error("Wrong mint key")]
    WrongMint,
    /// 12 Wrong Merkle path array
    #[error("Wrong merkle path")]
    WrongMerklePath,
    /// 13 Wrong Merkle root
    #[error("Wrong merkle root")]
    WrongMerkleRoot,
    /// 14 Wrong content hash
    #[error("Wrong content hash")]
    WrongContentHash,
    /// 15 Wrong signature key
    #[error("Wrong signature key")]
    WrongSignature,
}

impl From<BridgeError> for ProgramError {
    fn from(e: BridgeError) -> Self {
        ProgramError::Custom(e as u32)
    }
}

impl PrintProgramError for BridgeError {
    fn print<E>(&self) {
        msg!(&self.to_string());
    }
}

impl<T> DecodeError<T> for BridgeError {
    fn type_of() -> &'static str {
        "BridgeError"
    }
}
