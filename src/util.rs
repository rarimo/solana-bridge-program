use solana_program::pubkey::Pubkey;
use solana_program::account_info::AccountInfo;
use crate::error::BridgeError;
use solana_program::hash;
use solana_program::program_error::ProgramError;

pub(crate) fn get_mint_seeds_with_bump(token_id: &Option<String>, address: &Option<String>, mint_account_info: &AccountInfo, program_id: &Pubkey) -> Result<Vec<Vec<u8>>, ProgramError> {
    let mut seeds = get_mint_seeds(token_id, address);

    if seeds.len() > 0 {
        let (key, bump_seed) = Pubkey::find_program_address(seeds.iter().map(|seed| seed.as_slice()).collect::<Vec<&[u8]>>().as_ref(), program_id);

        if key != *mint_account_info.key {
            return Err(BridgeError::WrongMint.into());
        }

        seeds.push(vec![bump_seed])
    }

    return Ok(seeds);
}

pub(crate) fn get_mint_seeds(token_id: &Option<String>, address: &Option<String>) -> Vec<Vec<u8>> {
    let mut seeds = Vec::new();

    if let Some(token_id) = token_id {
        seeds.push(Vec::from(hash::hash(token_id.as_bytes()).to_bytes()));
    }

    if let Some(address) = address {
        seeds.push(Vec::from(hash::hash(address.as_bytes()).to_bytes()));
    }

    return seeds;
}
