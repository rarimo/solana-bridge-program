use crate::instruction::{DepositFTArgs, DepositNativeArgs, DepositNFTArgs, WithdrawArgs};
use solana_program::entrypoint::ProgramResult;
use crate::state::{MAX_ADDRESS_SIZE, MAX_NETWORKS_SIZE};
use crate::error::BridgeError;
use crate::util;

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
        if self.sender_address.as_bytes().len() > MAX_ADDRESS_SIZE || self.network_from.as_bytes().len() > MAX_NETWORKS_SIZE {
            return Err(BridgeError::WrongArgsSize.into());
        }

        util::validate_option_str(&self.token_id, MAX_ADDRESS_SIZE)?;
        Ok(())
    }
}