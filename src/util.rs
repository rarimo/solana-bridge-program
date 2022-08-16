use crate::error::BridgeError;
use solana_program::{
    hash, msg,
    entrypoint::ProgramResult
};

pub(crate) fn get_mint_seeds(token_id: &Option<String>, address: &Option<String>) -> Option<[[u8; 32]; 2]> {
    if let (Some(token_id), Some(address)) = (token_id, address) {
        msg!("Mint seeds found");
        return Some([hash::hash(token_id.as_bytes()).to_bytes(), hash::hash(address.as_bytes()).to_bytes()]);
    }
    return None;
}

pub(crate) fn validate_option_str(opt: &Option<String>, sz: usize) -> ProgramResult {
    if let Some(opt) = opt {
        if opt.as_bytes().len() > sz {
            return Err(BridgeError::WrongArgsSize.into());
        }
    }

    Ok(())
}