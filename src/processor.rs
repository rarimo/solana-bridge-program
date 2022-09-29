use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult, msg, program::{invoke, invoke_signed},
    pubkey::Pubkey, sysvar::{rent::Rent, Sysvar}, hash, system_instruction,
    secp256k1_recover::{SECP256K1_PUBLIC_KEY_LENGTH, SECP256K1_SIGNATURE_LENGTH},
};
use spl_token::{
    instruction::{transfer, initialize_mint, mint_to},
    solana_program::program_pack::Pack,
    state::{Mint},
};
use spl_associated_token_account::{get_associated_token_address, create_associated_token_account};
use mpl_token_metadata::{
    state::{DataV2, TokenStandard},
    instruction::{create_metadata_accounts_v2, verify_collection, create_master_edition_v3},
};
use borsh::{
    BorshDeserialize, BorshSerialize,
};
use crate::{
    instruction::BridgeInstruction,
    state::{BridgeAdmin, BRIDGE_ADMIN_SIZE, TokenType::{NFT, FT, Native}},
    error::BridgeError,
    state::{DEPOSIT_SIZE, Deposit, WITHDRAW_SIZE, Withdraw},
    util::{verify_ecdsa_signature, get_merkle_root},
    merkle::ContentNode,
};
use crate::merkle::{TransferOperation, Operation};

pub fn process_instruction<'a>(
    program_id: &'a Pubkey,
    accounts: &'a [AccountInfo<'a>],
    input: &[u8],
) -> ProgramResult {
    let instruction = BridgeInstruction::try_from_slice(input)?;
    match instruction {
        BridgeInstruction::InitializeAdmin(args) => {
            msg!("Instruction: Create Bridge Admin");
            process_init_admin(program_id, accounts, args.seeds, args.public_key)
        }
        BridgeInstruction::TransferOwnership(args) => {
            msg!("Instruction: Transfer Bridge Admin ownership");
            process_transfer_ownership(program_id, accounts, args.seeds, args.new_public_key, args.signature, args.recovery_id)
        }
        BridgeInstruction::DepositNative(args) => {
            msg!("Instruction: Deposit SOL");
            args.validate()?;
            process_deposit_native(program_id, accounts, args.seeds, args.network_to, args.receiver_address, args.amount, args.nonce)
        }
        BridgeInstruction::DepositFT(args) => {
            msg!("Instruction: Deposit FT");
            args.validate()?;
            process_deposit_ft(program_id, accounts, args.seeds, args.network_to, args.receiver_address, args.amount, args.nonce)
        }
        BridgeInstruction::DepositNFT(args) => {
            msg!("Instruction: Deposit NFT");
            args.validate()?;
            process_deposit_nft(program_id, accounts, args.seeds, args.network_to, args.receiver_address, args.nonce)
        }

        BridgeInstruction::WithdrawNative(args) => {
            msg!("Instruction: Withdraw SOL");
            args.validate()?;
            process_withdraw_native(program_id, accounts, args.seeds, args.signature, args.recovery_id, args.path, args.origin, args.amount)
        }

        BridgeInstruction::WithdrawFT(args) => {
            msg!("Instruction: Withdraw FT");
            args.validate()?;
            process_withdraw_ft(program_id, accounts, args.seeds, args.signature, args.recovery_id, args.path, args.origin, args.amount)
        }

        BridgeInstruction::WithdrawNFT(args) => {
            msg!("Instruction: Withdraw NFT");
            args.validate()?;
            process_withdraw_nft(program_id, accounts, args.seeds, args.signature, args.recovery_id, args.path, args.origin)
        }

        BridgeInstruction::MintFT(args) => {
            msg!("Instruction: Mint FT");
            args.validate()?;
            process_mint_ft(program_id, accounts, args.seeds, args.data, args.decimals, args.amount)
        }

        BridgeInstruction::MintNFT(args) => {
            msg!("Instruction: Mint NFT");
            args.validate()?;
            process_mint_nft(program_id, accounts, args.seeds, args.data, args.verify)
        }
    }
}

pub fn process_init_admin<'a>(
    program_id: &'a Pubkey,
    accounts: &'a [AccountInfo<'a>],
    seeds: [u8; 32],
    public_key: [u8; SECP256K1_PUBLIC_KEY_LENGTH],
) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();

    let bridge_admin_info = next_account_info(account_info_iter)?;
    let fee_payer_info = next_account_info(account_info_iter)?;
    let system_program = next_account_info(account_info_iter)?;
    let rent_info = next_account_info(account_info_iter)?;

    let bridge_key = Pubkey::create_program_address(&[&seeds], &program_id)?;
    if bridge_key != *bridge_admin_info.key {
        return Err(BridgeError::WrongSeeds.into());
    }

    call_create_account(
        fee_payer_info,
        bridge_admin_info,
        rent_info,
        system_program,
        BRIDGE_ADMIN_SIZE,
        program_id,
        &[&seeds],
    )?;

    let mut bridge_admin: BridgeAdmin = BorshDeserialize::deserialize(&mut bridge_admin_info.data.borrow_mut().as_ref())?;
    if bridge_admin.is_initialized {
        return Err(BridgeError::AlreadyInUse.into());
    }

    bridge_admin.public_key = public_key;
    bridge_admin.is_initialized = true;
    bridge_admin.serialize(&mut *bridge_admin_info.data.borrow_mut())?;
    Ok(())
}

