use solana_program::pubkey::Pubkey;
use std::hash::Hash;

const SOLANA_NETWORK: &str = "Solana";

pub struct ContentNode {
    // Hash of tx | event_id | network_from
    pub origin_hash: [u8; 32],
    // Empty line if is native
    pub address_to: Option<[u8; 32]>,
    // Empty line if is native or fungible
    pub token_id_to: Option<[u8; 32]>,
    pub receiver: [u8; 32],
    // Solana
    pub network_to: String,
    pub amount: u64,
    pub program_id: [u8; 32],
}

impl ContentNode {
    pub(crate) fn new(origin_hash: [u8; 32], amount: u64, mint: Option<[u8; 32]>, collection: Option<[u8; 32]>, receiver: [u8; 32], program_id: [u8; 32]) -> Self {
        ContentNode {
            origin_hash,
            address_to: collection,
            token_id_to: mint,
            receiver,
            network_to: String::from(SOLANA_NETWORK),
            amount,
            program_id,
        }
    }
}

impl ContentNode {
    pub fn hash(&self) -> solana_program::keccak::Hash {
        let mut data = Vec::new();

        if let Some(val) = self.address_to {
            data.append(&mut Vec::from(val.as_slice()));
        }

        if let Some(val) = self.token_id_to {
            data.append(&mut Vec::from(val.as_slice()));
        }

        data.append(&mut Vec::from(amount_bytes(self.amount)));
        data.append(&mut Vec::from(self.receiver.as_slice()));
        data.append(&mut Vec::from(self.origin_hash.as_slice()));
        data.append(&mut Vec::from(self.network_to.as_bytes()));
        data.append(&mut Vec::from(self.program_id.as_slice()));

        solana_program::keccak::hash(data.as_slice())
    }
}

fn amount_bytes(amount: u64) -> [u8; 32] {
    let bytes = amount.to_be_bytes();
    let mut result : [u8; 32] = [0; 32];

    for i in 0..bytes.len() {
        result[31 - i] = bytes[bytes.len() - 1 - i];
    }

    return result;
}