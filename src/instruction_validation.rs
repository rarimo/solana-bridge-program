use crate::instruction::{DepositFTArgs, DepositNativeArgs, DepositNFTArgs, WithdrawArgs, MintNFTArgs, MintFTArgs};
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
        if self.amount <= 0 {
            return Err(BridgeError::WrongArgsSize.into());
        }

        Ok(())
    }
}

impl MintFTArgs {
    pub fn validate(&self) -> ProgramResult {
        if self.amount <= 0 || self.decimals <= 0 {
            return Err(BridgeError::WrongArgsSize.into());
        }
        Ok(())
    }
}

impl MintNFTArgs {
    pub fn validate(&self) -> ProgramResult {
        Ok(())
    }
}
