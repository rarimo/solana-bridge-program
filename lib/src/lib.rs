use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    account_info::{AccountInfo, next_account_info},
    entrypoint::ProgramResult, msg,
    program::{invoke, invoke_signed}, pubkey::Pubkey, system_instruction,
    sysvar::{rent::Rent, Sysvar},
};
use spl_associated_token_account::create_associated_token_account;

pub mod merkle;
pub mod ecdsa;
pub mod error;
pub mod instructions;

pub const SOLANA_NETWORK: &str = "Solana";

pub const COMMISSION_ADMIN_PDA_SEED: &str = "commission_admin";
pub const UPGRADE_ADMIN_PDA_SEED: &str = "upgrade_admin";

#[repr(C)]
#[derive(BorshSerialize, BorshDeserialize, PartialEq, Debug, Clone)]
pub enum TokenType {
    Native,
    NFT,
    FT,
}

#[repr(C)]
#[derive(BorshSerialize, BorshDeserialize, PartialEq, Debug, Clone)]
pub enum CommissionToken {
    Native,
    FT(Pubkey),
    NFT(Pubkey),
}

#[repr(C)]
#[derive(BorshSerialize, BorshDeserialize, PartialEq, Debug, Clone)]
pub struct CommissionArgs {
    pub token: CommissionToken,
    pub deposit_token: TokenType,
    pub deposit_token_amount: u64,
}

pub fn call_create_account<'a>(
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

pub fn call_create_associated_account<'a>(
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