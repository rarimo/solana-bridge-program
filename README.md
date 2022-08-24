# Solana bridge contract

## Fundamentals
Solana decentralized bridge contract will manage deposits and withdrawals for Native (Sol), FT and NFT. 
All withdrawal operations are protected by ECDSA secp256k1 threshold (t-n) signature. 
This signature is produced by core multi-sig services depending on validated core state.   

## Requirements
- Admin account should store ECDSA t-n public key
- Changing of stored admin's key requires t-n signature of new key produced by old key
- Withdrawal operations should accept Merkle node content, Merkle path and admin t-n signature for Merkle root.
- Withdrawal operations should perform check for Merkle path, signature and other sensitive stuff

Note that in smart contract we are using 64-byte public key format.

## State definitions
Here we define the token type on contract level - for tracking in state what kind of operation has been called.
```rust
pub enum TokenType {
    Native,
    NFT,
    FT,
}
```

The BridgeAdmin account stores ECDSA t-n public key 
```rust
pub struct BridgeAdmin {
    pub public_key: [u8; SECP256K1_PUBLIC_KEY_LENGTH],
    pub is_initialized: bool,
}
```

Deposit account stores information about successfully processed deposit operation
```rust
pub struct Deposit {
    pub token_type: TokenType,
    // Can be None for native token
    pub mint: Option<Pubkey>,
    pub amount: u64,
    // Network to (target)
    pub network: String,
    // Receiver address on target network
    pub receiver_address: String,
    pub is_initialized: bool,
}
```

Withdraw account stores information about successfully processed withdrawal operation
```rust
pub struct Withdraw {
    pub token_type: TokenType,
   // Can be None for native token
    pub mint: Option<Pubkey>,
    pub amount: u64,
    // Network from
    pub network: String,
    // Receiver address on Solana
    pub receiver_address: Pubkey,
    pub is_initialized: bool,
}
```

## Instructions

For quick instructions overview take a look on [instructions.rs](./src/instruction.rs) 

Here we have deposits and withdrawal instructions split onto three groups: native, ft (fungible token) and nft (non-fungible token).
Note that deposit instructions accepts all tokens, and if you try to send token that are not supported by our core system you will lose all of them.
Also withdrawal operations does not take any care of matching tokens between chains. 
Valid Merkle path + admins signature means that data is also valid and signed by core system.

## Build

```commandline
 npm run build:program-rust
```

## Deploy
```commandline
solana program deploy --program-id ./dist/program/bridge-keypair.json ./dist/program/bridge.so
```