pub fn process_transfer_ownership<'a>(
    program_id: &'a Pubkey,
    accounts: &'a [AccountInfo<'a>],
    seeds: [u8; 32],
    new_public_key: [u8; SECP256K1_PUBLIC_KEY_LENGTH],
    signature: [u8; SECP256K1_SIGNATURE_LENGTH],
    recovery_id: u8,
) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();
    let bridge_admin_info = next_account_info(account_info_iter)?;

    let bridge_admin_key = Pubkey::create_program_address(&[&seeds], &program_id)?;
    if bridge_admin_key != *bridge_admin_info.key {
        return Err(BridgeError::WrongSeeds.into());
    }

    let mut bridge_admin: BridgeAdmin = BorshDeserialize::deserialize(&mut bridge_admin_info.data.borrow_mut().as_ref())?;
    if !bridge_admin.is_initialized {
        return Err(BridgeError::NotInitialized.into());
    }


    verify_ecdsa_signature(new_public_key.as_slice(), signature.as_slice(), recovery_id, bridge_admin.public_key)?;

    bridge_admin.public_key = new_public_key;
    bridge_admin.serialize(&mut *bridge_admin_info.data.borrow_mut())?;
    Ok(())
}


pub fn process_deposit_native<'a>(
    program_id: &'a Pubkey,
    accounts: &'a [AccountInfo<'a>],
    seeds: [u8; 32],
    network: String,
    receiver: String,
    amount: u64,
    nonce: [u8; 32],
) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();

    let bridge_admin_info = next_account_info(account_info_iter)?;
    let deposit_info = next_account_info(account_info_iter)?;
    let owner_info = next_account_info(account_info_iter)?;
    let system_program = next_account_info(account_info_iter)?;
    let rent_info = next_account_info(account_info_iter)?;

    let bridge_admin_key = Pubkey::create_program_address(&[&seeds], &program_id)?;
    if *bridge_admin_info.key != bridge_admin_key {
        return Err(BridgeError::WrongSeeds.into());
    }

    let bridge_admin: BridgeAdmin = BorshDeserialize::deserialize(&mut bridge_admin_info.data.borrow_mut().as_ref())?;
    if !bridge_admin.is_initialized {
        return Err(BridgeError::NotInitialized.into());
    }

    let (deposit_key, bump_seed) = Pubkey::find_program_address(&[&nonce], program_id);
    if deposit_key != *deposit_info.key {
        return Err(BridgeError::WrongNonce.into());
    }

    let transfer_tokens_instruction = solana_program::system_instruction::transfer(
        owner_info.key,
        bridge_admin_info.key,
        amount,
    );

    msg!("Transferring token");
    invoke(
        &transfer_tokens_instruction,
        &[
            owner_info.clone(),
            bridge_admin_info.clone(),
        ],
    )?;

    msg!("Creating deposit account");
    call_create_account(
        owner_info,
        deposit_info,
        rent_info,
        system_program,
        DEPOSIT_SIZE,
        program_id,
        &[&nonce, &[bump_seed]],
    )?;

    let mut deposit: Deposit = BorshDeserialize::deserialize(&mut deposit_info.data.borrow_mut().as_ref())?;
    if deposit.is_initialized {
        return Err(BridgeError::AlreadyInUse.into());
    }

    deposit.is_initialized = true;
    deposit.token_type = Native;
    deposit.amount = amount;
    deposit.mint = Option::None;
    deposit.network = network;
    deposit.receiver_address = receiver;
    deposit.serialize(&mut *deposit_info.data.borrow_mut())?;
    msg!("Deposit account created");
    Ok(())
}

