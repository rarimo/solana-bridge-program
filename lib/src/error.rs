//! Error types

use num_derive::FromPrimitive;
use solana_program::{decode_error::DecodeError, msg, program_error::{PrintProgramError, ProgramError}};
use thiserror::Error;

/// Errors that may be returned by the Token program.
#[derive(Clone, Debug, Eq, Error, FromPrimitive, PartialEq)]
pub enum LibError {
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
    #[error("Wrong signature public key")]
    WrongSignature,
    /// 16 Invalid signature
    #[error("Invalid signature")]
    InvalidSignature,
    /// 17 Wrong message for signing
    #[error("Invalid sign message")]
    InvalidMessage,
    /// 18 Invalid key
    #[error("Invalid key")]
    InvalidKey,
    /// 19 Wrong token type in the content
    #[error("Wrong token type")]
    WrongTokenType,
    /// 20 Wrong balance
    #[error("Wrong balance")]
    WrongBalance,
    /// 21 Uninitialized metadata
    #[error("Uninitialized metadata")]
    UninitializedMetadata,
    /// 22 Wrong token standard
    #[error("Wrong token standard")]
    WrongTokenStandard,
    /// 23 Wrong token seed
    #[error("Wrong token seed")]
    WrongTokenSeed,
    /// 24 No token metadata
    #[error("No token metadata provided")]
    NoTokenMeta,
    /// 25 Uninitialized mint
    #[error("Uninitialized mint")]
    UninitializedMint,
    /// 26 Wrong commission program
    #[error("Wrong commission program")]
    WrongCommissionProgram,
    /// 27 Wrong commission deposit arguments
    #[error("Wrong commission deposit arguments")]
    WrongCommissionArguments,
    /// 28 Wrong commission account
    #[error("Wrong commission account")]
    WrongCommissionAccount,
    /// 29 Token is not acceptable to charge commission in
    #[error("Not acceptable")]
    NotAcceptable,
    /// 30 Token is not supported yet
    #[error("Not supported")]
    NotSupported,
}


impl From<LibError> for ProgramError {
    fn from(e: LibError) -> Self {
        ProgramError::Custom(e as u32)
    }
}

impl PrintProgramError for LibError {
    fn print<E>(&self) {
        msg!(&self.to_string());
    }
}

impl<T> DecodeError<T> for LibError {
    fn type_of() -> &'static str {
        "LibError"
    }
}
