use crate::instruction::{TokenType, SignedContent};
use solana_program::pubkey::Pubkey;

const SOLANA_NETWORK: String = String::from("Solana");

pub struct ContentNode {
    pub tx_hash: String,
    // Empty line if was native
    pub address_from: String,
    // Empty line if was native or fungible
    pub token_id_from: String,

    // Empty line if is native
    pub address_to: String,
    // Empty line if is native or fungible
    pub token_id_to: String,

    pub receiver: String,

    pub network_from: String,
    // Solana
    pub network_to: String,
    pub amount: u64,
    pub token_type: TokenType,
}

impl ContentNode {
    pub(crate) fn new(content: &SignedContent, mint: String, collection: String, receiver: String) -> Self {
        ContentNode {
            tx_hash: content.tx_hash.clone(),
            address_from: content.address_from.clone(),
            token_id_from: content.token_id_from.clone(),
            address_to: collection,
            token_id_to: mint,
            receiver,
            network_from: content.network_from.clone(),
            network_to: SOLANA_NETWORK,
            amount: content.amount,
            token_type: content.token_type.clone(),
        }
    }
}

impl ToString for ContentNode {
    fn to_string(&self) -> String {
        let mut res = String::new();
        res.push_str(self.tx_hash.as_str());
        res.push_str(self.address_from.as_str());
        res.push_str(self.token_id_from.as_str());
        res.push_str(self.address_to.as_str());
        res.push_str(self.token_id_to.as_str());
        res.push_str(self.receiver.as_str());
        res.push_str(self.network_from.as_str());
        res.push_str(self.network_to.as_str());
        res.push_str(self.amount.to_string().as_str());
        res.push_str(self.token_type.to_string().as_str());
        res
    }
}