pub fn process_deposit_ft<'a>(
    program_id: &'a Pubkey,
    accounts: &'a [AccountInfo<'a>],
    seeds: [u8; 32],
    network: String,
    receiver: String,
    amount: u64,
    nonce: [u8; 32],
) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();

    let bridge_admin_info = next_account_info(account_info_iter)?;
    let mint_info = next_account_info(account_info_iter)?;
    let owner_associated_info = next_account_info(account_info_iter)?;
    let bridge_associated_info = next_account_info(account_info_iter)?;
    let deposit_info = next_account_info(account_info_iter)?;
    let owner_info = next_account_info(account_info_iter)?;

    let token_program = next_account_info(account_info_iter)?;
    let system_program = next_account_info(account_info_iter)?;
    let rent_info = next_account_info(account_info_iter)?;

    let bridge_admin_key = Pubkey::create_program_address(&[&seeds], &program_id)?;
    if *bridge_admin_info.key != bridge_admin_key {
        return Err(BridgeError::WrongSeeds.into());
    }

    let bridge_admin: BridgeAdmin = BorshDeserialize::deserialize(&mut bridge_admin_info.data.borrow_mut().as_ref())?;
    if !bridge_admin.is_initialized {
        return Err(BridgeError::NotInitialized.into());
    }

    let (deposit_key, bump_seed) = Pubkey::find_program_address(&[&nonce], program_id);
    if deposit_key != *deposit_info.key {
        return Err(BridgeError::WrongNonce.into());
    }

    if *bridge_associated_info.key !=
        get_associated_token_address(&bridge_admin_key, mint_info.key) {
        return Err(BridgeError::WrongTokenAccount.into());
    }

    if bridge_associated_info.data.borrow().as_ref().len() == 0 {
        msg!("Creating bridge admin associated account");
        call_create_associated_account(
            owner_info,
            bridge_admin_info,
            mint_info,
            bridge_associated_info,
            rent_info,
            system_program,
            token_program,
        )?;
    }

    let transfer_tokens_instruction = transfer(
        &spl_token::id(),
        owner_associated_info.key,
        bridge_associated_info.key,
        owner_info.key,
        &[],
        amount,
    )?;

    msg!("Transferring token");
    invoke(
        &transfer_tokens_instruction,
        &[
            owner_associated_info.clone(),
            bridge_associated_info.clone(),
            owner_info.clone(),
        ],
    )?;

    msg!("Creating deposit account");
    call_create_account(
        owner_info,
        deposit_info,
        rent_info,
        system_program,
        DEPOSIT_SIZE,
        program_id,
        &[&nonce, &[bump_seed]],
    )?;

    let mut deposit: Deposit = BorshDeserialize::deserialize(&mut deposit_info.data.borrow_mut().as_ref())?;
    if deposit.is_initialized {
        return Err(BridgeError::AlreadyInUse.into());
    }

    deposit.is_initialized = true;
    deposit.mint = Option::Some(mint_info.key.clone());
    deposit.token_type = FT;
    deposit.network = network;
    deposit.receiver_address = receiver;
    deposit.amount = amount;
    deposit.serialize(&mut *deposit_info.data.borrow_mut())?;
    msg!("Deposit account created");
    Ok(())
}

pub fn process_deposit_nft<'a>(
    program_id: &'a Pubkey,
    accounts: &'a [AccountInfo<'a>],
    seeds: [u8; 32],
    network: String,
    receiver: String,
    nonce: [u8; 32],
) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();

    let bridge_admin_info = next_account_info(account_info_iter)?;
    let mint_info = next_account_info(account_info_iter)?;
    let owner_associated_info = next_account_info(account_info_iter)?;
    let bridge_associated_info = next_account_info(account_info_iter)?;
    let deposit_info = next_account_info(account_info_iter)?;
    let owner_info = next_account_info(account_info_iter)?;

    let token_program = next_account_info(account_info_iter)?;
    let system_program = next_account_info(account_info_iter)?;
    let rent_info = next_account_info(account_info_iter)?;

    let bridge_admin_key = Pubkey::create_program_address(&[&seeds], &program_id)?;
    if *bridge_admin_info.key != bridge_admin_key {
        return Err(BridgeError::WrongSeeds.into());
    }

    let bridge_admin: BridgeAdmin = BorshDeserialize::deserialize(&mut bridge_admin_info.data.borrow_mut().as_ref())?;
    if !bridge_admin.is_initialized {
        return Err(BridgeError::NotInitialized.into());
    }

    if *bridge_associated_info.key !=
        get_associated_token_address(&bridge_admin_key, mint_info.key) {
        return Err(BridgeError::WrongTokenAccount.into());
    }

    let (deposit_key, bump_seed) = Pubkey::find_program_address(&[&nonce], program_id);
    if deposit_key != *deposit_info.key {
        return Err(BridgeError::WrongNonce.into());
    }

    if bridge_associated_info.data.borrow().as_ref().len() == 0 {
        msg!("Creating bridge admin associated account");
        call_create_associated_account(
            owner_info,
            bridge_admin_info,
            mint_info,
            bridge_associated_info,
            rent_info,
            system_program,
            token_program,
        )?;
    }

    let transfer_tokens_instruction = transfer(
        &spl_token::id(),
        owner_associated_info.key,
        bridge_associated_info.key,
        owner_info.key,
        &[],
        1,
    )?;

    msg!("Transferring token");
    invoke(
        &transfer_tokens_instruction,
        &[
            owner_associated_info.clone(),
            bridge_associated_info.clone(),
            owner_info.clone(),
        ],
    )?;

    msg!("Creating deposit account");
    call_create_account(
        owner_info,
        deposit_info,
        rent_info,
        system_program,
        DEPOSIT_SIZE,
        program_id,
        &[&nonce, &[bump_seed]],
    )?;

    let mut deposit: Deposit = BorshDeserialize::deserialize(&mut deposit_info.data.borrow_mut().as_ref())?;
    if deposit.is_initialized {
        return Err(BridgeError::AlreadyInUse.into());
    }

    deposit.is_initialized = true;
    deposit.amount = 1;
    deposit.token_type = NFT;
    deposit.mint = Option::Some(mint_info.key.clone());
    deposit.network = network;
    deposit.receiver_address = receiver;
    deposit.serialize(&mut *deposit_info.data.borrow_mut())?;
    msg!("Deposit account created");
    Ok(())
}

