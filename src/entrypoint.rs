use solana_program::{
    account_info::{AccountInfo},
    entrypoint,
    entrypoint::{ProgramResult},
    pubkey::Pubkey,
    program_error::PrintProgramError,
};
use crate::{processor};
use crate::error::BridgeError;

entrypoint!(process_instruction);

fn process_instruction<'a>(
    program_id: &'a Pubkey,
    accounts: &'a [AccountInfo<'a>],
    instruction_data: &[u8],
) -> ProgramResult {
    match processor::process_instruction(program_id, accounts, instruction_data) {
        Ok(()) => Ok(()),
        Err(e) => {
            // catch the error so we can print it
            e.print::<BridgeError>();
            return Err(e);
        }
    }
}
