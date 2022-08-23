use crate::instruction::{DepositFTArgs, DepositNativeArgs, DepositNFTArgs, WithdrawArgs, SignedContent};
use solana_program::entrypoint::ProgramResult;
use crate::state::{MAX_ADDRESS_SIZE, MAX_NETWORKS_SIZE, MAX_TOKEN_ID_SIZE, MAX_TX_SIZE};
use crate::error::BridgeError;

impl DepositNativeArgs {
    pub fn validate(&self) -> ProgramResult {
        if self.receiver_address.as_bytes().len() > MAX_ADDRESS_SIZE ||
            self.network_to.as_bytes().len() > MAX_NETWORKS_SIZE || self.amount <= 0 {
            return Err(BridgeError::WrongArgsSize.into());
        }

        Ok(())
    }
}

impl DepositFTArgs {
    pub fn validate(&self) -> ProgramResult {
        if self.receiver_address.as_bytes().len() > MAX_ADDRESS_SIZE ||
            self.network_to.as_bytes().len() > MAX_NETWORKS_SIZE || self.amount <= 0 {
            return Err(BridgeError::WrongArgsSize.into());
        }

        Ok(())
    }
}

impl DepositNFTArgs {
    pub fn validate(&self) -> ProgramResult {
        if self.receiver_address.as_bytes().len() > MAX_ADDRESS_SIZE || self.network_to.as_bytes().len() > MAX_NETWORKS_SIZE {
            return Err(BridgeError::WrongArgsSize.into());
        }

        Ok(())
    }
}

impl WithdrawArgs {
    pub fn validate(&self) -> ProgramResult {
        self.content.validate()
    }
}

impl SignedContent {
    pub fn validate(&self) -> ProgramResult {
        if self.network_from.as_bytes().len() > MAX_NETWORKS_SIZE ||
            self.address_from.as_bytes().len() >= MAX_ADDRESS_SIZE ||
            self.token_id_from.as_bytes().len() >= MAX_TOKEN_ID_SIZE ||
            self.amount <= 0 || self.tx_hash.as_bytes().len() >= MAX_TX_SIZE {
            return Err(BridgeError::WrongArgsSize.into());
        }

        Ok(())
    }
}
/*
impl MintArgs {
    pub fn validate(&self) -> ProgramResult {
        util::validate_option_str(&self.token_id, MAX_ADDRESS_SIZE)?;
        util::validate_option_str(&self.address, MAX_ADDRESS_SIZE)?;
        Ok(())
    }
}*/
