use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult, msg, program::{invoke, invoke_signed},
    pubkey::Pubkey, sysvar::{rent::Rent, Sysvar}, hash, system_instruction,
};
use spl_token::{
    solana_program::program_pack::Pack,
    instruction::{transfer, initialize_mint, mint_to},
    state::{Mint},
};
use spl_associated_token_account::{get_associated_token_address, create_associated_token_account};
use borsh::{
    BorshDeserialize, BorshSerialize,
};
use crate::{
    instruction::BridgeInstruction,
    state::{BridgeAdmin, BRIDGE_ADMIN_SIZE},
    error::BridgeError,
    state::{DEPOSIT_SIZE, Deposit, WITHDRAW_SIZE, Withdraw},
    util,
};
use solana_program::pubkey::PubkeyError;
use mpl_token_metadata::{
    state::DataV2,
    instruction::{create_metadata_accounts_v2, verify_collection, create_master_edition_v3},
};
use spl_token::state::Account;
use solana_program::program_error::ProgramError;

pub fn process_instruction<'a>(
    program_id: &'a Pubkey,
    accounts: &'a [AccountInfo<'a>],
    input: &[u8],
) -> ProgramResult {
    let instruction = BridgeInstruction::try_from_slice(input)?;
    match instruction {
        BridgeInstruction::InitializeAdmin(args) => {
            msg!("Instruction: Create Bridge Admin");
            process_init_admin(program_id, accounts, args.seeds)
        }
        BridgeInstruction::TransferOwnership(args) => {
            msg!("Instruction: Transfer Bridge Admin ownership");
            process_transfer_ownership(program_id, accounts, args.seeds, args.new_admin)
        }
        BridgeInstruction::DepositMetaplex(args) => {
            msg!("Instruction: Deposit token");
            args.validate()?;
            process_deposit_metaplex(program_id, accounts, args.seeds, args.network_to, args.receiver_address, args.token_id, args.address, args.nonce)
        }
        BridgeInstruction::WithdrawMetaplex(args) => {
            msg!("Instruction: Withdraw token");
            args.validate()?;
            process_withdraw_metaplex(program_id, accounts, args.seeds, args.deposit_tx, args.network_from, args.sender_address, args.token_id)
        }
        BridgeInstruction::MintMetaplex(args) => {
            msg!("Instruction: Mint token");
            process_mint_metaplex(program_id, accounts, args.seeds, args.data, args.verify, args.token_id, args.address)
        }
    }
}

