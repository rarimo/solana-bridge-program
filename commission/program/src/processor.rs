use solana_program::{
    account_info::{AccountInfo, next_account_info},
    entrypoint::ProgramResult, msg,
    program::{invoke, invoke_signed}, pubkey::Pubkey, system_instruction,
    sysvar::{rent::Rent, Sysvar},
};
use crate::state::{CommissionToken, CommissionAdmin, MAX_ADMIN_SIZE, OperationType};
use borsh::{
    BorshDeserialize, BorshSerialize,
};
use spl_token::instruction::transfer;
use spl_associated_token_account::get_associated_token_address;
use spl_associated_token_account::instruction::create_associated_token_account;
use solana_program::secp256k1_recover::SECP256K1_PUBLIC_KEY_LENGTH;
use lib::merkle::get_merkle_root;
use crate::merkle::Content;
use lib::ecdsa::verify_ecdsa_signature;
use lib::instructions::commission::{CommissionInstruction, CommissionTokenArg};
use lib::error::LibError;
use bridge::state::BridgeAdmin;

pub fn process_instruction<'a>(
    program_id: &'a Pubkey,
    accounts: &'a [AccountInfo<'a>],
    input: &[u8],
) -> ProgramResult {
    let instruction = CommissionInstruction::try_from_slice(input)?;
    match instruction {
        CommissionInstruction::InitializeAdmin(args) => {
            msg!("Instruction: Create Comission Admin");
            process_init_admin(program_id, accounts, args.acceptable_tokens)
        }
        CommissionInstruction::ChargeCommission(args) => {
            msg!("Instruction: Charge commission");
            process_charge_commission(program_id, accounts, args.token)
        }
        CommissionInstruction::AddFeeToken(args) => {
            msg!("Instruction: Add fee token");
            process_add_token(program_id, accounts, args.signature, args.recovery_id, args.path, args.token)
        }
        CommissionInstruction::RemoveFeeToken(args) => {
            msg!("Instruction: Remove fee token");
            process_remove_token(program_id, accounts, args.signature, args.recovery_id, args.path, args.token)
        }
        CommissionInstruction::UpdateFeeToken(args) => {
            msg!("Instruction: Update fee token");
            process_update_token(program_id, accounts, args.signature, args.recovery_id, args.path, args.token)
        }
        CommissionInstruction::Withdraw(args) => {
            msg!("Instruction: Withdraw collected tokens");
            process_withdraw(program_id, accounts,  args.signature, args.recovery_id, args.path, args.token, args.withdraw_amount)
        }
    }
}


pub fn process_init_admin<'a>(
    program_id: &'a Pubkey,
    accounts: &'a [AccountInfo<'a>],
    acceptable_tokens: Vec<CommissionTokenArg>,
) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();

    let commission_admin_info = next_account_info(account_info_iter)?;
    let bridge_admin_info = next_account_info(account_info_iter)?;

    let fee_payer_info = next_account_info(account_info_iter)?;
    let system_program = next_account_info(account_info_iter)?;
    let rent_info = next_account_info(account_info_iter)?;

    let commission_key = Pubkey::create_program_address(&[lib::COMMISSION_ADMIN_PDA_SEED.as_bytes(), bridge_admin_info.key.as_ref()], &program_id)?;
    if commission_key != *commission_admin_info.key {
        return Err(LibError::WrongAdmin.into());
    }

    lib::call_create_account(
        fee_payer_info,
        commission_admin_info,
        rent_info,
        system_program,
        MAX_ADMIN_SIZE,
        program_id,
        &[lib::COMMISSION_ADMIN_PDA_SEED.as_bytes(), bridge_admin_info.key.as_ref()],
    )?;

    let mut commission_admin: CommissionAdmin = BorshDeserialize::deserialize(&mut commission_admin_info.data.borrow_mut().as_ref())?;
    if commission_admin.is_initialized {
        return Err(LibError::AlreadyInUse.into());
    }

    commission_admin.acceptable_tokens = Vec::new();
    for t in acceptable_tokens {
        commission_admin.acceptable_tokens.push(CommissionToken::from(&t))
    }

    commission_admin.is_initialized = true;
    commission_admin.serialize(&mut *commission_admin_info.data.borrow_mut())?;
    Ok(())
}


