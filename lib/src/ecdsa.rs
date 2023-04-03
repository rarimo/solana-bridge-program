use solana_program::secp256k1_recover::{SECP256K1_PUBLIC_KEY_LENGTH, secp256k1_recover, Secp256k1Pubkey};
use solana_program::{
    entrypoint::ProgramResult, hash,
    msg,
};
use solana_program::program_error::ProgramError;
use crate::error::ECDSAError;

pub fn verify_ecdsa_signature(hash: &[u8], sig: &[u8], reid: u8, target_key: [u8; SECP256K1_PUBLIC_KEY_LENGTH]) -> ProgramResult {
    let recovered_key = secp256k1_recover(hash, reid, sig);
    if recovered_key.is_err() {
        return ProgramResult::Err(ECDSAError::InvalidSignature.into());
    }

    if recovered_key.unwrap().0 != target_key {
        return ProgramResult::Err(ECDSAError::WrongSignature.into());
    }

    Ok(())
}