pub fn process_init_admin<'a>(
    program_id: &'a Pubkey,
    accounts: &'a [AccountInfo<'a>],
    seeds: [u8; 32],
) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();
    let bridge_admin_account_info = next_account_info(account_info_iter)?;
    let admin_account_info = next_account_info(account_info_iter)?;
    let system_program = next_account_info(account_info_iter)?;
    let rent_info = next_account_info(account_info_iter)?;

    let bridge_admin_key = Pubkey::create_program_address(&[&seeds], &program_id)?;
    if bridge_admin_key != *bridge_admin_account_info.key {
        return Err(BridgeError::WrongSeeds.into());
    }

    call_create_account(
        admin_account_info,
        bridge_admin_account_info,
        rent_info,
        system_program,
        BRIDGE_ADMIN_SIZE,
        program_id,
        &[&seeds],
    )?;

    let mut bridge_admin: BridgeAdmin = BorshDeserialize::deserialize(&mut bridge_admin_account_info.data.borrow_mut().as_ref())?;
    if bridge_admin.is_initialized {
        return Err(BridgeError::AlreadyInUse.into());
    }

    bridge_admin.admin = *admin_account_info.key;
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

    let bridge_admin_key = Pubkey::create_program_address(&[&seeds], &program_id)?;
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
    token_id: Option<String>,
    address: Option<String>,
    nonce: [u8; 32],
) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();
    let bridge_admin_account_info = next_account_info(account_info_iter)?;
    let mint_account_info = next_account_info(account_info_iter)?;
    let owner_associated_account_info = next_account_info(account_info_iter)?;
    let bridge_token_account_info = next_account_info(account_info_iter)?;
    let deposit_account_info = next_account_info(account_info_iter)?;
    let owner_account_info = next_account_info(account_info_iter)?;
    let token_program = next_account_info(account_info_iter)?;
    let system_program = next_account_info(account_info_iter)?;
    let rent_info = next_account_info(account_info_iter)?;

    let bridge_admin_key = Pubkey::create_program_address(&[&seeds], &program_id)?;
    if *bridge_admin_account_info.key != bridge_admin_key {
        return Err(BridgeError::WrongSeeds.into());
    }

    let bridge_admin: BridgeAdmin = BorshDeserialize::deserialize(&mut bridge_admin_account_info.data.borrow_mut().as_ref())?;
    if !bridge_admin.is_initialized {
        return Err(BridgeError::NotInitialized.into());
    }

    if *bridge_token_account_info.key !=
        get_associated_token_address(&bridge_admin_key, mint_account_info.key) {
        return Err(BridgeError::WrongTokenAccount.into());
    }

    if *owner_associated_account_info.key !=
        get_associated_token_address(&owner_account_info.key, mint_account_info.key) {
        return Err(BridgeError::WrongTokenAccount.into());
    }


    let mint_seeds = util::get_mint_seeds(&token_id, &address);
    if let Some(mint_seeds) = mint_seeds {
        let (key, ..) = Pubkey::find_program_address(&[&mint_seeds[0], &mint_seeds[1]], program_id);
        if key != *mint_account_info.key {
            return Err(BridgeError::WrongMint.into());
        }
    }

    if bridge_token_account_info.data.borrow().as_ref().len() == 0 {
        call_create_associated_account(
            owner_account_info,
            bridge_admin_account_info,
            mint_account_info,
            bridge_token_account_info,
            rent_info,
            system_program,
            token_program,
        )?;
    }

    let transfer_tokens_instruction = transfer(
        token_program.key,
        owner_associated_account_info.key,
        bridge_token_account_info.key,
        owner_account_info.key,
        &[],
        1,
    )?;

    invoke(
        &transfer_tokens_instruction,
        &[
            owner_associated_account_info.clone(),
            bridge_token_account_info.clone(),
            owner_account_info.clone(),
        ],
    )?;

    let (deposit_key, bump_seed) = Pubkey::find_program_address(&[&nonce], program_id);
    if deposit_key != *deposit_account_info.key {
        return Err(BridgeError::WrongNonce.into());
    }

    call_create_account(
        owner_account_info,
        deposit_account_info,
        rent_info,
        system_program,
        DEPOSIT_SIZE,
        program_id,
        &[&nonce, &[bump_seed]],
    )?;

    msg!("Deposit account created");

    let mut deposit: Deposit = BorshDeserialize::deserialize(&mut deposit_account_info.data.borrow_mut().as_ref())?;
    if deposit.is_initialized {
        return Err(BridgeError::AlreadyInUse.into());
    }

    deposit.is_initialized = true;
    deposit.network = network;
    deposit.receiver_address = receiver;
    deposit.token_id = token_id;
    deposit.address = address;
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
    token_id: Option<String>,
) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();
    let bridge_admin_account_info = next_account_info(account_info_iter)?;
    let mint_account_info = next_account_info(account_info_iter)?;
    let owner_account_info = next_account_info(account_info_iter)?;
    let owner_associated_account_info = next_account_info(account_info_iter)?;
    let bridge_token_account_info = next_account_info(account_info_iter)?;
    let withdraw_account_info = next_account_info(account_info_iter)?;
    let admin_account_info = next_account_info(account_info_iter)?;
    let token_program = next_account_info(account_info_iter)?;
    let system_program = next_account_info(account_info_iter)?;
    let rent_info = next_account_info(account_info_iter)?;

    let bridge_admin_key = Pubkey::create_program_address(&[&seeds], &program_id).unwrap();
    if *bridge_admin_account_info.key != bridge_admin_key {
        return Err(BridgeError::WrongSeeds.into());
    }

    let bridge_admin: BridgeAdmin = BorshDeserialize::deserialize(&mut bridge_admin_account_info.data.borrow_mut().as_ref())?;
    if !bridge_admin.is_initialized {
        return Err(BridgeError::NotInitialized.into());
    }

    if *admin_account_info.key != bridge_admin.admin {
        return Err(BridgeError::WrongAdmin.into());
    }

    if !admin_account_info.is_signer {
        return Err(BridgeError::UnsignedAdmin.into());
    }

    if *bridge_token_account_info.key !=
        get_associated_token_address(&bridge_admin_key, mint_account_info.key) {
        return Err(BridgeError::WrongTokenAccount.into());
    }

    if owner_associated_account_info.data.borrow().as_ref().len() == 0 {
        call_create_associated_account(
            owner_account_info,
            owner_account_info,
            mint_account_info,
            owner_associated_account_info,
            rent_info,
            system_program,
            token_program,
        )?;
    }

    let transfer_tokens_instruction = transfer(
        token_program.key,
        bridge_token_account_info.key,
        owner_associated_account_info.key,
        &bridge_admin_key,
        &[],
        1,
    )?;

    invoke_signed(
        &transfer_tokens_instruction,
        &[
            bridge_token_account_info.clone(),
            owner_associated_account_info.clone(),
            bridge_admin_account_info.clone(),
        ],
        &[&[&seeds]],
    )?;

    let nonce = hash::hash(tx.as_bytes()).to_bytes();

    let (withdraw_key, bump_seed) = Pubkey::find_program_address(&[&nonce], program_id);
    if withdraw_key != *withdraw_account_info.key {
        return Err(BridgeError::WrongNonce.into());
    }

    call_create_account(
        owner_account_info,
        withdraw_account_info,
        rent_info,
        system_program,
        WITHDRAW_SIZE,
        program_id,
        &[&nonce, &[bump_seed]],
    )?;

    msg!("Withdraw account created");

    let mut withdraw: Withdraw = BorshDeserialize::deserialize(&mut withdraw_account_info.data.borrow_mut().as_ref())?;
    if withdraw.is_initialized {
        return Err(BridgeError::AlreadyInUse.into());
    }

    withdraw.is_initialized = true;
    withdraw.network = network;
    withdraw.sender_address = sender;
    withdraw.token_id = token_id;
    withdraw.serialize(&mut *withdraw_account_info.data.borrow_mut())?;
    Ok(())
}