pub fn process_charge_commission<'a>(
    program_id: &'a Pubkey,
    accounts: &'a [AccountInfo<'a>],
    token: lib::CommissionToken,
) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();

    let commission_admin_info = next_account_info(account_info_iter)?;
    let bridge_admin_info = next_account_info(account_info_iter)?;
    let owner_info = next_account_info(account_info_iter)?;
    let system_program = next_account_info(account_info_iter)?;
    let rent_info = next_account_info(account_info_iter)?;

    let commission_key = Pubkey::create_program_address(&[lib::COMMISSION_ADMIN_PDA_SEED.as_bytes(), bridge_admin_info.key.as_ref()], &program_id)?;
    if commission_key != *commission_admin_info.key {
        return Err(LibError::WrongAdmin.into());
    }

    let commission_admin: CommissionAdmin = BorshDeserialize::deserialize(&mut commission_admin_info.data.borrow_mut().as_ref())?;
    if !commission_admin.is_initialized {
        return Err(LibError::NotInitialized.into());
    }

    let commission_token = check_token_is_acceptable(commission_admin.acceptable_tokens, token)?;

    match commission_token.token.into() {
        lib::CommissionToken::Native => {
            call_transfer_native(
                owner_info,
                commission_admin_info,
                commission_token.amount,
                &[lib::COMMISSION_ADMIN_PDA_SEED.as_bytes(), bridge_admin_info.key.as_ref()],
            )?;
        }
        lib::CommissionToken::FT(mint) => {
            let token_program = next_account_info(account_info_iter)?;
            let owner_associated_info = next_account_info(account_info_iter)?;
            let commission_associated_info = next_account_info(account_info_iter)?;

            if *commission_associated_info.key !=
                get_associated_token_address(&commission_key, &mint) {
                return Err(LibError::WrongTokenAccount.into());
            }

            if commission_associated_info.data.borrow().as_ref().len() == 0 {
                msg!("Creating commission admin associated account");
                let mint_info = next_account_info(account_info_iter)?;
                lib::call_create_associated_account(
                    owner_info,
                    commission_admin_info,
                    mint_info,
                    commission_associated_info,
                    rent_info,
                    system_program,
                    token_program,
                )?;
            }

            call_transfer_ft(
                owner_associated_info,
                commission_associated_info,
                owner_info,
                commission_token.amount,
                &[lib::COMMISSION_ADMIN_PDA_SEED.as_bytes(), bridge_admin_info.key.as_ref()],
            )?;
        }
        lib::CommissionToken::NFT(mint) => {
            return Err(LibError::NotSupported.into());
        }
    }

    Ok(())
}


pub fn process_add_token<'a>(
    program_id: &'a Pubkey,
    accounts: &'a [AccountInfo<'a>],
    signature: [u8; SECP256K1_PUBLIC_KEY_LENGTH],
    recovery_id: u8,
    path: Vec<[u8; 32]>,
    token: CommissionTokenArg,
) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();

    let commission_admin_info = next_account_info(account_info_iter)?;
    let bridge_admin_info = next_account_info(account_info_iter)?;

    let commission_key = Pubkey::create_program_address(&[lib::COMMISSION_ADMIN_PDA_SEED.as_bytes(), bridge_admin_info.key.as_ref()], &program_id)?;
    if commission_key != *commission_admin_info.key {
        return Err(LibError::WrongAdmin.into());
    }

    let mut commission_admin: CommissionAdmin = BorshDeserialize::deserialize(&mut commission_admin_info.data.borrow_mut().as_ref())?;
    if commission_admin.is_initialized {
        return Err(LibError::NotInitialized.into());
    }

    let bridge_admin: BridgeAdmin = BorshDeserialize::deserialize(&mut bridge_admin_info.data.borrow_mut().as_ref())?;
    if !bridge_admin.is_initialized {
        return Err(LibError::NotInitialized.into());
    }

    let content = Content::new(
        commission_admin.add_token_nonce,
        None,
        *program_id,
        OperationType::AddToken,
        CommissionToken::from(&token),
    );

    let root = get_merkle_root(content.hash(), &path)?;
    verify_ecdsa_signature(root.as_slice(), signature.as_slice(), recovery_id, bridge_admin.public_key)?;

    commission_admin.add_token_nonce += 1;
    commission_admin.acceptable_tokens.push(CommissionToken::from(&token));
    commission_admin.serialize(&mut *commission_admin_info.data.borrow_mut())?;

    Ok(())
}