pub fn process_withdraw_native<'a>(
    program_id: &'a Pubkey,
    accounts: &'a [AccountInfo<'a>],
    seeds: [u8; 32],
    signature: [u8; SECP256K1_SIGNATURE_LENGTH],
    recovery_id: u8,
    path: Vec<[u8; 32]>,
    origin: Vec<u8>,
    amount: u64,
) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();

    let bridge_admin_info = next_account_info(account_info_iter)?;
    let owner_info = next_account_info(account_info_iter)?;
    let withdraw_info = next_account_info(account_info_iter)?;

    let system_program = next_account_info(account_info_iter)?;
    let rent_info = next_account_info(account_info_iter)?;

    let bridge_admin_key = Pubkey::create_program_address(&[&seeds], &program_id).unwrap();
    if *bridge_admin_info.key != bridge_admin_key {
        return Err(BridgeError::WrongSeeds.into());
    }

    let bridge_admin: BridgeAdmin = BorshDeserialize::deserialize(&mut bridge_admin_info.data.borrow_mut().as_ref())?;
    if !bridge_admin.is_initialized {
        return Err(BridgeError::NotInitialized.into());
    }

    let (withdraw_key, bump_seed) = Pubkey::find_program_address(&[origin.as_slice()], program_id);
    if withdraw_key != *withdraw_info.key {
        return Err(BridgeError::WrongNonce.into());
    }


    let content = ContentNode::new(
        origin.to_vec(),
        owner_info.key.to_bytes(),
        program_id.to_bytes(),
        TransferOperation::new_native_transfer(
            amount,
        ).get_operation(),
    );
    let root = get_merkle_root(content, &path)?;

    verify_ecdsa_signature(root.as_slice(), signature.as_slice(), recovery_id, bridge_admin.public_key)?;

    // TODO check rent
    if **bridge_admin_info.try_borrow_lamports()? < amount {
        return Err(BridgeError::WrongBalance.into());
    }

    // Need to do that before transferring SOls
    msg!("Creating withdraw account");
    call_create_account(
        owner_info,
        withdraw_info,
        rent_info,
        system_program,
        WITHDRAW_SIZE,
        program_id,
        &[origin.as_slice(), &[bump_seed]],
    )?;

    msg!("Transferring token");
    **bridge_admin_info.try_borrow_mut_lamports()? -= amount;
    **owner_info.try_borrow_mut_lamports()? += amount;

    msg!("Initializing withdraw account");

    let mut withdraw: Withdraw = BorshDeserialize::deserialize(&mut withdraw_info.data.borrow_mut().as_ref())?;
    if withdraw.is_initialized {
        return Err(BridgeError::AlreadyInUse.into());
    }

    withdraw.is_initialized = true;
    withdraw.token_type = Native;
    withdraw.origin = origin;
    withdraw.mint = Option::None;
    withdraw.amount = amount;
    withdraw.receiver_address = *owner_info.key;
    withdraw.serialize(&mut *withdraw_info.data.borrow_mut())?;
    msg!("Withdraw account created");
    Ok(())
}

