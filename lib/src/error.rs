//! Error types

use num_derive::FromPrimitive;
use solana_program::{decode_error::DecodeError, msg, program_error::{PrintProgramError, ProgramError}};
use thiserror::Error;

/// Errors that may be returned by the Token program.
#[derive(Clone, Debug, Eq, Error, FromPrimitive, PartialEq)]
pub enum ECDSAError {
    /// 0 Wrong signature key
    #[error("Wrong signature public key")]
    WrongSignature,
    /// 1 Invalid signature
    #[error("Invalid signature")]
    InvalidSignature,
}


impl From<ECDSAError> for ProgramError {
    fn from(e: ECDSAError) -> Self {
        ProgramError::Custom(e as u32)
    }
}

impl PrintProgramError for ECDSAError {
    fn print<E>(&self) {
        msg!(&self.to_string());
    }
}

impl<T> DecodeError<T> for ECDSAError {
    fn type_of() -> &'static str {
        "ECDSAError"
    }
}