pub fn process_remove_token<'a>(
    program_id: &'a Pubkey,
    accounts: &'a [AccountInfo<'a>],
    signature: [u8; SECP256K1_PUBLIC_KEY_LENGTH],
    recovery_id: u8,
    path: Vec<[u8; 32]>,
    token: CommissionTokenArg,
) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();

    let commission_admin_info = next_account_info(account_info_iter)?;
    let bridge_admin_info = next_account_info(account_info_iter)?;

    let commission_key = Pubkey::create_program_address(&[lib::COMMISSION_ADMIN_PDA_SEED.as_bytes(), bridge_admin_info.key.as_ref()], &program_id)?;
    if commission_key != *commission_admin_info.key {
        return Err(LibError::WrongAdmin.into());
    }

    let mut commission_admin: CommissionAdmin = BorshDeserialize::deserialize(&mut commission_admin_info.data.borrow_mut().as_ref())?;
    if commission_admin.is_initialized {
        return Err(LibError::NotInitialized.into());
    }

    let bridge_admin: BridgeAdmin = BorshDeserialize::deserialize(&mut bridge_admin_info.data.borrow_mut().as_ref())?;
    if !bridge_admin.is_initialized {
        return Err(LibError::NotInitialized.into());
    }

    let content = Content::new(
        commission_admin.remove_token_nonce,
        None,
        *program_id,
        OperationType::RemoveToken,
        CommissionToken::from(&token),
    );
    let root = get_merkle_root(content.hash(), &path)?;
    verify_ecdsa_signature(root.as_slice(), signature.as_slice(), recovery_id, bridge_admin.public_key)?;

    let token_to_remove = CommissionToken::from(&token);
    for i in 0..commission_admin.acceptable_tokens.len() {
        if commission_admin.acceptable_tokens[i].eq(&token_to_remove) {
            commission_admin.acceptable_tokens.remove(i);
            break;
        }
    }

    commission_admin.remove_token_nonce += 1;
    commission_admin.serialize(&mut *commission_admin_info.data.borrow_mut())?;

    Ok(())
}

pub fn process_update_token<'a>(
    program_id: &'a Pubkey,
    accounts: &'a [AccountInfo<'a>],
    signature: [u8; SECP256K1_PUBLIC_KEY_LENGTH],
    recovery_id: u8,
    path: Vec<[u8; 32]>,
    token: CommissionTokenArg,
) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();

    let commission_admin_info = next_account_info(account_info_iter)?;
    let bridge_admin_info = next_account_info(account_info_iter)?;

    let commission_key = Pubkey::create_program_address(&[lib::COMMISSION_ADMIN_PDA_SEED.as_bytes(), bridge_admin_info.key.as_ref()], &program_id)?;
    if commission_key != *commission_admin_info.key {
        return Err(LibError::WrongAdmin.into());
    }

    let mut commission_admin: CommissionAdmin = BorshDeserialize::deserialize(&mut commission_admin_info.data.borrow_mut().as_ref())?;
    if commission_admin.is_initialized {
        return Err(LibError::NotInitialized.into());
    }

    let bridge_admin: BridgeAdmin = BorshDeserialize::deserialize(&mut bridge_admin_info.data.borrow_mut().as_ref())?;
    if !bridge_admin.is_initialized {
        return Err(LibError::NotInitialized.into());
    }

    let content = Content::new(
        commission_admin.update_token_nonce,
        None,
        *program_id,
        OperationType::UpdateToken,
        CommissionToken::from(&token),
    );
    let root = get_merkle_root(content.hash(), &path)?;
    verify_ecdsa_signature(root.as_slice(), signature.as_slice(), recovery_id, bridge_admin.public_key)?;

    let token_to_update = CommissionToken::from(&token);
    for i in 0..commission_admin.acceptable_tokens.len() {
        if commission_admin.acceptable_tokens[i].token.eq(&token_to_update.token) {
            commission_admin.acceptable_tokens[i].amount = token_to_update.amount;
            break;
        }
    }

    commission_admin.update_token_nonce += 1;
    commission_admin.serialize(&mut *commission_admin_info.data.borrow_mut())?;

    Ok(())
}