pub fn process_withdraw_ft<'a>(
    program_id: &'a Pubkey,
    accounts: &'a [AccountInfo<'a>],
    seeds: [u8; 32],
    signature: [u8; SECP256K1_SIGNATURE_LENGTH],
    recovery_id: u8,
    path: Vec<[u8; 32]>,
    origin: Vec<u8>,
    amount: u64,
) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();

    let bridge_admin_info = next_account_info(account_info_iter)?;
    let mint_info = next_account_info(account_info_iter)?;
    let metadata_info = next_account_info(account_info_iter)?;
    let owner_info = next_account_info(account_info_iter)?;
    let owner_associated_info = next_account_info(account_info_iter)?;
    let bridge_token_info = next_account_info(account_info_iter)?;
    let withdraw_info = next_account_info(account_info_iter)?;

    let token_program = next_account_info(account_info_iter)?;
    let system_program = next_account_info(account_info_iter)?;
    let rent_info = next_account_info(account_info_iter)?;

    let bridge_admin_key = Pubkey::create_program_address(&[&seeds], &program_id).unwrap();
    if *bridge_admin_info.key != bridge_admin_key {
        return Err(BridgeError::WrongSeeds.into());
    }

    let bridge_admin: BridgeAdmin = BorshDeserialize::deserialize(&mut bridge_admin_info.data.borrow_mut().as_ref())?;
    if !bridge_admin.is_initialized {
        return Err(BridgeError::NotInitialized.into());
    }

    if *metadata_info.key != mpl_token_metadata::pda::find_metadata_account(mint_info.key).0 {
        return Err(BridgeError::WrongMetadataAccount.into());
    }

    if metadata_info.data.borrow().as_ref().len() == 0 {
        return Err(BridgeError::UninitializedMetadata.into());
    }

    let metadata: mpl_token_metadata::state::Metadata = BorshDeserialize::deserialize(&mut metadata_info.data.borrow_mut().as_ref())?;

    let (withdraw_key, bump_seed) = Pubkey::find_program_address(&[origin.as_slice()], program_id);
    if withdraw_key != *withdraw_info.key {
        return Err(BridgeError::WrongNonce.into());
    }

    let content = ContentNode::new(
        origin.to_vec(),
        owner_info.key.to_bytes(),
        program_id.to_bytes(),
        TransferOperation::new_ft_transfer(
            mint_info.key.to_bytes(),
            amount,
            metadata.data.name.trim_matches(char::from(0)).to_string(),
            metadata.data.symbol.trim_matches(char::from(0)).to_string(),
            metadata.data.uri.trim_matches(char::from(0)).to_string(),
        ).get_operation(),
    );

    let root = get_merkle_root(content, &path)?;

    verify_ecdsa_signature(root.as_slice(), signature.as_slice(), recovery_id, bridge_admin.public_key)?;

    if *bridge_token_info.key !=
        get_associated_token_address(&bridge_admin_key, mint_info.key) {
        return Err(BridgeError::WrongTokenAccount.into());
    }

    if *owner_associated_info.key !=
        get_associated_token_address(&owner_info.key, mint_info.key) {
        return Err(BridgeError::WrongTokenAccount.into());
    }

    if owner_associated_info.data.borrow().as_ref().len() == 0 {
        msg!("Deposit owner associated account");
        call_create_associated_account(
            owner_info,
            owner_info,
            mint_info,
            owner_associated_info,
            rent_info,
            system_program,
            token_program,
        )?;
    }

    let transfer_tokens_instruction = transfer(
        &spl_token::id(),
        bridge_token_info.key,
        owner_associated_info.key,
        &bridge_admin_key,
        &[],
        amount,
    )?;

    msg!("Transferring token");
    invoke_signed(
        &transfer_tokens_instruction,
        &[
            bridge_token_info.clone(),
            owner_associated_info.clone(),
            bridge_admin_info.clone(),
        ],
        &[&[&seeds]],
    )?;

    msg!("Creating withdraw account");
    call_create_account(
        owner_info,
        withdraw_info,
        rent_info,
        system_program,
        WITHDRAW_SIZE,
        program_id,
        &[origin.as_slice(), &[bump_seed]],
    )?;

    msg!("Initializing withdraw account");

    let mut withdraw: Withdraw = BorshDeserialize::deserialize(&mut withdraw_info.data.borrow_mut().as_ref())?;
    if withdraw.is_initialized {
        return Err(BridgeError::AlreadyInUse.into());
    }

    withdraw.is_initialized = true;
    withdraw.token_type = FT;
    withdraw.origin = origin;
    withdraw.mint = Option::Some(mint_info.key.clone());
    withdraw.amount = amount;
    withdraw.receiver_address = *owner_info.key;
    withdraw.serialize(&mut *withdraw_info.data.borrow_mut())?;
    msg!("Withdraw account created");
    Ok(())
}

