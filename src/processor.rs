use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    msg,
    program::{invoke, invoke_signed},
    program_error::ProgramError,
    pubkey::Pubkey,
    sysvar::{rent::Rent, Sysvar},
    hash,
};
use spl_token::{
    solana_program::program_pack::Pack,
    instruction::{transfer, initialize_mint, initialize_account, mint_to, set_authority},
    state::{Account, Mint},
};
use spl_associated_token_account::{
    get_associated_token_address,
    create_associated_token_account,
};
use mpl_token_metadata::{
    pda::find_metadata_account,
    state::Data,
    instruction::create_metadata_accounts,
};
use borsh::{
    BorshDeserialize, BorshSerialize,
};
use crate::{
    instruction::BridgeInstruction,
    state::{BridgeAdmin, BRIDGE_ADMIN_SIZE},
    error::BridgeError,
    state::{DEPOSIT_SIZE, Deposit, WITHDRAW_SIZE, Withdraw},
};
use spl_token::instruction::AuthorityType;

pub fn process_instruction<'a>(
    program_id: &'a Pubkey,
    accounts: &'a [AccountInfo<'a>],
    input: &[u8],
) -> ProgramResult {
    let instruction = BridgeInstruction::try_from_slice(input)?;
    match instruction {
        BridgeInstruction::InitializeAdmin(args) => {
            msg!("Instruction: Create Bridge Admin");
            process_init_admin(program_id, accounts, args.seeds, args.admin)
        }
        BridgeInstruction::TransferOwnership(args) => {
            msg!("Instruction: Transfer Bridge Admin ownership");
            process_transfer_ownership(program_id, accounts, args.seeds, args.new_admin)
        }
        BridgeInstruction::DepositMetaplex(args) => {
            msg!("Instruction: Deposit token");
            process_deposit_metaplex(program_id, accounts, args.seeds, args.network_to, args.receiver_address, args.nonce)
        }
        BridgeInstruction::WithdrawMetaplex(args) => {
            msg!("Instruction: Withdraw token");
            process_withdraw_metaplex(program_id, accounts, args.seeds, args.deposit_tx, args.network_from, args.sender_address, args.data)
        }
    }
}

pub fn process_init_admin<'a>(
    program_id: &'a Pubkey,
    accounts: &'a [AccountInfo<'a>],
    seeds: [u8; 32],
    admin: Pubkey,
) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();
    let bridge_admin_account_info = next_account_info(account_info_iter)?;
    let rent_info = next_account_info(account_info_iter)?;

    let mut bridge_admin: BridgeAdmin = BorshDeserialize::deserialize(&mut bridge_admin_account_info.data.borrow_mut().as_ref())?;
    if bridge_admin.is_initialized {
        return Err(BridgeError::AlreadyInUse.into());
    }

    let bridge_admin_key = Pubkey::create_program_address(&[&seeds], &program_id).unwrap();
    if bridge_admin_key != *bridge_admin_account_info.key {
        return Err(BridgeError::WrongSeeds.into());
    }

    if !bridge_admin_account_info.data_len() != BRIDGE_ADMIN_SIZE {
        return Err(BridgeError::WrongDataLen.into());
    }

    let rent = Rent::from_account_info(rent_info)?;
    if !rent.is_exempt(bridge_admin_account_info.lamports(), BRIDGE_ADMIN_SIZE) {
        return Err(BridgeError::NotRentExempt.into());
    }

    bridge_admin.admin = admin;
    bridge_admin.is_initialized = true;
    bridge_admin.serialize(&mut *bridge_admin_account_info.data.borrow_mut())?;
    Ok(())
}

pub fn process_transfer_ownership<'a>(
    program_id: &'a Pubkey,
    accounts: &'a [AccountInfo<'a>],
    seeds: [u8; 32],
    new_admin: Pubkey,
) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();
    let bridge_admin_account_info = next_account_info(account_info_iter)?;
    let current_admin_account_info = next_account_info(account_info_iter)?;

    let bridge_admin_key = Pubkey::create_program_address(&[&seeds], &program_id).unwrap();
    if bridge_admin_key != *bridge_admin_account_info.key {
        return Err(BridgeError::WrongSeeds.into());
    }

    let mut bridge_admin: BridgeAdmin = BorshDeserialize::deserialize(&mut bridge_admin_account_info.data.borrow_mut().as_ref())?;
    if !bridge_admin.is_initialized {
        return Err(BridgeError::NotInitialized.into());
    }

    if *current_admin_account_info.key != bridge_admin.admin {
        return Err(BridgeError::WrongAdmin.into());
    }

    if !current_admin_account_info.is_signer {
        return Err(BridgeError::UnsignedAdmin.into());
    }

    bridge_admin.admin = new_admin;
    bridge_admin.serialize(&mut *bridge_admin_account_info.data.borrow_mut())?;
    Ok(())
}

