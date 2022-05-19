use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    msg,
    program::{invoke, invoke_signed},
    program_error::ProgramError,
    pubkey::Pubkey,
};
use crate::instruction::BridgeInstruction;
use mpl_token_metadata::state::Data;

pub fn process_instruction<'a>(
    program_id: &'a Pubkey,
    accounts: &'a [AccountInfo<'a>],
    input: &[u8],
) -> ProgramResult {
    let instruction = BridgeInstruction::try_from_slice(input)?;
    match instruction {
        BridgeInstruction::InitializeAdmin => {
            msg!("Instruction: Create Bridge Admin");
            process_init_admin(program_id, accounts)
        }
        BridgeInstruction::TransferOwnership => {
            msg!("Instruction: Transfer Bridge Admin ownership");
            process_transfer_ownership(program_id, accounts)
        }
        BridgeInstruction::Deposit(args) => {
            msg!("Instruction: Deposit token");
            process_deposit(program_id, accounts, args.network_to, args.receiver_address, args.nonce)
        }
        BridgeInstruction::Withdraw(args) => {
            msg!("Instruction: Withdraw token");
            process_withdraw(program_id, accounts, args.deposit_tx, args.network_from, args.sender_address, args.data)
        }
    }
}

pub fn process_init_admin<'a>(
    program_id: &'a Pubkey,
    accounts: &'a [AccountInfo<'a>],
) -> ProgramResult {
    // TODO
    Ok(())
}

pub fn process_transfer_ownership<'a>(
    program_id: &'a Pubkey,
    accounts: &'a [AccountInfo<'a>],
) -> ProgramResult {
    // TODO
    Ok(())
}

pub fn process_deposit<'a>(
    program_id: &'a Pubkey,
    accounts: &'a [AccountInfo<'a>],
    network: String,
    receiver: String,
    nonce: String,
) -> ProgramResult {
    // TODO
    Ok(())
}

pub fn process_withdraw<'a>(
    program_id: &'a Pubkey,
    accounts: &'a [AccountInfo<'a>],
    tx: String,
    network: String,
    sender: String,
    data: Data,
) -> ProgramResult {
    // TODO
    Ok(())
}