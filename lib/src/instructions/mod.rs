pub mod bridge;
pub mod commission;

use solana_program::entrypoint::ProgramResult;

pub trait InstructionValidation {
    fn validate(&self) -> ProgramResult;
}
