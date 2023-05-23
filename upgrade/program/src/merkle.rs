use lib::merkle::{SOLANA_NETWORK, amount_bytes};
use solana_program::pubkey::Pubkey;

pub struct Content {
    pub network: String,
    pub nonce: u64,
    pub contract: Pubkey,
    pub buffer: Pubkey,
}

impl Content {
    pub fn new(nonce: u64, contract: Pubkey, buffer: Pubkey) -> Self {
        Content {
            network: String::from(SOLANA_NETWORK),
            nonce,
            contract,
            buffer,
        }
    }

    pub fn hash(self) -> solana_program::keccak::Hash {
        let mut data = Vec::new();
        data.append(&mut Vec::from(self.network.as_bytes()));
        data.append(&mut Vec::from(amount_bytes(self.nonce)));
        data.append(&mut Vec::from(self.contract.as_ref()));
        data.append(&mut Vec::from(self.buffer.as_ref()));
        solana_program::keccak::hash(data.as_slice())
    }
}