pub fn process_withdraw_nft<'a>(
    program_id: &'a Pubkey,
    accounts: &'a [AccountInfo<'a>],
    seeds: [u8; 32],
    signature: [u8; SECP256K1_SIGNATURE_LENGTH],
    recovery_id: u8,
    path: Vec<[u8; 32]>,
    origin: Vec<u8>,
) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();

    let bridge_admin_info = next_account_info(account_info_iter)?;
    let mint_info = next_account_info(account_info_iter)?;
    let metadata_info = next_account_info(account_info_iter)?;
    let owner_info = next_account_info(account_info_iter)?;
    let owner_associated_info = next_account_info(account_info_iter)?;
    let bridge_token_info = next_account_info(account_info_iter)?;
    let withdraw_info = next_account_info(account_info_iter)?;

    let token_program = next_account_info(account_info_iter)?;
    let system_program = next_account_info(account_info_iter)?;
    let rent_info = next_account_info(account_info_iter)?;

    let bridge_admin_key = Pubkey::create_program_address(&[&seeds], &program_id).unwrap();
    if *bridge_admin_info.key != bridge_admin_key {
        return Err(BridgeError::WrongSeeds.into());
    }

    let bridge_admin: BridgeAdmin = BorshDeserialize::deserialize(&mut bridge_admin_info.data.borrow_mut().as_ref())?;
    if !bridge_admin.is_initialized {
        return Err(BridgeError::NotInitialized.into());
    }

    let (withdraw_key, bump_seed) = Pubkey::find_program_address(&[origin.as_slice()], program_id);
    if withdraw_key != *withdraw_info.key {
        return Err(BridgeError::WrongNonce.into());
    }

    if *metadata_info.key != mpl_token_metadata::pda::find_metadata_account(mint_info.key).0 {
        return Err(BridgeError::WrongMetadataAccount.into());
    }

    if metadata_info.data.borrow().as_ref().len() == 0 {
        return Err(BridgeError::UninitializedMetadata.into());
    }

    let metadata: mpl_token_metadata::state::Metadata = BorshDeserialize::deserialize(&mut metadata_info.data.borrow_mut().as_ref())?;

    let mut collection: Option<[u8; 32]> = {
        if metadata.collection.is_some() {
            Some(metadata.collection.unwrap().key.to_bytes())
        } else {
            None
        }
    };

    let content = ContentNode::new(
        origin.to_vec(),
        owner_info.key.to_bytes(),
        program_id.to_bytes(),
        TransferOperation::new_nft_transfer(
            mint_info.key.to_bytes(),
            collection,
            metadata.data.name.trim_matches(char::from(0)).to_string(),
            metadata.data.symbol.trim_matches(char::from(0)).to_string(),
            metadata.data.uri.trim_matches(char::from(0)).to_string(),
        ).get_operation(),
    );

    let root = get_merkle_root(content, &path)?;

    verify_ecdsa_signature(root.as_slice(), signature.as_slice(), recovery_id, bridge_admin.public_key)?;

    if *bridge_token_info.key !=
        get_associated_token_address(&bridge_admin_key, mint_info.key) {
        return Err(BridgeError::WrongTokenAccount.into());
    }

    if *owner_associated_info.key !=
        get_associated_token_address(&owner_info.key, mint_info.key) {
        return Err(BridgeError::WrongTokenAccount.into());
    }

    if owner_associated_info.data.borrow().as_ref().len() == 0 {
        msg!("Deposit owner associated account");
        call_create_associated_account(
            owner_info,
            owner_info,
            mint_info,
            owner_associated_info,
            rent_info,
            system_program,
            token_program,
        )?;
    }

    let transfer_tokens_instruction = transfer(
        &spl_token::id(),
        bridge_token_info.key,
        owner_associated_info.key,
        &bridge_admin_key,
        &[],
        1,
    )?;

    msg!("Transferring token");
    invoke_signed(
        &transfer_tokens_instruction,
        &[
            bridge_token_info.clone(),
            owner_associated_info.clone(),
            bridge_admin_info.clone(),
        ],
        &[&[&seeds]],
    )?;

    msg!("Creating withdraw account");
    call_create_account(
        owner_info,
        withdraw_info,
        rent_info,
        system_program,
        WITHDRAW_SIZE,
        program_id,
        &[origin.as_slice(), &[bump_seed]],
    )?;

    msg!("Initializing withdraw account");

    let mut withdraw: Withdraw = BorshDeserialize::deserialize(&mut withdraw_info.data.borrow_mut().as_ref())?;
    if withdraw.is_initialized {
        return Err(BridgeError::AlreadyInUse.into());
    }

    withdraw.is_initialized = true;
    withdraw.token_type = NFT;
    withdraw.origin = origin;
    withdraw.mint = Option::Some(mint_info.key.clone());
    withdraw.amount = 1;
    withdraw.receiver_address = *owner_info.key;
    withdraw.serialize(&mut *withdraw_info.data.borrow_mut())?;
    msg!("Withdraw account created");
    Ok(())
}


