use std::hash::Hash;

use solana_program::{
    msg,
    pubkey::Pubkey,
};
use lib::merkle::{amount_bytes, Data};
use crate::state::{CommissionToken, OperationType};

const SOLANA_NATIVE_DECIMALS: u8 = 9u8;


pub struct CommissionTokenData {
    pub operation_type: OperationType,
    pub token: CommissionToken,
}

impl CommissionTokenData {
    pub fn new_data(operation_type: OperationType, token: CommissionToken) -> Self {
        CommissionTokenData {
            operation_type,
            token,
        }
    }
}

impl Data for CommissionTokenData {
    fn get_operation(&self) -> Vec<u8> {
        let mut data = Vec::new();

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
        data
    }
}