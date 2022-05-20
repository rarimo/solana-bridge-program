//! Error types

use num_derive::FromPrimitive;
use solana_program::{decode_error::DecodeError, msg, program_error::{PrintProgramError, ProgramError}};
use thiserror::Error;

/// Errors that may be returned by the Token program.
#[derive(Clone, Debug, Eq, Error, FromPrimitive, PartialEq)]
pub enum BridgeError {
    /// The account cannot be initialized because it is already being used.
    #[error("Already in use")]
    AlreadyInUse,
    /// Lamport balance below rent-exempt threshold.
    #[error("Lamport balance below rent-exempt threshold")]
    NotRentExempt,
    /// The account hasn't been initialized
    #[error("Not initialized")]
    NotInitialized,
    /// Admin signature was not provided
    #[error("No signature")]
    UnsignedAdmin,
    /// Created account data length is wrong
    #[error("Wrong data len")]
    WrongDataLen,
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