pub fn process_mint_ft<'a>(
    program_id: &'a Pubkey,
    accounts: &'a [AccountInfo<'a>],
    seeds: [u8; 32],
    data: DataV2,
    decimals: u8,
    amount: u64,
) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();
    let bridge_admin_info = next_account_info(account_info_iter)?;

    let mint_info = next_account_info(account_info_iter)?;
    let bridge_token_info = next_account_info(account_info_iter)?;
    let metadata_info = next_account_info(account_info_iter)?;
    let payer_info = next_account_info(account_info_iter)?;

    let token_program = next_account_info(account_info_iter)?;
    let _metadata_program = next_account_info(account_info_iter)?;
    let rent_info = next_account_info(account_info_iter)?;
    let system_program = next_account_info(account_info_iter)?;

    let bridge_admin_key = Pubkey::create_program_address(&[&seeds], &program_id).unwrap();
    if *bridge_admin_info.key != bridge_admin_key {
        return Err(BridgeError::WrongSeeds.into());
    }

    let bridge_admin: BridgeAdmin = BorshDeserialize::deserialize(&mut bridge_admin_info.data.borrow_mut().as_ref())?;
    if !bridge_admin.is_initialized {
        return Err(BridgeError::NotInitialized.into());
    }

    if *bridge_token_info.key !=
        get_associated_token_address(&bridge_admin_key, mint_info.key) {
        return Err(BridgeError::WrongTokenAccount.into());
    }

    msg!("Creating mint account");
    call_create_account(
        payer_info,
        mint_info,
        rent_info,
        system_program,
        Mint::LEN,
        &spl_token::id(),
        &[],
    )?;

    msg!("Initializing mint account");
    call_init_mint(
        mint_info,
        bridge_admin_info,
        rent_info,
        decimals,
    )?;

    msg!("Crating bridge admin associated account");
    call_create_associated_account(
        payer_info,
        bridge_admin_info,
        mint_info,
        bridge_token_info,
        rent_info,
        system_program,
        token_program,
    )?;

    msg!("Minting token to bridge admin");
    call_mint_to(
        mint_info,
        bridge_token_info,
        bridge_admin_info,
        seeds,
        amount,
    )?;

    msg!("Creating metadata account");
    call_create_metadata(
        metadata_info,
        mint_info,
        bridge_admin_info,
        payer_info,
        bridge_admin_info,
        data,
        rent_info,
        system_program,
        seeds,
    )?;

    Ok(())
}

pub fn process_mint_nft<'a>(
    program_id: &'a Pubkey,
    accounts: &'a [AccountInfo<'a>],
    seeds: [u8; 32],
    data: DataV2,
    verify: bool,
) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();
    let bridge_admin_info = next_account_info(account_info_iter)?;

    let mint_info = next_account_info(account_info_iter)?;
    let bridge_token_info = next_account_info(account_info_iter)?;
    let metadata_info = next_account_info(account_info_iter)?;
    let master_info = next_account_info(account_info_iter)?;

    let payer_info = next_account_info(account_info_iter)?;

    let token_program = next_account_info(account_info_iter)?;
    let _metadata_program = next_account_info(account_info_iter)?;
    let rent_info = next_account_info(account_info_iter)?;
    let system_program = next_account_info(account_info_iter)?;
    let _associated_program = next_account_info(account_info_iter)?;

    let bridge_admin_key = Pubkey::create_program_address(&[&seeds], &program_id).unwrap();
    if *bridge_admin_info.key != bridge_admin_key {
        return Err(BridgeError::WrongSeeds.into());
    }

    let bridge_admin: BridgeAdmin = BorshDeserialize::deserialize(&mut bridge_admin_info.data.borrow_mut().as_ref())?;
    if !bridge_admin.is_initialized {
        return Err(BridgeError::NotInitialized.into());
    }

    if *bridge_token_info.key !=
        get_associated_token_address(&bridge_admin_key, mint_info.key) {
        return Err(BridgeError::WrongTokenAccount.into());
    }

    msg!("Creating mint account");
    call_create_account(
        payer_info,
        mint_info,
        rent_info,
        system_program,
        Mint::LEN,
        &spl_token::id(),
        &[],
    )?;

    msg!("Initializing mint account");
    call_init_mint(
        mint_info,
        bridge_admin_info,
        rent_info,
        0,
    )?;

    msg!("Crating bridge admin associated account");
    call_create_associated_account(
        payer_info,
        bridge_admin_info,
        mint_info,
        bridge_token_info,
        rent_info,
        system_program,
        token_program,
    )?;

    msg!("Minting token to bridge admin");
    call_mint_to(
        mint_info,
        bridge_token_info,
        bridge_admin_info,
        seeds,
        1,
    )?;

    msg!("Creating metadata account");
    call_create_metadata(
        metadata_info,
        mint_info,
        bridge_admin_info,
        payer_info,
        bridge_admin_info,
        data,
        rent_info,
        system_program,
        seeds,
    )?;

    msg!("Creating master edition account");
    call_create_master_edition(
        master_info,
        mint_info,
        bridge_admin_info,
        bridge_admin_info,
        metadata_info,
        payer_info,
        token_program,
        system_program,
        rent_info,
        seeds,
    )?;

    if verify {
        let collection_info = next_account_info(account_info_iter)?;
        let collection_metadata_info = next_account_info(account_info_iter)?;
        let collection_master_info = next_account_info(account_info_iter)?;

        let verify_collection_instruction = verify_collection(
            mpl_token_metadata::id(),
            *metadata_info.key,
            bridge_admin_key,
            *payer_info.key,
            *collection_info.key,
            *collection_metadata_info.key,
            *collection_master_info.key,
            None,
        );

        msg!("Verifying collection");
        invoke_signed(
            &verify_collection_instruction,
            &[
                metadata_info.clone(),
                bridge_admin_info.clone(),
                payer_info.clone(),
                collection_info.clone(),
                collection_metadata_info.clone(),
                collection_master_info.clone(),
            ],
            &[&[&seeds]],
        )?;
    }

    Ok(())
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

