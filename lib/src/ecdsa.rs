use solana_program::secp256k1_recover::{SECP256K1_PUBLIC_KEY_LENGTH, secp256k1_recover, Secp256k1Pubkey};
use solana_program::{
    entrypoint::ProgramResult, hash,
    msg,
};
use solana_program::program_error::ProgramError;
use crate::error::LibError;

pub fn verify_ecdsa_signature(hash: &[u8], sig: &[u8], reid: u8, target_key: [u8; SECP256K1_PUBLIC_KEY_LENGTH]) -> ProgramResult {
    let recovered_key = secp256k1_recover(hash, reid, sig);
    if recovered_key.is_err() {
        return ProgramResult::Err(LibError::InvalidSignature.into());
    }

    let key =  recovered_key.unwrap().0;

    msg!("Recovered public key from signature: {}", bs58::encode(key.as_ref()).into_string().as_str());
    msg!("Required public key: {}", bs58::encode(target_key.as_ref()).into_string().as_str());

    if key != target_key {
        return ProgramResult::Err(LibError::WrongSignature.into());
    }

    Ok(())
}

