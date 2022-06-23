use solana_program::pubkey::Pubkey;
use solana_program::account_info::AccountInfo;
use crate::error::BridgeError;
use solana_program::{hash, msg, program_error::ProgramError};

pub(crate) fn get_mint_seeds(token_id: &Option<String>, address: &Option<String>) -> Option<[[u8; 32]; 2]> {
    if let (Some(token_id), Some(address)) = (token_id, address) {
        msg!("Mint seeds found");
        return Some([hash::hash(token_id.as_bytes()).to_bytes(), hash::hash(address.as_bytes()).to_bytes()]);
    }
    return None;
}
