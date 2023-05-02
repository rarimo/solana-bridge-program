pub mod bridge;
pub mod commission;
pub mod upgrade;

use solana_program::entrypoint::ProgramResult;

pub trait InstructionValidation {
    fn validate(&self) -> ProgramResult;
}
