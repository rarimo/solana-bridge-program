use crate::error::BridgeError;
use solana_program::{
    hash, msg,
    entrypoint::ProgramResult,
};

use secp256k1::{Secp256k1, Message, PublicKey};
use secp256k1::ecdsa::Signature;
use crate::error::BridgeError::WrongMerklePath;
use crate::instruction::SignedContent;
use crate::merkle_node::ContentNode;
use solana_program::pubkey::Pubkey;

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

pub(crate) fn verify_ecdsa_signature(message: &[u8], sig: &[u8], key: &[u8]) -> ProgramResult {
    let secp = Secp256k1::new();

    let msg = Message::from_slice(hash::hash(message).as_ref())?;
    let signature = Signature::from_compact(sig)?;
    let pubkey = PublicKey::from_slice(key)?;

    secp.verify_ecdsa(&msg, &signature, &pubkey)?;
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

    if path != root {
        return ProgramResult::Err(BridgeError::WrongMerkleRoot.into());
    }

    Ok(())
}

pub(crate) fn verify_signed_content(target_hash: [u8; 32], content: &SignedContent, mint: String, collection: String, receiver: String) -> ProgramResult {
    let hash = hash::hash(ContentNode::new(content, mint, collection, receiver).to_string().as_bytes());
    if hash != target_hash {
        return ProgramResult::Err(BridgeError::WrongContentHash.into());
    }
    Ok(())
}