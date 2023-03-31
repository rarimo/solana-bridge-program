//! Error types

use num_derive::FromPrimitive;
use solana_program::{decode_error::DecodeError, msg, program_error::{PrintProgramError, ProgramError}};
use thiserror::Error;

/// Errors that may be returned by the Token program.
#[derive(Clone, Debug, Eq, Error, FromPrimitive, PartialEq)]
pub enum CommissionError {
    /// 0 Invalid PDA
    #[error("Wrong PDA account")]
    WrongPDA,
    /// 1 The account cannot be initialized because it is already being used.
    #[error("Already in use")]
    AlreadyInUse,
    /// 2 The account hasn't been initialized
    #[error("Not initialized")]
    NotInitialized,
    /// 3 Token is not acceptable to charge commission in
    #[error("Not acceptable")]
    NotAcceptable,
    /// 4 Token is not supported yet
    #[error("Not supported")]
    NotSupported,
    /// 5 Wrong token account
    #[error("Wrong token account")]
    WrongTokenAccount,
}


impl From<CommissionError> for ProgramError {
    fn from(e: CommissionError) -> Self {
        ProgramError::Custom(e as u32)
    }
}

impl PrintProgramError for CommissionError {
    fn print<E>(&self) {
        msg!(&self.to_string());
    }
}

impl<T> DecodeError<T> for CommissionError {
    fn type_of() -> &'static str {
        "BridgeError"
    }
}