pub fn process_deposit_metaplex<'a>(
    program_id: &'a Pubkey,
    accounts: &'a [AccountInfo<'a>],
    seeds: [u8; 32],
    network: String,
    receiver: String,
    nonce: [u8; 32],
) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();
    let bridge_admin_account_info = next_account_info(account_info_iter)?;
    let mint_account_info = next_account_info(account_info_iter)?;
    let owner_associated_account_info = next_account_info(account_info_iter)?;
    let program_token_account_info = next_account_info(account_info_iter)?;
    let deposit_account_info = next_account_info(account_info_iter)?;
    let owner_account_info = next_account_info(account_info_iter)?;
    let token_program = next_account_info(account_info_iter)?;
    let rent_info = next_account_info(account_info_iter)?;

    let bridge_admin_key = Pubkey::create_program_address(&[&seeds], &program_id).unwrap();
    if *bridge_admin_account_info.key != bridge_admin_key {
        return Err(BridgeError::WrongSeeds.into());
    }

    let mut bridge_admin: BridgeAdmin = BorshDeserialize::deserialize(&mut bridge_admin_account_info.data.borrow_mut().as_ref())?;
    if !bridge_admin.is_initialized {
        return Err(BridgeError::NotInitialized.into());
    }

    if *program_token_account_info.key !=
        get_associated_token_address(&bridge_admin_key, mint_account_info.key) {
        return Err(BridgeError::WrongTokenAccount.into());
    }

    if *owner_associated_account_info.key !=
        get_associated_token_address(&owner_account_info.key, mint_account_info.key) {
        return Err(BridgeError::WrongTokenAccount.into());
    }

    let transfer_tokens_instruction = transfer(
        token_program.key,
        owner_associated_account_info.key,
        program_token_account_info.key,
        owner_account_info.key,
        &[],
        1,
    )?;

    invoke(
        &transfer_tokens_instruction,
        &[
            owner_associated_account_info.clone(),
            program_token_account_info.clone(),
            token_program.clone(),
            owner_account_info.clone(),
        ],
    )?;

    let deposit_key = Pubkey::create_program_address(&[&nonce], &bridge_admin_key).unwrap();
    if deposit_key != *deposit_account_info.key {
        return Err(BridgeError::WrongNonce.into());
    }

    let mut deposit: Deposit = BorshDeserialize::deserialize(&mut deposit_account_info.data.borrow_mut().as_ref())?;
    if deposit.is_initialized {
        return Err(BridgeError::AlreadyInUse.into());
    }

    if !deposit_account_info.data_len() != DEPOSIT_SIZE {
        return Err(BridgeError::WrongDataLen.into());
    }

    let rent = Rent::from_account_info(rent_info)?;
    if !rent.is_exempt(deposit_account_info.lamports(), DEPOSIT_SIZE) {
        return Err(BridgeError::NotRentExempt.into());
    }

    deposit.is_initialized = true;
    deposit.network = network;
    deposit.receiver_address = receiver;
    deposit.serialize(&mut *deposit_account_info.data.borrow_mut())?;
    Ok(())
}

