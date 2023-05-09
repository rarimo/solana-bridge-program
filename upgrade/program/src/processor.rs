use solana_program::{
    account_info::{AccountInfo, next_account_info},
    entrypoint::ProgramResult, msg,
    program::{invoke, invoke_signed}, pubkey::Pubkey, system_instruction,
    sysvar::{rent::Rent, Sysvar},
};
use crate::state::{MAX_ADMIN_SIZE, UpgradeAdmin};
use borsh::{
    BorshDeserialize, BorshSerialize,
};
use solana_program::secp256k1_recover::{SECP256K1_PUBLIC_KEY_LENGTH, SECP256K1_SIGNATURE_LENGTH};
use lib::ecdsa::verify_ecdsa_signature;
use lib::error::LibError;
use lib::instructions::upgrade::UpgradeInstruction;

pub fn process_instruction<'a>(
    program_id: &'a Pubkey,
    accounts: &'a [AccountInfo<'a>],
    input: &[u8],
) -> ProgramResult {
    let instruction = UpgradeInstruction::try_from_slice(input)?;
    match instruction {
        UpgradeInstruction::InitializeAdmin(args) => {
            msg!("Instruction: Create Upgrade Admin");
            process_init_admin(program_id, accounts, args.public_key, args.contract)
        }
        UpgradeInstruction::TransferOwnership(args) => {
            msg!("Instruction: Transfer ownership");
            process_transfer_ownership(program_id, accounts, args.new_public_key, args.signature, args.recovery_id)
        }
        UpgradeInstruction::Upgrade(args) => {
            msg!("Instruction: Upgrade");
            process_upgrade(program_id, accounts, args.signature, args.recovery_id)
        }
    }
}


pub fn process_init_admin<'a>(
    program_id: &'a Pubkey,
    accounts: &'a [AccountInfo<'a>],
    public_key: [u8; SECP256K1_PUBLIC_KEY_LENGTH],
    upgrade_program: Pubkey,
) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();

    let upgrade_admin_info = next_account_info(account_info_iter)?;
    let fee_payer_info = next_account_info(account_info_iter)?;
    let system_program = next_account_info(account_info_iter)?;
    let rent_info = next_account_info(account_info_iter)?;

    let upgrade_key = Pubkey::create_program_address(&[lib::UPGRADE_ADMIN_PDA_SEED.as_bytes(), upgrade_program.as_ref()], &program_id)?;
    if upgrade_key != *upgrade_admin_info.key {
        return Err(LibError::WrongAdmin.into());
    }

    lib::call_create_account(
        fee_payer_info,
        upgrade_admin_info,
        rent_info,
        system_program,
        MAX_ADMIN_SIZE,
        program_id,
        &[lib::UPGRADE_ADMIN_PDA_SEED.as_bytes()],
    )?;

    let mut upgrade_admin: UpgradeAdmin = BorshDeserialize::deserialize(&mut upgrade_admin_info.data.borrow_mut().as_ref())?;
    if upgrade_admin.is_initialized {
        return Err(LibError::AlreadyInUse.into());
    }

    upgrade_admin.contract = upgrade_program;
    upgrade_admin.public_key = public_key;
    upgrade_admin.is_initialized = true;
    upgrade_admin.serialize(&mut *upgrade_admin_info.data.borrow_mut())?;
    Ok(())
}


pub fn process_transfer_ownership<'a>(
    program_id: &'a Pubkey,
    accounts: &'a [AccountInfo<'a>],
    new_public_key: [u8; SECP256K1_PUBLIC_KEY_LENGTH],
    signature: [u8; SECP256K1_SIGNATURE_LENGTH],
    recovery_id: u8,
) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();
    let upgrade_admin_info = next_account_info(account_info_iter)?;

    let mut upgrade_admin: UpgradeAdmin = BorshDeserialize::deserialize(&mut upgrade_admin_info.data.borrow_mut().as_ref())?;
    if !upgrade_admin.is_initialized {
        return Err(LibError::NotInitialized.into());
    }

    let upgrade_admin_key = Pubkey::create_program_address(&[lib::UPGRADE_ADMIN_PDA_SEED.as_bytes(), upgrade_admin.contract.as_ref()], &program_id)?;
    if upgrade_admin_key != *upgrade_admin_info.key {
        return Err(LibError::WrongSeeds.into());
    }

    verify_ecdsa_signature(solana_program::keccak::hash(new_public_key.as_slice()).as_ref(), signature.as_slice(), recovery_id, upgrade_admin.public_key)?;

    upgrade_admin.public_key = new_public_key;
    upgrade_admin.serialize(&mut *upgrade_admin_info.data.borrow_mut())?;
    Ok(())
}

pub fn process_upgrade<'a>(
    program_id: &'a Pubkey,
    accounts: &'a [AccountInfo<'a>],
    signature: [u8; SECP256K1_SIGNATURE_LENGTH],
    recovery_id: u8,
) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();
    let upgrade_admin_info = next_account_info(account_info_iter)?;
    let upgrade_program_data = next_account_info(account_info_iter)?;
    let upgrade_program = next_account_info(account_info_iter)?;
    let upgrade_buffer = next_account_info(account_info_iter)?;
    let upgrade_spill = next_account_info(account_info_iter)?;
    let rent_info = next_account_info(account_info_iter)?;
    let clock_info = next_account_info(account_info_iter)?;

    let upgrade_admin_key = Pubkey::create_program_address(&[lib::UPGRADE_ADMIN_PDA_SEED.as_bytes(), upgrade_program.key.as_ref()], &program_id)?;
    if upgrade_admin_key != *upgrade_admin_info.key {
        return Err(LibError::WrongSeeds.into());
    }

    let mut upgrade_admin: UpgradeAdmin = BorshDeserialize::deserialize(&mut upgrade_admin_info.data.borrow_mut().as_ref())?;
    if !upgrade_admin.is_initialized {
        return Err(LibError::NotInitialized.into());
    }

    let instruction =  solana_program::bpf_loader_upgradeable::upgrade(
        upgrade_program.key,
        upgrade_buffer.key,
        &upgrade_admin_key,
        upgrade_spill.key,
    );

    let mut data = Vec::new();
    data.append(&mut Vec::from(solana_program::keccak::hash(&upgrade_buffer.data.borrow()).as_ref()));
    data.append(&mut Vec::from(lib::merkle::SOLANA_NETWORK));
    data.append(&mut Vec::from(lib::merkle::amount_bytes(upgrade_admin.nonce)));
    data.append(&mut Vec::from(program_id.as_ref()));

    verify_ecdsa_signature(solana_program::keccak::hash(data.as_slice()).as_ref(), signature.as_slice(), recovery_id, upgrade_admin.public_key)?;

    invoke_signed(
        &instruction,
        &[
            upgrade_program_data.clone(),
            upgrade_program.clone(),
            upgrade_buffer.clone(),
            upgrade_spill.clone(),
            rent_info.clone(),
            clock_info.clone(),
            upgrade_admin_info.clone(),
        ],
        &[&[lib::UPGRADE_ADMIN_PDA_SEED.as_bytes()]],
    )?;


    upgrade_admin.nonce = upgrade_admin.nonce + 1;
    upgrade_admin.serialize(&mut *upgrade_admin_info.data.borrow_mut())?;
    Ok(())
}