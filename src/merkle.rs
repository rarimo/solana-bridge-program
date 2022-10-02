use solana_program::{
    pubkey::Pubkey,
    msg,
};
use std::hash::Hash;

const SOLANA_NETWORK: &str = "Solana";
const SOLANA_NATIVE_DECIMALS: u8 = 9u8;

pub trait Operation {
    fn get_operation(&self) -> Vec<u8>;
}

pub struct TransferFullMetaOperation {
    // Empty line if is native
    pub address_to: Option<[u8; 32]>,
    // Empty line if is native or fungible
    pub token_id_to: Option<[u8; 32]>,
    pub amount: u64,
    pub name: String,
    pub symbol: String,
    pub uri: String,
    pub decimals: u8,
}

impl TransferFullMetaOperation {
    pub fn new_ft_transfer(mint: [u8; 32], amount: u64, name: String, symbol: String, uri: String, decimals: u8) -> Self {
        TransferFullMetaOperation {
            address_to: Some(mint),
            token_id_to: None,
            amount,
            name,
            symbol,
            uri,
            decimals,
        }
    }

    pub fn new_nft_transfer(mint: [u8; 32], collection: Option<[u8; 32]>, name: String, symbol: String, uri: String) -> Self {
        TransferFullMetaOperation {
            address_to: collection,
            token_id_to: Some(mint),
            amount: 1,
            name,
            symbol,
            uri,
            decimals: 0,
        }
    }
}

impl Operation for TransferFullMetaOperation {
    fn get_operation(&self) -> Vec<u8> {
        let mut data = Vec::new();

        if let Some(val) = self.address_to {
            data.append(&mut Vec::from(val.as_slice()));
        }

        if let Some(val) = self.token_id_to {
            data.append(&mut Vec::from(val.as_slice()));
        }

        data.append(&mut Vec::from(amount_bytes(self.amount)));
        data.append(&mut Vec::from(self.name.as_bytes()));
        data.append(&mut Vec::from(self.symbol.as_bytes()));
        data.append(&mut Vec::from(self.uri.as_bytes()));
        data.append(&mut Vec::from(amount_bytes(self.decimals as u64)));
        data
    }
}


pub struct TransferOperation {
    // Empty line if is native
    pub address_to: Option<[u8; 32]>,
    // Empty line if is native or fungible
    pub token_id_to: Option<[u8; 32]>,
    pub amount: u64,
}

impl TransferOperation {
    pub fn new_native_transfer(amount: u64) -> Self {
        TransferOperation {
            address_to: None,
            token_id_to: None,
            amount,
        }
    }
}

impl Operation for TransferOperation {
    fn get_operation(&self) -> Vec<u8> {
        let mut data = Vec::new();

        if let Some(val) = self.address_to {
            data.append(&mut Vec::from(val.as_slice()));
        }

        if let Some(val) = self.token_id_to {
            data.append(&mut Vec::from(val.as_slice()));
        }

        data.append(&mut Vec::from(amount_bytes(self.amount)));
        data
    }
}

pub struct ContentNode {
    // Hash of deposit tx info. See spec in core for more information.
    pub origin: [u8; 32],
    // Solana
    pub network_to: String,
    pub receiver: [u8; 32],
    pub program_id: [u8; 32],
    pub data: Vec<u8>,
}

impl ContentNode {
    pub fn new(origin: [u8; 32], receiver: [u8; 32], program_id: [u8; 32], data: Vec<u8>) -> Self {
        ContentNode {
            origin,
            receiver,
            network_to: String::from(SOLANA_NETWORK),
            program_id,
            data,
        }
    }

    pub fn hash(self) -> solana_program::keccak::Hash {
        let mut data = Vec::new();
        data.append(&mut Vec::from(self.origin.as_slice()));
        data.append(&mut Vec::from(self.network_to.as_bytes()));
        data.append(&mut Vec::from(self.receiver.as_slice()));
        data.append(&mut Vec::from(self.program_id.as_slice()));
        data.append(&mut Vec::from(self.data));
        solana_program::keccak::hash(data.as_slice())
    }
}

fn amount_bytes(amount: u64) -> [u8; 32] {
    let bytes = amount.to_be_bytes();
    let mut result: [u8; 32] = [0; 32];

    for i in 0..bytes.len() {
        result[31 - i] = bytes[bytes.len() - 1 - i];
    }

    return result;
}