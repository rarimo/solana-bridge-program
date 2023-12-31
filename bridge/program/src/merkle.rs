use std::hash::Hash;

use solana_program::{
    msg,
    pubkey::Pubkey,
};

use lib::merkle::{amount_bytes};
use lib::SOLANA_NETWORK;

const SOLANA_NATIVE_DECIMALS: u8 = 9u8;

pub trait Data {
    fn get_operation(&self) -> Vec<u8>;
}

pub struct Content {
    pub origin: [u8; 32],
    pub network_to: String,
    pub receiver: [u8;32],
    pub program_id: [u8; 32],
    pub data: Vec<u8>,
}

impl Content {
    pub fn new(origin: [u8; 32], receiver: [u8;32], program_id: [u8; 32], data: Box<dyn Data>) -> Self {
        Content {
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

pub struct TransferData {
    // Empty line if is native
    pub address_to: Option<[u8; 32]>,
    // Empty line if is native or fungible
    pub token_id_to: Option<[u8; 32]>,
    pub amount: Option<u64>,
    pub name: Option<String>,
    pub symbol: Option<String>,
    pub uri: Option<String>,
    pub decimals: Option<u8>,
}

impl TransferData {
    pub fn new_ft_transfer(mint: [u8; 32], amount: u64, name: String, symbol: String, uri: String, decimals: u8) -> Self {
        TransferData {
            address_to: Some(mint),
            token_id_to: None,
            amount: Some(amount),
            name: Some(name),
            symbol: Some(symbol),
            uri: Some(uri),
            decimals: Some(decimals),
        }
    }

    pub fn new_nft_transfer(mint: [u8; 32], collection: Option<[u8; 32]>, name: String, symbol: String, uri: String) -> Self {
        TransferData {
            address_to: collection,
            token_id_to: Some(mint),
            amount: None,
            name: Some(name),
            symbol: Some(symbol),
            uri: Some(uri),
            decimals: None,
        }
    }

    pub fn new_native_transfer(amount: u64) -> Self {
        TransferData {
            amount: Some(amount),
            address_to: None,
            token_id_to: None,
            name: None,
            symbol: None,
            uri: None,
            decimals: None,
        }
    }
}

impl Data for TransferData {
    fn get_operation(&self) -> Vec<u8> {
        let mut data = Vec::new();

        if let Some(val) = self.address_to {
            data.append(&mut Vec::from(val.as_slice()));
        }

        if let Some(val) = &self.name {
            data.append(&mut Vec::from(val.as_bytes()));
        }

        if let Some(val) = self.token_id_to {
            data.append(&mut Vec::from(val.as_slice()));
        }

        if let Some(val) = &self.uri {
            data.append(&mut Vec::from(val.as_bytes()));
        }

        if let Some(val) = self.amount {
            data.append(&mut Vec::from(amount_bytes(val)));
        }

        if let Some(val) = &self.symbol {
            data.append(&mut Vec::from(val.as_bytes()));
        }

        if let Some(val) = self.decimals {
            data.push(val);
        }

        data
    }
}