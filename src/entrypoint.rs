use solana_program::{
    account_info::AccountInfo,
    declare_id,
    entrypoint,
    entrypoint::ProgramResult,
    pubkey::Pubkey,
};
use solana_program::program_error::PrintProgramError;

use crate::processor;
use crate::error::BridgeError;

declare_id!("8RuX2EomaZj5xiEyU78XpDWRRp5wou4QNckHnkYX2Fgs");
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