pub fn process_withdraw<'a>(
    program_id: &'a Pubkey,
    accounts: &'a [AccountInfo<'a>],
    signature: [u8; SECP256K1_PUBLIC_KEY_LENGTH],
    recovery_id: u8,
    path: Vec<[u8; 32]>,
    token: CommissionTokenArg,
    withdraw_amount: u64,
) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();

    let commission_admin_info = next_account_info(account_info_iter)?;
    let bridge_admin_info = next_account_info(account_info_iter)?;
    let receiver_info = next_account_info(account_info_iter)?;
    let system_program = next_account_info(account_info_iter)?;
    let rent_info = next_account_info(account_info_iter)?;

    let commission_key = Pubkey::create_program_address(&[lib::COMMISSION_ADMIN_PDA_SEED.as_bytes(), bridge_admin_info.key.as_ref()], &program_id)?;
    if commission_key != *commission_admin_info.key {
        return Err(LibError::WrongAdmin.into());
    }

    let mut commission_admin: CommissionAdmin = BorshDeserialize::deserialize(&mut commission_admin_info.data.borrow_mut().as_ref())?;
    if !commission_admin.is_initialized {
        return Err(LibError::NotInitialized.into());
    }

    let bridge_admin: BridgeAdmin = BorshDeserialize::deserialize(&mut bridge_admin_info.data.borrow_mut().as_ref())?;
    if !bridge_admin.is_initialized {
        return Err(LibError::NotInitialized.into());
    }

    let content = Content::new(
        commission_admin.withdraw_token_nonce,
        Some(*receiver_info.key),
        *program_id,
        OperationType::WithdrawToken,
        CommissionToken::from(&token),
    );
    let root = get_merkle_root(content.hash(), &path)?;
    verify_ecdsa_signature(root.as_slice(), signature.as_slice(), recovery_id, bridge_admin.public_key)?;

    match token.token.into() {
        lib::CommissionToken::Native => {
            call_transfer_native(
                commission_admin_info,
                receiver_info,
                withdraw_amount,
                &[lib::COMMISSION_ADMIN_PDA_SEED.as_bytes(), bridge_admin_info.key.as_ref()],
            )?;
        }
        lib::CommissionToken::FT(mint) => {
            let token_program = next_account_info(account_info_iter)?;
            let receiver_associated_info = next_account_info(account_info_iter)?;
            let commission_associated_info = next_account_info(account_info_iter)?;

            if *commission_associated_info.key !=
                get_associated_token_address(&commission_key, &mint) {
                return Err(LibError::WrongTokenAccount.into());
            }

            if *receiver_associated_info.key !=
                get_associated_token_address(receiver_info.key, &mint) {
                return Err(LibError::WrongTokenAccount.into());
            }

            if receiver_associated_info.data.borrow().as_ref().len() == 0 {
                msg!("Creating receiver associated account");
                let mint_info = next_account_info(account_info_iter)?;
                lib::call_create_associated_account(
                    receiver_info,
                    receiver_info,
                    mint_info,
                    receiver_associated_info,
                    rent_info,
                    system_program,
                    token_program,
                )?;
            }

            call_transfer_ft(
                commission_associated_info,
                receiver_associated_info,
                commission_admin_info,
                withdraw_amount,
                &[lib::COMMISSION_ADMIN_PDA_SEED.as_bytes(), bridge_admin_info.key.as_ref()],
            )?;
        }
        lib::CommissionToken::NFT(mint) => {
            return Err(LibError::NotSupported.into());
        }
    }

    commission_admin.withdraw_token_nonce += 1;
    commission_admin.serialize(&mut *commission_admin_info.data.borrow_mut())?;

    Ok(())
}

fn call_transfer_native<'a>(
    from: &AccountInfo<'a>,
    to: &AccountInfo<'a>,
    amount: u64,
    seeds: &[&[u8]],
) -> ProgramResult {
    let transfer_tokens_instruction = solana_program::system_instruction::transfer(
        from.key,
        to.key,
        amount,
    );

    let accounts = [
        from.clone(),
        to.clone(),
    ];

    if seeds.len() > 0 {
        return invoke_signed(&transfer_tokens_instruction, &accounts, &[seeds]);
    }

    invoke(&transfer_tokens_instruction, &accounts)
}

fn call_transfer_ft<'a>(
    from: &AccountInfo<'a>,
    to: &AccountInfo<'a>,
    authority: &AccountInfo<'a>,
    amount: u64,
    seeds: &[&[u8]],
) -> ProgramResult {
    let transfer_tokens_instruction = transfer(
        &spl_token::id(),
        from.key,
        to.key,
        authority.key,
        &[],
        amount,
    )?;

    let accounts = [
        from.clone(),
        to.clone(),
        authority.clone(),
    ];

    if seeds.len() > 0 {
        return invoke_signed(&transfer_tokens_instruction, &accounts, &[seeds]);
    }

    invoke(&transfer_tokens_instruction, &accounts)
}

fn check_token_is_acceptable(list: Vec<CommissionToken>, token: lib::CommissionToken) -> Result<CommissionToken, LibError> {
    for l in list {
        if l.token == token {
            return Ok(l);
        }
    }

    return Err(LibError::NotAcceptable.into());
}