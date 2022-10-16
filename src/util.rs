use std::hash::Hash;

use solana_program::{
    entrypoint::ProgramResult, hash,
    msg,
};
use solana_program::program_error::ProgramError;
use solana_program::secp256k1_recover::{SECP256K1_PUBLIC_KEY_LENGTH, secp256k1_recover, Secp256k1Pubkey};

use crate::error::BridgeError;
use crate::merkle::ContentNode;

pub(crate) fn verify_ecdsa_signature(hash: &[u8], sig: &[u8], reid: u8, target_key: [u8; SECP256K1_PUBLIC_KEY_LENGTH]) -> ProgramResult {
    let recovered_key = secp256k1_recover(hash, reid, sig);
    if recovered_key.is_err() {
        return ProgramResult::Err(BridgeError::InvalidSignature.into());
    }

    if recovered_key.unwrap().0 != target_key {
        return ProgramResult::Err(BridgeError::WrongSignature.into());
    }

    Ok(())
}

pub(crate) fn get_merkle_root(content: ContentNode, path: &Vec<[u8; 32]>) -> Result<[u8; 32], ProgramError> {
    let mut hash = content.hash();

    for i in 0..path.len() {
        let leaf = solana_program::keccak::Hash::new_from_array(path[i]);
        if leaf >= hash {
            hash = solana_program::keccak::hash([leaf.as_ref(), hash.as_ref()].concat().as_slice());
        } else {
            hash = solana_program::keccak::hash([hash.as_ref(), leaf.as_ref()].concat().as_slice());
        }
    }

    msg!(bs58::encode(hash.to_bytes().as_slice()).into_string().as_str());
    Result::Ok(hash.to_bytes())
}