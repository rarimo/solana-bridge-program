use solana_program::entrypoint::ProgramResult;

use crate::error::BridgeError;
use crate::instruction::{DepositFTArgs, DepositNativeArgs, DepositNFTArgs, MintCollectionArgs, SignedMetadata, WithdrawArgs};
use crate::state::{MAX_ADDRESS_SIZE, MAX_NETWORKS_SIZE, MAX_TOKEN_ID_SIZE, MAX_TX_SIZE};

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

impl MintCollectionArgs {
    pub fn validate(&self) -> ProgramResult {
        self.data.validate()
    }
}

impl SignedMetadata {
    pub fn validate(&self) -> ProgramResult {
        if self.name.as_bytes().len() > mpl_token_metadata::state::MAX_NAME_LENGTH ||
            self.symbol.as_bytes().len() > mpl_token_metadata::state::MAX_SYMBOL_LENGTH ||
            self.uri.as_bytes().len() > mpl_token_metadata::state::MAX_URI_LENGTH {
            return Err(BridgeError::WrongArgsSize.into());
        }

        Ok(())
    }
}