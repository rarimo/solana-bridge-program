use solana_program::{
    account_info::{AccountInfo, next_account_info},
    entrypoint::ProgramResult, msg,
    program::{invoke, invoke_signed}, pubkey::Pubkey, system_instruction,
    sysvar::{rent::Rent, Sysvar},
};
use crate::state::{CommissionToken, CommissionAdmin};
use borsh::{
    BorshDeserialize, BorshSerialize,
};
use spl_token::instruction::transfer;
use spl_associated_token_account::get_associated_token_address;
use spl_associated_token_account::instruction::create_associated_token_account;
use solana_program::secp256k1_recover::SECP256K1_PUBLIC_KEY_LENGTH;
use lib::merkle::{ContentNode, get_merkle_root};
use crate::merkle::{CommissionTokenData, OperationType};
use lib::ecdsa::verify_ecdsa_signature;
use lib::instructions::commission::{CommissionInstruction, CommissionTokenArg, MAX_ADMIN_SIZE};
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
            msg!("Instruction: Create Bridge Admin");
            process_init_admin(program_id, accounts, args.acceptable_tokens)
        }
        CommissionInstruction::ChargeCommission(args) => {
            msg!("Instruction: Charge commission");
            process_charge_commission(program_id, accounts, args.token)
        }
        CommissionInstruction::AddFeeToken(args) => {
            msg!("Instruction: Add fee token");
            process_add_token(program_id, accounts, args.origin, args.signature, args.recovery_id, args.path, args.token)
        }
        CommissionInstruction::RemoveFeeToken(args) => {
            msg!("Instruction: Remove fee token");
            process_remove_token(program_id, accounts, args.origin, args.signature, args.recovery_id, args.path, args.token)
        }
        CommissionInstruction::UpdateFeeToken(args) => {
            msg!("Instruction: Update fee token");
            process_update_token(program_id, accounts, args.origin, args.signature, args.recovery_id, args.path, args.token)
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

    call_create_account(
        fee_payer_info,
        bridge_admin_info,
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
    let token_program = next_account_info(account_info_iter)?;

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
            call_charge_in_native(owner_info, commission_admin_info, commission_token.amount)?;
        }
        lib::CommissionToken::FT(mint) => {
            let owner_associated_info = next_account_info(account_info_iter)?;
            let commission_associated_info = next_account_info(account_info_iter)?;

            if *commission_associated_info.key !=
                get_associated_token_address(&commission_key, &mint) {
                return Err(LibError::WrongTokenAccount.into());
            }

            if commission_associated_info.data.borrow().as_ref().len() == 0 {
                msg!("Creating bridge admin associated account");
                let mint_info = next_account_info(account_info_iter)?;
                call_create_associated_account(
                    owner_info,
                    commission_admin_info,
                    mint_info,
                    commission_associated_info,
                    rent_info,
                    system_program,
                    token_program,
                )?;
            }

            call_charge_in_ft(
                owner_associated_info,
                commission_associated_info,
                owner_info,
                commission_token.amount,
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
    origin: [u8; 32],
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

    let content = ContentNode::new(
        origin,
        commission_admin_info.key.to_bytes(),
        program_id.to_bytes(),
        Box::new(
            CommissionTokenData::new_data(OperationType::AddToken, CommissionToken::from(&token))
        ),
    );
    let root = get_merkle_root(content, &path)?;
    verify_ecdsa_signature(root.as_slice(), signature.as_slice(), recovery_id, bridge_admin.public_key)?;

    commission_admin.acceptable_tokens.push(CommissionToken::from(&token));
    commission_admin.serialize(&mut *commission_admin_info.data.borrow_mut())?;
    Ok(())
}

pub fn process_remove_token<'a>(
    program_id: &'a Pubkey,
    accounts: &'a [AccountInfo<'a>],
    origin: [u8; 32],
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

    let content = ContentNode::new(
        origin,
        commission_admin_info.key.to_bytes(),
        program_id.to_bytes(),
        Box::new(
            CommissionTokenData::new_data(OperationType::RemoveToken, CommissionToken::from(&token))
        ),
    );
    let root = get_merkle_root(content, &path)?;
    verify_ecdsa_signature(root.as_slice(), signature.as_slice(), recovery_id, bridge_admin.public_key)?;

    let token_to_remove = CommissionToken::from(&token);
    for i in 0..commission_admin.acceptable_tokens.len() {
        if commission_admin.acceptable_tokens[i].eq(&token_to_remove) {
            commission_admin.acceptable_tokens.remove(i);
            break;
        }
    }

    commission_admin.serialize(&mut *commission_admin_info.data.borrow_mut())?;
    Ok(())
}