pub fn process_withdraw_metaplex<'a>(
    program_id: &'a Pubkey,
    accounts: &'a [AccountInfo<'a>],
    seeds: [u8; 32],
    tx: String,
    network: String,
    sender: String,
    data: Data,
) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();
    let bridge_admin_account_info = next_account_info(account_info_iter)?;
    let mint_account_info = next_account_info(account_info_iter)?;
    let token_metadata_account_info = next_account_info(account_info_iter)?;
    let owner_associated_account_info = next_account_info(account_info_iter)?;
    let owner_account_info = next_account_info(account_info_iter)?;
    let program_token_account_info = next_account_info(account_info_iter)?;
    let withdraw_account_info = next_account_info(account_info_iter)?;
    let admin_account_info = next_account_info(account_info_iter)?;
    let token_program = next_account_info(account_info_iter)?;
    let metadata_program = next_account_info(account_info_iter)?;
    let rent_info = next_account_info(account_info_iter)?;

    let bridge_admin_key = Pubkey::create_program_address(&[&seeds], &program_id).unwrap();
    if *bridge_admin_account_info.key != bridge_admin_key {
        return Err(BridgeError::WrongSeeds.into());
    }

    let mut bridge_admin: BridgeAdmin = BorshDeserialize::deserialize(&mut bridge_admin_account_info.data.borrow_mut().as_ref())?;
    if !bridge_admin.is_initialized {
        return Err(BridgeError::NotInitialized.into());
    }

    if *admin_account_info.key != bridge_admin.admin {
        return Err(BridgeError::WrongAdmin.into());
    }

    if !admin_account_info.is_signer {
        return Err(BridgeError::UnsignedAdmin.into());
    }

    if *program_token_account_info.key !=
        get_associated_token_address(&bridge_admin_key, mint_account_info.key) {
        return Err(BridgeError::WrongTokenAccount.into());
    }

    if *owner_associated_account_info.key !=
        get_associated_token_address(owner_account_info.key, mint_account_info.key) {
        return Err(BridgeError::WrongTokenAccount.into());
    }

    let (metadata_key, _) = find_metadata_account(mint_account_info.key);
    if *token_metadata_account_info.key != metadata_key {
        return Err(BridgeError::WrongMetadataAccount.into());
    }


    let mint = Mint::unpack_from_slice(mint_account_info.data.borrow().as_ref())?;
    if !mint.is_initialized {
        mint_metaplex(
            bridge_admin_account_info,
            mint_account_info,
            token_metadata_account_info,
            owner_associated_account_info,
            owner_account_info,
            token_program,
            metadata_program,
            seeds,
            data,
        )?;
    } else {
        let transfer_tokens_instruction = transfer(
            &token_program.key,
            &program_token_account_info.key,
            owner_associated_account_info.key,
            &bridge_admin_key,
            &[],
            1,
        )?;

        invoke_signed(
            &transfer_tokens_instruction,
            &[
                token_program.clone(),
                program_token_account_info.clone(),
                owner_associated_account_info.clone(),
                bridge_admin_account_info.clone(),
            ],
            &[&[&seeds]],
        )?;
    }

    let nonce = hash::hash(tx.as_bytes()).to_bytes();
    let withdraw_key = Pubkey::create_program_address(&[&nonce], &bridge_admin_key).unwrap();
    if withdraw_key != *withdraw_account_info.key {
        return Err(BridgeError::WrongNonce.into());
    }

    let mut withdraw: Withdraw = BorshDeserialize::deserialize(&mut withdraw_account_info.data.borrow_mut().as_ref())?;
    if withdraw.is_initialized {
        return Err(BridgeError::AlreadyInUse.into());
    }

    if !withdraw_account_info.data_len() != WITHDRAW_SIZE {
        return Err(BridgeError::WrongDataLen.into());
    }

    let rent = Rent::from_account_info(rent_info)?;
    if !rent.is_exempt(withdraw_account_info.lamports(), WITHDRAW_SIZE) {
        return Err(BridgeError::NotRentExempt.into());
    }

    withdraw.is_initialized = true;
    withdraw.network = network;
    withdraw.sender_address = sender;
    withdraw.serialize(&mut *withdraw_account_info.data.borrow_mut())?;
    Ok(())
}

fn mint_metaplex(
    bridge_admin_account_info: &AccountInfo,
    mint_account_info: &AccountInfo,
    metadata_account_info: &AccountInfo,
    owner_associated_account_info: &AccountInfo,
    owner_account_info: &AccountInfo,
    token_program: &AccountInfo,
    metadata_program: &AccountInfo,
    seeds: [u8; 32],
    data: Data,
) -> ProgramResult {
    let init_mint_instruction = initialize_mint(
        token_program.key,
        mint_account_info.key,
        bridge_admin_account_info.key,
        None,
        0,
    )?;

    invoke_signed(
        &init_mint_instruction,
        &[
            token_program.clone(),
            mint_account_info.clone(),
            bridge_admin_account_info.clone(),
        ],
        &[&[&seeds]],
    )?;

    let init_account_instruction = create_associated_token_account(
        owner_account_info.key,
        owner_account_info.key,
        mint_account_info.key,
    )?;

    invoke(
        &init_account_instruction,
        &[
            owner_account_info.clone(),
            bridge_admin_account_info.clone(),
        ],
    )?;

    let mint_to_instruction = mint_to(
        token_program.key,
        mint_account_info.key,
        owner_associated_account_info.key,
        bridge_admin_account_info.key,
        &[&[]],
        1,
    )?;

    invoke_signed(
        &mint_to_instruction,
        &[
            token_program.clone(),
            mint_account_info.clone(),
            owner_associated_account_info.clone(),
            bridge_admin_account_info.clone(),
        ],
        &[&[&seeds]],
    )?;

    let init_metadata_instruction = create_metadata_accounts(
        metadata_program.key.clone(),
        metadata_account_info.key.clone(),
        mint_account_info.key.clone(),
        bridge_admin_account_info.key.clone(),
        owner_account_info.key.clone(),
        bridge_admin_account_info.key.clone(),
        data.name,
        data.symbol,
        data.uri,
        data.creators,
        data.seller_fee_basis_points,
        true,
        false,
    )?;

    invoke_signed(
        &init_metadata_instruction,
        &[
            metadata_program.clone(),
            metadata_account_info.clone(),
            mint_account_info.clone(),
            bridge_admin_account_info.clone(),
            owner_account_info.clone(),
        ],
        &[&[&seeds]],
    )?;


    let block_authority_instruction = set_authority(
        token_program.key,
        mint_account_info.key,
        None,
        AuthorityType::MintTokens,
        bridge_admin_account_info.key,
        &[&[]],
    )?;

    invoke_signed(
        &block_authority_instruction,
        &[
            token_program.clone(),
            mint_account_info.clone(),
            bridge_admin_account_info.clone(),
        ],
        &[&[&seeds]],
    )?;

    Ok(())
}