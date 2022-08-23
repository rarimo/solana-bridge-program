use crate::error::BridgeError;
use solana_program::{
    hash, msg,
    entrypoint::ProgramResult,
};
use crate::instruction::SignedContent;
use crate::merkle_node::ContentNode;
use solana_program::secp256k1_recover::{secp256k1_recover, Secp256k1Pubkey};

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

pub(crate) fn verify_ecdsa_signature(message: &[u8], sig: &[u8], target_key: &[u8]) -> ProgramResult {
    let recovered_key = secp256k1_recover(hash::hash(message).as_ref(), 0, sig);
    if recovered_key.is_err() {
        return ProgramResult::Err(BridgeError::WrongSignature.into());
    }

    let key = Secp256k1Pubkey::new(target_key);

    if recovered_key.unwrap().ne(&key) {
        return ProgramResult::Err(BridgeError::WrongSignature.into());
    }

    Ok(())
}

pub(crate) fn verify_merkle_path(path: &Vec<[u8; 32]>, root: [u8; 32]) -> ProgramResult {
    if path.len() == 0 {
        return ProgramResult::Err(BridgeError::WrongMerklePath.into());
    }

    let mut hash = path[0];

    for i in 1..path.len() {
        let mut sum = Vec::from(hash);
        sum.append(&mut Vec::from(path[i]));
        hash = hash::hash(sum.as_slice()).to_bytes();
    }

    if hash != root {
        return ProgramResult::Err(BridgeError::WrongMerkleRoot.into());
    }

    Ok(())
}

pub(crate) fn verify_signed_content(target_hash: [u8; 32], content: &SignedContent, mint: String, collection: String, receiver: String) -> ProgramResult {
    let hash = hash::hash(ContentNode::new(content, mint, collection, receiver).to_string().as_bytes());
    if hash.to_bytes() != target_hash {
        return ProgramResult::Err(BridgeError::WrongContentHash.into());
    }
    Ok(())
}