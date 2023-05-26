use solana_program::program_error::ProgramError;

pub fn amount_bytes(amount: u64) -> Vec<u8> {
    let mut result: [u8; 32] = [0; 32];

    let bytes = amount.to_be_bytes();
    for i in 0..bytes.len() {
        result[31 - i] = bytes[bytes.len() - 1 - i];
    }

    return Vec::from(result);
}

pub fn get_merkle_root(mut hash: solana_program::keccak::Hash, path: &Vec<[u8; 32]>) -> Result<[u8; 32], ProgramError> {
    for i in 0..path.len() {
        let leaf = solana_program::keccak::Hash::new_from_array(path[i]);
        if leaf >= hash {
            hash = solana_program::keccak::hash([leaf.as_ref(), hash.as_ref()].concat().as_slice());
        } else {
            hash = solana_program::keccak::hash([hash.as_ref(), leaf.as_ref()].concat().as_slice());
        }
    }

    Result::Ok(hash.to_bytes())
}