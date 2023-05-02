use solana_program::program_error::ProgramError;

pub const SOLANA_NETWORK: &str = "Solana";

pub trait Data {
    fn get_operation(&self) -> Vec<u8>;
}

pub struct ContentNode {
    pub origin: [u8; 32],
    pub network_to: String,
    pub receiver: [u8; 32],
    pub program_id: [u8; 32],
    pub data: Vec<u8>,
}

impl ContentNode {
    pub fn new(origin: [u8; 32], receiver: [u8; 32], program_id: [u8; 32], data: Box<dyn Data>) -> Self {
        ContentNode {
            origin,
            receiver,
            network_to: String::from(SOLANA_NETWORK),
            program_id,
            data: data.get_operation(),
        }
    }

    pub fn hash(self) -> solana_program::keccak::Hash {
        let mut data = Vec::new();
        data.append(&mut Vec::from(self.data));

        data.append(&mut Vec::from(self.origin.as_slice()));

        data.append(&mut Vec::from(self.network_to.as_bytes()));

        data.append(&mut Vec::from(self.receiver.as_slice()));

        data.append(&mut Vec::from(self.program_id.as_slice()));

        solana_program::keccak::hash(data.as_slice())
    }
}

pub fn amount_bytes(amount: u64) -> [u8; 32] {
    let bytes = amount.to_be_bytes();
    let mut result: [u8; 32] = [0; 32];

    for i in 0..bytes.len() {
        result[31 - i] = bytes[bytes.len() - 1 - i];
    }

    return result;
}

pub fn get_merkle_root(content: ContentNode, path: &Vec<[u8; 32]>) -> Result<[u8; 32], ProgramError> {
    let mut hash = content.hash();

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