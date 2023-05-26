use std::hash::Hash;

use solana_program::{
    msg,
    pubkey::Pubkey,
};
use lib::merkle::amount_bytes;
use crate::state::{CommissionToken, OperationType};
use lib::SOLANA_NETWORK;

const SOLANA_NATIVE_DECIMALS: u8 = 9u8;

pub struct Content {
    pub nonce: u64,
    pub receiver: Option<Pubkey>,
    pub contract: Pubkey,
    pub network: String,
    pub operation_type: OperationType,
    pub token: CommissionToken,
}

impl Content {
    pub fn new(nonce: u64, receiver: Option<Pubkey>, contract: Pubkey, operation_type: OperationType, token: CommissionToken) -> Self {
        Content {
            nonce,
            receiver,
            contract,
            network: String::from(SOLANA_NETWORK),
            operation_type,
            token
        }
    }

    pub fn hash(self) -> solana_program::keccak::Hash {
        let mut data = Vec::new();
        data.append(&mut Vec::from(amount_bytes(self.nonce)));

        if let Some(receiver) = self.receiver {
            data.append(&mut Vec::from(receiver.as_ref()));
        }

        data.append(&mut Vec::from(self.contract.as_ref()));

        data.append(&mut Vec::from(self.network.as_bytes()));

        data.push(self.operation_type.clone().into());

        match self.token.token {
            lib::CommissionToken::Native => {
                // Nothing to add
            }
            lib::CommissionToken::FT(mint) => {
                data.append(&mut Vec::from(mint.to_bytes()))
            }
            lib::CommissionToken::NFT(mint) => {
                data.append(&mut Vec::from(mint.to_bytes()))
            }
        }

        data.append(&mut Vec::from(amount_bytes(self.token.amount)));

        solana_program::keccak::hash(data.as_slice())
    }
}