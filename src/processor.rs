use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    msg,
    program::{invoke, invoke_signed},
    pubkey::Pubkey,
    sysvar::{rent::Rent, Sysvar},
    hash,
};
use spl_token::{
    solana_program::program_pack::Pack,
    instruction::transfer,
    state::Mint,
};
use spl_associated_token_account::get_associated_token_address;
use borsh::{
    BorshDeserialize, BorshSerialize,
};
use crate::{
    instruction::BridgeInstruction,
    state::{BridgeAdmin, BRIDGE_ADMIN_SIZE},
    error::BridgeError,
    state::{DEPOSIT_SIZE, Deposit, WITHDRAW_SIZE, Withdraw},
};

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
            process_withdraw_metaplex(program_id, accounts, args.seeds, args.deposit_tx, args.network_from, args.sender_address)
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

    let bridge_admin_key = Pubkey::create_program_address(&[&seeds], &program_id).unwrap();
    if bridge_admin_key != *bridge_admin_account_info.key {
        return Err(BridgeError::WrongSeeds.into());
    }

    if bridge_admin_account_info.data_len() != BRIDGE_ADMIN_SIZE {
        return Err(BridgeError::WrongDataLen.into());
    }

    let rent = Rent::from_account_info(rent_info)?;
    if !rent.is_exempt(bridge_admin_account_info.lamports(), BRIDGE_ADMIN_SIZE) {
        return Err(BridgeError::NotRentExempt.into());
    }

    let mut bridge_admin: BridgeAdmin = BorshDeserialize::deserialize(&mut bridge_admin_account_info.data.borrow_mut().as_ref())?;
    if bridge_admin.is_initialized {
        return Err(BridgeError::AlreadyInUse.into());
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

    let bridge_admin: BridgeAdmin = BorshDeserialize::deserialize(&mut bridge_admin_account_info.data.borrow_mut().as_ref())?;
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

    if deposit_account_info.data_len() != DEPOSIT_SIZE {
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
) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();
    let bridge_admin_account_info = next_account_info(account_info_iter)?;
    let mint_account_info = next_account_info(account_info_iter)?;
    let owner_associated_account_info = next_account_info(account_info_iter)?;
    let owner_account_info = next_account_info(account_info_iter)?;
    let program_token_account_info = next_account_info(account_info_iter)?;
    let withdraw_account_info = next_account_info(account_info_iter)?;
    let admin_account_info = next_account_info(account_info_iter)?;
    let token_program = next_account_info(account_info_iter)?;
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

    if *program_token_account_info.key !=
        get_associated_token_address(&bridge_admin_key, mint_account_info.key) {
        return Err(BridgeError::WrongTokenAccount.into());
    }

    if *owner_associated_account_info.key !=
        get_associated_token_address(owner_account_info.key, mint_account_info.key) {
        return Err(BridgeError::WrongTokenAccount.into());
    }

    let mint = Mint::unpack_from_slice(mint_account_info.data.borrow().as_ref())?;
    if !mint.is_initialized {
        return Err(BridgeError::NotInitialized.into());
    }

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


    let nonce = hash::hash(tx.as_bytes()).to_bytes();
    let withdraw_key = Pubkey::create_program_address(&[&nonce], &bridge_admin_key).unwrap();
    if withdraw_key != *withdraw_account_info.key {
        return Err(BridgeError::WrongNonce.into());
    }

    let mut withdraw: Withdraw = BorshDeserialize::deserialize(&mut withdraw_account_info.data.borrow_mut().as_ref())?;
    if withdraw.is_initialized {
        return Err(BridgeError::AlreadyInUse.into());
    }

    if withdraw_account_info.data_len() != WITHDRAW_SIZE {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::instruction::*;
    use solana_program::{
        account_info::IntoAccountInfo, clock::Epoch, instruction::Instruction, program_error,
        sysvar::rent,
    };
    use solana_sdk::account::{
        create_account_for_test, create_is_signer_account_infos, Account as SolanaAccount,
    };
    use std::net::Shutdown::Both;
    use solana_program::instruction::AccountMeta;

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

    #[test]
    fn test_initialize_mint() {
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

        // account is not rent exempt
        assert_eq!(
            Err(BridgeError::NotRentExempt.into()),
            do_process_instruction(
                initialize_admin(program_id, bridge_key, admin_key, seeds),
                vec![
                    &mut SolanaAccount::new(Rent::default().minimum_balance(BRIDGE_ADMIN_SIZE) - 1, BRIDGE_ADMIN_SIZE, &program_id),
                    &mut rent_sysvar(),
                ],
            )
        );

        // create twice
        assert_eq!(
            Err(BridgeError::AlreadyInUse.into()),
            do_process_instruction(
                initialize_admin(program_id, bridge_key, admin_key, seeds),
                vec![
                    &mut bridge_account,
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
                    &mut rent_sysvar(),
                ],
            )
        );

        // wrong data len
        assert_eq!(
            Err(BridgeError::WrongDataLen.into()),
            do_process_instruction(
                initialize_admin(program_id, bridge_key, admin_key, seeds),
                vec![
                    &mut SolanaAccount::new(Rent::default().minimum_balance(BRIDGE_ADMIN_SIZE), BRIDGE_ADMIN_SIZE - 1, &program_id),
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
        let bridge_key = Pubkey::create_program_address(&[&seeds], &program_id).unwrap();

        let bridge = BridgeAdmin {
            admin: admin_account.owner,
            is_initialized: true,
        };

        let mut bridge_account =
            SolanaAccount::new(Rent::default().minimum_balance(BRIDGE_ADMIN_SIZE), 0, &program_id);
        bridge.serialize(&mut bridge_account.data);

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
}