fn call_mint_to<'a>(
    mint: &AccountInfo<'a>,
    account: &AccountInfo<'a>,
    owner: &AccountInfo<'a>,
    seeds: [u8; 32],
    amount: u64,
) -> ProgramResult {
    let mint_to_instruction = mint_to(
        &spl_token::id(),
        mint.key,
        account.key,
        owner.key,
        &[],
        amount,
    )?;

    invoke_signed(
        &mint_to_instruction,
        &[
            mint.clone(),
            account.clone(),
            owner.clone(),
        ],
        &[&[&seeds]],
    )
}

fn call_init_mint<'a>(
    mint: &AccountInfo<'a>,
    mint_authority: &AccountInfo<'a>,
    rent: &AccountInfo<'a>,
    decimals: u8,
) -> ProgramResult {
    let init_mint_instruction = initialize_mint(
        &spl_token::id(),
        mint.key,
        mint_authority.key,
        None,
        decimals,
    )?;

    invoke(
        &init_mint_instruction,
        &[
            mint.clone(),
            rent.clone(),
        ],
    )
}

fn call_create_master_edition<'a>(
    edition: &AccountInfo<'a>,
    mint: &AccountInfo<'a>,
    update_authority: &AccountInfo<'a>,
    mint_authority: &AccountInfo<'a>,
    metadata: &AccountInfo<'a>,
    payer: &AccountInfo<'a>,
    token_program: &AccountInfo<'a>,
    system_program: &AccountInfo<'a>,
    rent: &AccountInfo<'a>,
    seeds: [u8; 32],
) -> ProgramResult {
    let create_master_edition_instruction = create_master_edition_v3(
        mpl_token_metadata::id(),
        *edition.key,
        *mint.key,
        *update_authority.key,
        *mint_authority.key,
        *metadata.key,
        *payer.key,
        Some(0),
    );

    invoke_signed(
        &create_master_edition_instruction,
        &[
            edition.clone(),
            mint.clone(),
            update_authority.clone(),
            mint_authority.clone(),
            payer.clone(),
            metadata.clone(),
            token_program.clone(),
            system_program.clone(),
            rent.clone(),
        ],
        &[&[&seeds]],
    )
}

fn call_create_metadata<'a>(
    metadata_account: &AccountInfo<'a>,
    mint: &AccountInfo<'a>,
    mint_authority: &AccountInfo<'a>,
    payer: &AccountInfo<'a>,
    update_authority: &AccountInfo<'a>,
    data: DataV2,
    rent: &AccountInfo<'a>,
    system_program: &AccountInfo<'a>,
    seeds: [u8; 32],
) -> ProgramResult {
    let create_metadata_instruction = create_metadata_accounts_v2(
        mpl_token_metadata::id(),
        *metadata_account.key,
        *mint.key,
        *mint_authority.key,
        *payer.key,
        *mint_authority.key,
        data.name,
        data.symbol,
        data.uri,
        data.creators,
        data.seller_fee_basis_points,
        true,
        true,
        data.collection,
        data.uses,
    );

    invoke_signed(
        &create_metadata_instruction,
        &[
            metadata_account.clone(),
            mint.clone(),
            mint_authority.clone(),
            payer.clone(),
            update_authority.clone(),
            rent.clone(),
            system_program.clone(),
        ],
        &[&[&seeds]],
    )
}