pub fn process_update_token<'a>(
    program_id: &'a Pubkey,
    accounts: &'a [AccountInfo<'a>],
    origin: [u8; 32],
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

    let content = ContentNode::new(
        origin,
        commission_admin_info.key.to_bytes(),
        program_id.to_bytes(),
        Box::new(
            CommissionTokenData::new_data(OperationType::UpdateToken, CommissionToken::from(&token))
        ),
    );
    let root = get_merkle_root(content, &path)?;
    verify_ecdsa_signature(root.as_slice(), signature.as_slice(), recovery_id, bridge_admin.public_key)?;

    let token_to_update = CommissionToken::from(&token);
    for i in 0..commission_admin.acceptable_tokens.len() {
        if commission_admin.acceptable_tokens[i].token.eq(&token_to_update.token) {
            commission_admin.acceptable_tokens[i].amount = token_to_update.amount;
            break;
        }
    }

    commission_admin.serialize(&mut *commission_admin_info.data.borrow_mut())?;
    Ok(())
}

fn call_charge_in_native<'a>(
    from: &AccountInfo<'a>,
    to: &AccountInfo<'a>,
    amount: u64,
) -> ProgramResult {
    let transfer_tokens_instruction = solana_program::system_instruction::transfer(
        from.key,
        to.key,
        amount,
    );

    msg!("Charging commission in native token");
    invoke(
        &transfer_tokens_instruction,
        &[
            from.clone(),
            to.clone(),
        ],
    )
}

fn call_charge_in_ft<'a>(
    from: &AccountInfo<'a>,
    to: &AccountInfo<'a>,
    authority: &AccountInfo<'a>,
    amount: u64,
) -> ProgramResult {
    let transfer_tokens_instruction = transfer(
        &spl_token::id(),
        from.key,
        to.key,
        authority.key,
        &[],
        amount,
    )?;

    invoke(
        &transfer_tokens_instruction,
        &[
            from.clone(),
            to.clone(),
            authority.clone(),
        ],
    )
}

fn call_create_associated_account<'a>(
    payer: &AccountInfo<'a>,
    wallet: &AccountInfo<'a>,
    mint: &AccountInfo<'a>,
    account: &AccountInfo<'a>,
    rent_info: &AccountInfo<'a>,
    system_program: &AccountInfo<'a>,
    spl_token: &AccountInfo<'a>,
) -> ProgramResult {
    invoke(
        &create_associated_token_account(
            payer.key,
            wallet.key,
            mint.key,
            spl_token.key,
        ),
        &[
            payer.clone(),
            account.clone(),
            wallet.clone(),
            mint.clone(),
            system_program.clone(),
            spl_token.clone(),
            rent_info.clone()
        ],
    )
}

fn call_create_account<'a>(
    payer: &AccountInfo<'a>,
    account: &AccountInfo<'a>,
    rent_info: &AccountInfo<'a>,
    system_program: &AccountInfo<'a>,
    space: usize,
    owner: &Pubkey,
    seeds: &[&[u8]],
) -> ProgramResult {
    let rent = Rent::from_account_info(rent_info)?;

    let instruction = system_instruction::create_account(
        payer.key,
        account.key,
        rent.minimum_balance(space),
        space as u64,
        owner,
    );

    let accounts = [
        payer.clone(),
        account.clone(),
        system_program.clone(),
    ];

    if seeds.len() > 0 {
        invoke_signed(&instruction, &accounts, &[seeds])
    } else {
        invoke(&instruction, &accounts)
    }
}

fn check_token_is_acceptable(list: Vec<CommissionToken>, token: lib::CommissionToken) -> Result<CommissionToken, LibError> {
    for l in list {
        if l.token == token {
            return Ok(l);
        }
    }

    return Err(LibError::NotAcceptable.into());
}