pub fn process_mint_metaplex<'a>(
    program_id: &'a Pubkey,
    accounts: &'a [AccountInfo<'a>],
    seeds: [u8; 32],
    data: DataV2,
    verify: bool,
    token_id: Option<String>,
    address: Option<String>,
) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();
    let bridge_admin_account_info = next_account_info(account_info_iter)?;

    let mint_account_info = next_account_info(account_info_iter)?;
    let bridge_token_account_info = next_account_info(account_info_iter)?;
    let metadata_account_info = next_account_info(account_info_iter)?;
    let master_account_info = next_account_info(account_info_iter)?;

    let admin_account_info = next_account_info(account_info_iter)?;
    let payer_account_info = next_account_info(account_info_iter)?;

    let token_program = next_account_info(account_info_iter)?;
    let metadata_program = next_account_info(account_info_iter)?;
    let rent_info = next_account_info(account_info_iter)?;
    let system_program = next_account_info(account_info_iter)?;
    let associated_program = next_account_info(account_info_iter)?;

    let bridge_admin_key = Pubkey::create_program_address(&[&seeds], &program_id).unwrap();
    if *bridge_admin_account_info.key != bridge_admin_key {
        return Err(BridgeError::WrongSeeds.into());
    }

    let bridge_admin: BridgeAdmin = BorshDeserialize::deserialize(&mut bridge_admin_account_info.data.borrow_mut().as_ref())?;
    if !bridge_admin.is_initialized {
        return Err(BridgeError::NotInitialized.into());
    }

    if *admin_account_info.key != bridge_admin.admin {
        return Err(BridgeError::WrongAdmin.into());
    }

    if !admin_account_info.is_signer {
        return Err(BridgeError::UnsignedAdmin.into());
    }

    if *bridge_token_account_info.key !=
        get_associated_token_address(&bridge_admin_key, mint_account_info.key) {
        return Err(BridgeError::WrongTokenAccount.into());
    }

    let mint_seeds = util::get_mint_seeds(&token_id, &address);
    if let Some(mint_seeds) = mint_seeds {
        let (key, bump_seed) = Pubkey::find_program_address(&[&mint_seeds[0], &mint_seeds[1]], program_id);
        if key != *mint_account_info.key {
            return Err(BridgeError::WrongMint.into());
        }

        msg!("Creating mint account with seeds");
        call_create_account(
            payer_account_info,
            mint_account_info,
            rent_info,
            system_program,
            Mint::LEN,
            token_program.key,
            &[&mint_seeds[0], &mint_seeds[1], &[bump_seed]],
        )?;
    } else {
        msg!("Creating mint account");
        call_create_account(
            payer_account_info,
            mint_account_info,
            rent_info,
            system_program,
            Mint::LEN,
            token_program.key,
            &[],
        )?;
    }
    
    msg!("Initializing mint account");
    call_init_mint(
        token_program.key,
        mint_account_info,
        bridge_admin_account_info,
        rent_info,
    )?;

    msg!("Crating bridge admin associated account");
    call_create_associated_account(
        payer_account_info,
        bridge_admin_account_info,
        mint_account_info,
        bridge_token_account_info,
        rent_info,
        system_program,
        token_program,
    )?;

    msg!("Minting token to bridge admin");
    call_mint_to(
        token_program.key,
        mint_account_info,
        bridge_token_account_info,
        bridge_admin_account_info,
        seeds,
    )?;

    msg!("Creating metadata account");
    call_create_metadata(
        *metadata_program.key,
        metadata_account_info,
        mint_account_info,
        bridge_admin_account_info,
        payer_account_info,
        bridge_admin_account_info,
        data,
        rent_info,
        system_program,
        seeds,
    )?;

    msg!("Creating master edition account");
    call_create_master_edition(
        *metadata_program.key,
        master_account_info,
        mint_account_info,
        bridge_admin_account_info,
        bridge_admin_account_info,
        metadata_account_info,
        payer_account_info,
        token_program,
        system_program,
        rent_info,
        seeds,
    )?;

    if verify {
        let collection_account_info = next_account_info(account_info_iter)?;
        let collection_metadata_account_info = next_account_info(account_info_iter)?;
        let collection_master_account_info = next_account_info(account_info_iter)?;

        let verify_collection_instruction = verify_collection(
            *metadata_program.key,
            *metadata_account_info.key,
            bridge_admin_key,
            *payer_account_info.key,
            *collection_account_info.key,
            *collection_metadata_account_info.key,
            *collection_master_account_info.key,
            None,
        );

        msg!("Verifying collection");
        invoke_signed(
            &verify_collection_instruction,
            &[
                metadata_account_info.clone(),
                bridge_admin_account_info.clone(),
                payer_account_info.clone(),
                collection_account_info.clone(),
                collection_metadata_account_info.clone(),
                collection_master_account_info.clone(),
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
    program_id: &Pubkey,
    mint: &AccountInfo<'a>,
    account: &AccountInfo<'a>,
    owner: &AccountInfo<'a>,
    seeds: [u8; 32],
) -> ProgramResult {
    let mint_to_instruction = mint_to(
        program_id,
        mint.key,
        account.key,
        owner.key,
        &[],
        1,
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
    program_id: &Pubkey,
    mint: &AccountInfo<'a>,
    mint_authority: &AccountInfo<'a>,
    rent: &AccountInfo<'a>,
) -> ProgramResult {
    let init_mint_instruction = initialize_mint(
        program_id,
        mint.key,
        mint_authority.key,
        None,
        0,
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
    program_id: Pubkey,
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
        program_id,
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
    program_id: Pubkey,
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
        program_id,
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
/*
#[cfg(test)]
mod tests {
    use super::*;
    use crate::instruction::*;
    use solana_program::{instruction::Instruction, program_error};
    use solana_sdk::account::{create_account_for_test, create_is_signer_account_infos, Account as SolanaAccount, ReadableAccount};
    use solana_program::instruction::AccountMeta;
    use solana_program::program_option::COption;
    use spl_token::state::{Account, AccountState};

    struct SyscallStubs {}

    impl solana_sdk::program_stubs::SyscallStubs for SyscallStubs {
        fn sol_log(&self, _message: &str) {}

        fn sol_invoke_signed(
            &self,
            _instruction: &Instruction,
            _account_infos: &[AccountInfo],
            _signers_seeds: &[&[&[u8]]],
        ) -> ProgramResult {
            msg!("Call invoke signed: {}", _instruction.program_id);

            // cloning account infos to make it mutable
            let mut account_infos: Vec<AccountInfo> = Vec::new();
            for info in _account_infos {
                account_infos.push(info.clone());
            }

            // signing with seeds
            if _signers_seeds.len() > 0 {
                for seed in _signers_seeds {
                    let key = Pubkey::create_program_address(*seed, &crate::entrypoint::id())?;
                    for mut info in account_infos.as_mut_slice() {
                        if *info.key == key {
                            info.is_signer = true;
                        }
                    }
                }
            }

            if _instruction.program_id == spl_token::id() {
                // call token program
                return spl_token::processor::Processor::process(&_instruction.program_id, account_infos.as_slice(), &_instruction.data);
            }

            Ok(())
        }

        fn sol_get_clock_sysvar(&self, _var_addr: *mut u8) -> u64 {
            program_error::UNSUPPORTED_SYSVAR
        }

        fn sol_get_epoch_schedule_sysvar(&self, _var_addr: *mut u8) -> u64 {
            program_error::UNSUPPORTED_SYSVAR
        }

        #[allow(deprecated)]
        fn sol_get_fees_sysvar(&self, _var_addr: *mut u8) -> u64 {
            program_error::UNSUPPORTED_SYSVAR
        }

        fn sol_get_rent_sysvar(&self, var_addr: *mut u8) -> u64 {
            unsafe {
                *(var_addr as *mut _ as *mut Rent) = Rent::default();
            }
            solana_program::entrypoint::SUCCESS
        }
    }

    fn do_process_instruction(
        instruction: Instruction,
        accounts: Vec<&mut SolanaAccount>,
    ) -> ProgramResult {
        {
            use std::sync::Once;
            static ONCE: Once = Once::new();

            ONCE.call_once(|| {
                solana_sdk::program_stubs::set_syscall_stubs(Box::new(SyscallStubs {}));
            });
        }

        let mut meta = instruction
            .accounts
            .iter()
            .zip(accounts)
            .map(|(account_meta, account)| (&account_meta.pubkey, account_meta.is_signer, account))
            .collect::<Vec<_>>();

        let account_infos = create_is_signer_account_infos(&mut meta);

        process_instruction(&instruction.program_id, &account_infos, &instruction.data)
    }

    fn rent_sysvar() -> SolanaAccount {
        create_account_for_test(&Rent::default())
    }

    fn token_program_account() -> SolanaAccount {
        SolanaAccount::new(0, 0, &spl_token::id())
    }

    #[test]
    fn test_initialize_bridge_account() {
        let program_id = crate::entrypoint::id();
        let admin_key = Pubkey::new_unique();
        let seeds = hash::hash("Seed for bridge admin account".as_bytes()).to_bytes();
        let bridge_key = Pubkey::create_program_address(&[&seeds], &program_id).unwrap();

        let mut bridge_account = SolanaAccount::new(Rent::default().minimum_balance(BRIDGE_ADMIN_SIZE), BRIDGE_ADMIN_SIZE, &program_id);

        // positive flow
        do_process_instruction(
            initialize_admin(program_id, bridge_key, admin_key, seeds),
            vec![
                &mut bridge_account,
                &mut SolanaAccount::default(),
                &mut SolanaAccount::default(),
                &mut rent_sysvar(),
            ],
        ).unwrap();

        let bridge: BridgeAdmin = BorshDeserialize::deserialize(&mut bridge_account.data.as_ref()).unwrap();
        assert_eq!(
            BridgeAdmin {
                admin: admin_key,
                is_initialized: true,
            },
            bridge
        );

        // create twice
        assert_eq!(
            Err(BridgeError::AlreadyInUse.into()),
            do_process_instruction(
                initialize_admin(program_id, bridge_key, admin_key, seeds),
                vec![
                    &mut bridge_account,
                    &mut SolanaAccount::default(),
                    &mut SolanaAccount::default(),
                    &mut rent_sysvar(),
                ],
            )
        );

        // wrong seeds
        assert_eq!(
            Err(BridgeError::WrongSeeds.into()),
            do_process_instruction(
                initialize_admin(program_id, Pubkey::new_unique(), admin_key, seeds),
                vec![
                    &mut SolanaAccount::new(Rent::default().minimum_balance(BRIDGE_ADMIN_SIZE), BRIDGE_ADMIN_SIZE, &program_id),
                    &mut SolanaAccount::default(),
                    &mut SolanaAccount::default(),
                    &mut rent_sysvar(),
                ],
            )
        );
    }

    #[test]
    fn test_transfer_ownership() {
        let program_id = crate::entrypoint::id();

        let mut admin_account = SolanaAccount::new(0, 0, &Pubkey::new_unique());
        let new_admin_key = Pubkey::new_unique();

        let seeds = hash::hash("Seed for bridge admin account".as_bytes()).to_bytes();
        let mut bridge_account = SolanaAccount::default();
        let bridge_key = init_bridge_account(&mut bridge_account, &seeds, &program_id, &admin_account.owner);

        let mut other_admin = SolanaAccount::new(0, 0, &Pubkey::new_unique());

        // Wrong admin
        assert_eq!(
            Err(BridgeError::WrongAdmin.into()),
            do_process_instruction(
                transfer_ownership(program_id, bridge_key, other_admin.owner, new_admin_key, seeds),
                vec![
                    &mut bridge_account,
                    &mut other_admin,
                ],
            )
        );

        // wrong seeds
        assert_eq!(
            Err(BridgeError::WrongSeeds.into()),
            do_process_instruction(
                transfer_ownership(program_id, Pubkey::new_unique(), admin_account.owner, new_admin_key, seeds),
                vec![
                    &mut SolanaAccount::new(Rent::default().minimum_balance(BRIDGE_ADMIN_SIZE), BRIDGE_ADMIN_SIZE, &program_id),
                    &mut other_admin,
                ],
            )
        );

        let mut unsigned_instruction = transfer_ownership(program_id, bridge_key, admin_account.owner, new_admin_key, seeds);
        unsigned_instruction.accounts[1] = AccountMeta::new(admin_account.owner, false);

        // Unsigned admin
        assert_eq!(
            Err(BridgeError::UnsignedAdmin.into()),
            do_process_instruction(
                unsigned_instruction,
                vec![
                    &mut bridge_account,
                    &mut admin_account,
                ],
            )
        );

        // positive flow
        do_process_instruction(
            transfer_ownership(program_id, bridge_key, admin_account.owner, new_admin_key, seeds),
            vec![
                &mut bridge_account,
                &mut admin_account,
            ],
        ).unwrap();

        let bridge: BridgeAdmin = BorshDeserialize::deserialize(&mut bridge_account.data.as_ref()).unwrap();
        assert_eq!(
            BridgeAdmin {
                admin: new_admin_key,
                is_initialized: true,
            },
            bridge
        );

        bridge_account =
            SolanaAccount::new(Rent::default().minimum_balance(BRIDGE_ADMIN_SIZE), BRIDGE_ADMIN_SIZE, &program_id);

        // not initialized
        assert_eq!(
            Err(BridgeError::NotInitialized.into()),
            do_process_instruction(
                transfer_ownership(program_id, bridge_key, admin_account.owner, new_admin_key, seeds),
                vec![
                    &mut bridge_account,
                    &mut admin_account,
                ],
            )
        );
    }

    #[test]
    fn test_deposit_metaplex() {
        let program_id = crate::entrypoint::id();
        let admin_account = SolanaAccount::new(0, 0, &Pubkey::new_unique());
        let seeds = hash::hash("Seed for bridge admin account".as_bytes()).to_bytes();

        let mut bridge_account = SolanaAccount::default();
        let bridge_key = init_bridge_account(&mut bridge_account, &seeds, &program_id, &admin_account.owner);

        let mut mint_account = SolanaAccount::default();
        let mint_key = init_mint_account(&mut mint_account);

        let owner_key = Pubkey::new_unique();
        let mut owner_account = SolanaAccount::new(0, 0, &owner_key);

        let mut owner_associated_account = SolanaAccount::default();
        let owner_associated_key = init_associated_account(&mut owner_associated_account, &owner_key, &mint_key, 1);

        let mut bridge_associated_account = SolanaAccount::default();
        let bridge_associated_key = init_associated_account(&mut bridge_associated_account, &bridge_key, &mint_key, 0);

        let nonce = Pubkey::new_unique().to_bytes();
        let (mut deposit_account, deposit_key) = get_nonce_account(nonce, &bridge_key, &program_id, DEPOSIT_SIZE);

        // positive flow
        do_process_instruction(
            deposit_metaplex(
                program_id,
                bridge_key,
                mint_key,
                owner_associated_key,
                bridge_associated_key,
                deposit_key,
                owner_key,
                seeds,
                "Ethereum".to_string(),
                "0xF65F3f18D9087c4E35BAC5b9746492082e186872".to_string(),
                Some("123456788".to_string()),
                nonce,
            ),
            vec![
                &mut bridge_account,
                &mut mint_account,
                &mut owner_associated_account,
                &mut bridge_associated_account,
                &mut deposit_account,
                &mut owner_account,
                &mut token_program_account(),
                &mut SolanaAccount::default(),
                &mut rent_sysvar(),
            ],
        ).unwrap();

        let owner_token_account = Account::unpack_from_slice(owner_associated_account.data.as_slice()).unwrap();
        let bridge_token_account = Account::unpack_from_slice(bridge_associated_account.data.as_slice()).unwrap();
        assert_eq!(
            owner_token_account.amount,
            0,
        );

        assert_eq!(
            bridge_token_account.amount,
            1,
        );

        let deposit: Deposit = BorshDeserialize::deserialize(&mut deposit_account.data()).unwrap();
        assert_eq!(
            Deposit {
                network: "Ethereum".to_string(),
                receiver_address: "0xF65F3f18D9087c4E35BAC5b9746492082e186872".to_string(),
                is_initialized: true,
                token_id: Some("123456788".to_string()),
            },
            deposit
        );

        // wrong seeds
        assert_eq!(
            Err(BridgeError::WrongSeeds.into()),
            do_process_instruction(
                deposit_metaplex(
                    program_id,
                    Pubkey::new_unique(),
                    mint_key,
                    owner_associated_key,
                    bridge_associated_key,
                    deposit_key,
                    owner_key,
                    seeds,
                    "Ethereum".to_string(),
                    "0xF65F3f18D9087c4E35BAC5b9746492082e186872".to_string(),
                    None,
                    nonce,
                ),
                vec![
                    &mut bridge_account,
                    &mut mint_account,
                    &mut owner_associated_account,
                    &mut bridge_associated_account,
                    &mut deposit_account,
                    &mut owner_account,
                    &mut token_program_account(),
                    &mut SolanaAccount::default(),
                    &mut rent_sysvar(),
                ],
            )
        );

        // bridge admin not initialized
        assert_eq!(
            Err(BridgeError::NotInitialized.into()),
            do_process_instruction(
                deposit_metaplex(
                    program_id,
                    bridge_key,
                    mint_key,
                    owner_associated_key,
                    bridge_associated_key,
                    deposit_key,
                    owner_key,
                    seeds,
                    "Ethereum".to_string(),
                    "0xF65F3f18D9087c4E35BAC5b9746492082e186872".to_string(),
                    None,
                    nonce,
                ),
                vec![
                    &mut SolanaAccount::new(Rent::default().minimum_balance(BRIDGE_ADMIN_SIZE), BRIDGE_ADMIN_SIZE, &program_id),
                    &mut mint_account,
                    &mut owner_associated_account,
                    &mut bridge_associated_account,
                    &mut deposit_account,
                    &mut owner_account,
                    &mut token_program_account(),
                    &mut SolanaAccount::default(),
                    &mut rent_sysvar(),
                ],
            )
        );

        // wrong bridge token account
        assert_eq!(
            Err(BridgeError::WrongTokenAccount.into()),
            do_process_instruction(
                deposit_metaplex(
                    program_id,
                    bridge_key,
                    mint_key,
                    owner_associated_key,
                    Pubkey::new_unique(),
                    deposit_key,
                    owner_key,
                    seeds,
                    "Ethereum".to_string(),
                    "0xF65F3f18D9087c4E35BAC5b9746492082e186872".to_string(),
                    None,
                    nonce,
                ),
                vec![
                    &mut bridge_account,
                    &mut mint_account,
                    &mut owner_associated_account,
                    &mut bridge_associated_account,
                    &mut deposit_account,
                    &mut owner_account,
                    &mut token_program_account(),
                    &mut SolanaAccount::default(),
                    &mut rent_sysvar(),
                ],
            )
        );

        // wrong owner token account
        assert_eq!(
            Err(BridgeError::WrongTokenAccount.into()),
            do_process_instruction(
                deposit_metaplex(
                    program_id,
                    bridge_key,
                    mint_key,
                    Pubkey::new_unique(),
                    bridge_associated_key,
                    deposit_key,
                    owner_key,
                    seeds,
                    "Ethereum".to_string(),
                    "0xF65F3f18D9087c4E35BAC5b9746492082e186872".to_string(),
                    None,
                    nonce,
                ),
                vec![
                    &mut bridge_account,
                    &mut mint_account,
                    &mut owner_associated_account,
                    &mut bridge_associated_account,
                    &mut deposit_account,
                    &mut owner_account,
                    &mut token_program_account(),
                    &mut SolanaAccount::default(),
                    &mut rent_sysvar(),
                ],
            )
        );

        let mut owner_associated_account = SolanaAccount::default();
        let owner_associated_key = init_associated_account(&mut owner_associated_account, &owner_key, &mint_key, 1);

        let mut bridge_associated_account = SolanaAccount::default();
        let bridge_associated_key = init_associated_account(&mut bridge_associated_account, &bridge_key, &mint_key, 0);

        // deposit wrong nonce
        assert_eq!(
            Err(BridgeError::WrongNonce.into()),
            do_process_instruction(
                deposit_metaplex(
                    program_id,
                    bridge_key,
                    mint_key,
                    owner_associated_key,
                    bridge_associated_key,
                    Pubkey::new_unique(),
                    owner_key,
                    seeds,
                    "Ethereum".to_string(),
                    "0xF65F3f18D9087c4E35BAC5b9746492082e186872".to_string(),
                    None,
                    nonce,
                ),
                vec![
                    &mut bridge_account,
                    &mut mint_account,
                    &mut owner_associated_account,
                    &mut bridge_associated_account,
                    &mut deposit_account,
                    &mut owner_account,
                    &mut token_program_account(),
                    &mut SolanaAccount::default(),
                    &mut rent_sysvar(),
                ],
            )
        );
    }

    #[test]
    fn test_withdraw_metaplex() {
        let program_id = crate::entrypoint::id();
        let mut admin_account = SolanaAccount::new(0, 0, &Pubkey::new_unique());
        let seeds = hash::hash("Seed for bridge admin account".as_bytes()).to_bytes();

        let mut bridge_account = SolanaAccount::default();
        let bridge_key = init_bridge_account(&mut bridge_account, &seeds, &program_id, &admin_account.owner);

        let mut mint_account = SolanaAccount::default();
        let mint_key = init_mint_account(&mut mint_account);

        let mut owner_associated_account = SolanaAccount::default();
        let owner_associated_key = init_associated_account(&mut owner_associated_account, &Pubkey::new_unique(), &mint_key, 0);

        let mut bridge_associated_account = SolanaAccount::default();
        let bridge_associated_key = init_associated_account(&mut bridge_associated_account, &bridge_key, &mint_key, 1);

        let nonce = hash::hash("0xe7c7d1b3c59da71c1716b1fc88769857b5d5c8d191d53b9a8d2b66261ecd25ef".as_bytes()).to_bytes();
        let (mut withdraw_account, withdraw_key) = get_nonce_account(nonce, &bridge_key, &program_id, WITHDRAW_SIZE);

        // positive flow
        do_process_instruction(
            withdraw_metaplex(
                program_id,
                bridge_key,
                mint_key,
                owner_associated_account.owner,
                owner_associated_key,
                bridge_associated_key,
                withdraw_key,
                admin_account.owner,
                seeds,
                "0xe7c7d1b3c59da71c1716b1fc88769857b5d5c8d191d53b9a8d2b66261ecd25ef".to_string(),
                "Ethereum".to_string(),
                "0xf65f3f18d9087c4e35bac5b9746492082e186872".to_string(),
                Some("123456788".to_string()),
            ),
            vec![
                &mut bridge_account,
                &mut mint_account,
                &mut SolanaAccount::default(),
                &mut owner_associated_account,
                &mut bridge_associated_account,
                &mut withdraw_account,
                &mut admin_account,
                &mut token_program_account(),
                &mut SolanaAccount::default(),
                &mut rent_sysvar(),
            ],
        ).unwrap();


        let owner_token_account = Account::unpack_from_slice(owner_associated_account.data.as_slice()).unwrap();
        let bridge_token_account = Account::unpack_from_slice(bridge_associated_account.data.as_slice()).unwrap();
        assert_eq!(
            owner_token_account.amount,
            1,
        );

        assert_eq!(
            bridge_token_account.amount,
            0,
        );

        let withdraw: Withdraw = BorshDeserialize::deserialize(&mut withdraw_account.data()).unwrap();
        assert_eq!(
            Withdraw {
                network: "Ethereum".to_string(),
                is_initialized: true,
                sender_address: "0xf65f3f18d9087c4e35bac5b9746492082e186872".to_string(),
                token_id: Some("123456788".to_string()),
            },
            withdraw
        );

        // wrong seeds
        assert_eq!(
            Err(BridgeError::WrongSeeds.into()),
            do_process_instruction(
                withdraw_metaplex(
                    program_id,
                    Pubkey::new_unique(),
                    mint_key,
                    owner_associated_account.owner,
                    owner_associated_key,
                    bridge_associated_key,
                    withdraw_key,
                    admin_account.owner,
                    seeds,
                    "0xe7c7d1b3c59da71c1716b1fc88769857b5d5c8d191d53b9a8d2b66261ecd25ef".to_string(),
                    "Ethereum".to_string(),
                    "0xf65f3f18d9087c4e35bac5b9746492082e186872".to_string(),
                    None,
                ),
                vec![
                    &mut bridge_account,
                    &mut mint_account,
                    &mut SolanaAccount::default(),
                    &mut owner_associated_account,
                    &mut bridge_associated_account,
                    &mut withdraw_account,
                    &mut admin_account,
                    &mut token_program_account(),
                    &mut SolanaAccount::default(),
                    &mut rent_sysvar(),
                ],
            )
        );

        // bridge account not initialized
        assert_eq!(
            Err(BridgeError::NotInitialized.into()),
            do_process_instruction(
                withdraw_metaplex(
                    program_id,
                    bridge_key,
                    mint_key,
                    owner_associated_account.owner,
                    owner_associated_key,
                    bridge_associated_key,
                    withdraw_key,
                    admin_account.owner,
                    seeds,
                    "0xe7c7d1b3c59da71c1716b1fc88769857b5d5c8d191d53b9a8d2b66261ecd25ef".to_string(),
                    "Ethereum".to_string(),
                    "0xf65f3f18d9087c4e35bac5b9746492082e186872".to_string(),
                    None,
                ),
                vec![
                    &mut SolanaAccount::new(Rent::default().minimum_balance(BRIDGE_ADMIN_SIZE), BRIDGE_ADMIN_SIZE, &program_id),
                    &mut mint_account,
                    &mut SolanaAccount::default(),
                    &mut owner_associated_account,
                    &mut bridge_associated_account,
                    &mut withdraw_account,
                    &mut admin_account,
                    &mut token_program_account(),
                    &mut SolanaAccount::default(),
                    &mut rent_sysvar(),
                ],
            )
        );

        // wrong bridge admin key
        assert_eq!(
            Err(BridgeError::WrongAdmin.into()),
            do_process_instruction(
                withdraw_metaplex(
                    program_id,
                    bridge_key,
                    mint_key,
                    owner_associated_account.owner,
                    owner_associated_key,
                    bridge_associated_key,
                    withdraw_key,
                    Pubkey::new_unique(),
                    seeds,
                    "0xe7c7d1b3c59da71c1716b1fc88769857b5d5c8d191d53b9a8d2b66261ecd25ef".to_string(),
                    "Ethereum".to_string(),
                    "0xf65f3f18d9087c4e35bac5b9746492082e186872".to_string(),
                    None,
                ),
                vec![
                    &mut bridge_account,
                    &mut mint_account,
                    &mut SolanaAccount::default(),
                    &mut owner_associated_account,
                    &mut bridge_associated_account,
                    &mut withdraw_account,
                    &mut admin_account,
                    &mut token_program_account(),
                    &mut SolanaAccount::default(),
                    &mut rent_sysvar(),
                ],
            )
        );

        let mut unsigned_instruction = withdraw_metaplex(
            program_id,
            bridge_key,
            mint_key,
            owner_associated_account.owner,
            owner_associated_key,
            bridge_associated_key,
            withdraw_key,
            admin_account.owner,
            seeds,
            "0xe7c7d1b3c59da71c1716b1fc88769857b5d5c8d191d53b9a8d2b66261ecd25ef".to_string(),
            "Ethereum".to_string(),
            "0xf65f3f18d9087c4e35bac5b9746492082e186872".to_string(),
            None,
        );

        unsigned_instruction.accounts[6] = AccountMeta::new(admin_account.owner, false);
        // admin account unsigned
        assert_eq!(
            Err(BridgeError::UnsignedAdmin.into()),
            do_process_instruction(
                unsigned_instruction,
                vec![
                    &mut bridge_account,
                    &mut mint_account,
                    &mut SolanaAccount::default(),
                    &mut owner_associated_account,
                    &mut bridge_associated_account,
                    &mut withdraw_account,
                    &mut admin_account,
                    &mut token_program_account(),
                    &mut SolanaAccount::default(),
                    &mut rent_sysvar(),
                ],
            )
        );

        // wrong bridge token account
        assert_eq!(
            Err(BridgeError::WrongTokenAccount.into()),
            do_process_instruction(
                withdraw_metaplex(
                    program_id,
                    bridge_key,
                    mint_key,
                    owner_associated_account.owner,
                    owner_associated_key,
                    Pubkey::new_unique(),
                    withdraw_key,
                    admin_account.owner,
                    seeds,
                    "0xe7c7d1b3c59da71c1716b1fc88769857b5d5c8d191d53b9a8d2b66261ecd25ef".to_string(),
                    "Ethereum".to_string(),
                    "0xf65f3f18d9087c4e35bac5b9746492082e186872".to_string(),
                    None,
                ),
                vec![
                    &mut bridge_account,
                    &mut mint_account,
                    &mut SolanaAccount::default(),
                    &mut owner_associated_account,
                    &mut bridge_associated_account,
                    &mut withdraw_account,
                    &mut admin_account,
                    &mut token_program_account(),
                    &mut SolanaAccount::default(),
                    &mut rent_sysvar(),
                ],
            )
        );

        let mut owner_associated_account = SolanaAccount::default();
        let owner_associated_key = init_associated_account(&mut owner_associated_account, &Pubkey::new_unique(), &mint_key, 0);

        let mut bridge_associated_account = SolanaAccount::default();
        let bridge_associated_key = init_associated_account(&mut bridge_associated_account, &bridge_key, &mint_key, 1);

        // wrong nonce for withdraw (wrong pub key or tx)
        assert_eq!(
            Err(BridgeError::WrongNonce.into()),
            do_process_instruction(
                withdraw_metaplex(
                    program_id,
                    bridge_key,
                    mint_key,
                    owner_associated_account.owner,
                    owner_associated_key,
                    bridge_associated_key,
                    withdraw_key,
                    admin_account.owner,
                    seeds,
                    "0xtxtxtxtxtxttxtxtxtxttxtx".to_string(),
                    "Ethereum".to_string(),
                    "0xf65f3f18d9087c4e35bac5b9746492082e186872".to_string(),
                    None,
                ),
                vec![
                    &mut bridge_account,
                    &mut mint_account,
                    &mut SolanaAccount::default(),
                    &mut owner_associated_account,
                    &mut bridge_associated_account,
                    &mut withdraw_account,
                    &mut admin_account,
                    &mut token_program_account(),
                    &mut SolanaAccount::default(),
                    &mut rent_sysvar(),
                ],
            )
        );

        let mut owner_associated_account = SolanaAccount::default();
        let owner_associated_key = init_associated_account(&mut owner_associated_account, &Pubkey::new_unique(), &mint_key, 0);

        let mut bridge_associated_account = SolanaAccount::default();
        let bridge_associated_key = init_associated_account(&mut bridge_associated_account, &bridge_key, &mint_key, 1);

        // withdraw already initialized
        assert_eq!(
            Err(BridgeError::AlreadyInUse.into()),
            do_process_instruction(
                withdraw_metaplex(
                    program_id,
                    bridge_key,
                    mint_key,
                    owner_associated_account.owner,
                    owner_associated_key,
                    bridge_associated_key,
                    withdraw_key,
                    admin_account.owner,
                    seeds,
                    "0xe7c7d1b3c59da71c1716b1fc88769857b5d5c8d191d53b9a8d2b66261ecd25ef".to_string(),
                    "Ethereum".to_string(),
                    "0xf65f3f18d9087c4e35bac5b9746492082e186872".to_string(),
                    None,
                ),
                vec![
                    &mut bridge_account,
                    &mut mint_account,
                    &mut SolanaAccount::default(),
                    &mut owner_associated_account,
                    &mut bridge_associated_account,
                    &mut withdraw_account,
                    &mut admin_account,
                    &mut token_program_account(),
                    &mut SolanaAccount::default(),
                    &mut rent_sysvar(),
                ],
            )
        );
    }

    fn get_nonce_account(nonce: [u8; 32], owner: &Pubkey, program_id: &Pubkey, size: usize) -> (SolanaAccount, Pubkey) {
        let (_, bump_seed) = Pubkey::find_program_address(&[&nonce], owner);

        return (
            SolanaAccount::new(Rent::default().minimum_balance(size), size, program_id),
            Pubkey::create_program_address(&[&nonce, &[bump_seed]], owner).unwrap()
        );
    }

    fn init_associated_account(associated_account: &mut SolanaAccount, owner_key: &Pubkey, mint_key: &Pubkey, amount: u64) -> Pubkey {
        *associated_account = SolanaAccount::new(Rent::default().minimum_balance(Account::LEN), Account::LEN, &owner_key);
        let account = Account {
            mint: mint_key.clone(),
            owner: owner_key.clone(),
            amount,
            delegate: COption::None,
            state: AccountState::Initialized,
            is_native: COption::None,
            delegated_amount: 0,
            close_authority: COption::None,
        };
        account.pack_into_slice(&mut associated_account.data);
        return spl_associated_token_account::get_associated_token_address(&owner_key, &mint_key);
    }

    fn init_mint_account(mint_account: &mut SolanaAccount) -> Pubkey {
        *mint_account = SolanaAccount::new(Rent::default().minimum_balance(Mint::LEN), Mint::LEN, &spl_token::id());
        let mint = Mint {
            mint_authority: COption::None,
            supply: 1,
            decimals: 0,
            is_initialized: true,
            freeze_authority: COption::None,
        };

        mint.pack_into_slice(&mut mint_account.data);
        return Pubkey::new_unique();
    }

    fn init_bridge_account(bridge_account: &mut SolanaAccount, seeds: &[u8; 32], program_id: &Pubkey, admin: &Pubkey) -> Pubkey {
        *bridge_account =
            SolanaAccount::new(Rent::default().minimum_balance(BRIDGE_ADMIN_SIZE), 0, &program_id);
        let bridge = BridgeAdmin {
            admin: admin.clone(),
            is_initialized: true,
        };
        bridge.serialize(&mut bridge_account.data);
        return Pubkey::create_program_address(&[seeds], &program_id).unwrap();
    }
}
*/