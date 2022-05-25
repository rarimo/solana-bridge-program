# Solana bridge contract

## Fundamentals
Solana bridge contract will manage all crosschain transfers for Solana Metaplex NFT. It will work in couple with a backend Solana proxy service that will create and sign withdrawal transactions and also store deposit/withdrawal history.

## Requirements
- Withdraw operations should be accessible only with the admin signature
- Bridge contract should be responsible for minting a new token or unlocking an existing one during withdrawal
- Bridge contract should be responsible for preventing ‘double-withdrawal’ attac
- There should be an option to change contract admin that should sign withdrawal operations

Let’s clarify that the contract is not responsible for managing what mint account we will use in withdrawal operation. Cause of Solana contracts peculiarities we cannot manage token mint accounts and collect information about its deposits, so the backend service should manage all mint and metadata creation/selection.

## State:
- BridgeAdmin:
    - admin public key
  
- DepositData
    - network
    - receiver address

- WithdrawData
    - network
    - sender address

## Methods:
Here we will use the next terms:

- __bridge token account__ - the token associated account of bridge admin account. Using the Solana PDA mechanics we can secure the account from all operations except of calls from our contract.

- __bridge admin account__ - account that represents contract admin and stores admin public key. Its account address can be derived from PDA of seed and program id.

- __seed__ - seed is the 32-byte array that derives the bridge admin account. The backend service is responsible for seed storing and correctness (cause PDA should not lie on ed25519 curve some seeds can produce wrong addresses).

- __nonce__ - nonce is the 32-byte array that derives deposit/withdrawal data stored on-chain. It helps us to create unique public keys, but because of Solana peculiarities we should calculate the bump seed for the key and then re-create public key using it.

More documentation about PDA, bump seed, etc. you can find [here](https://docs.rs/solana-program/latest/solana_program/pubkey/struct.Pubkey.html#method.find_program_address)

1. initAdmin

    __Logic__: 	Here we will initialize account data with admin’s public key


2. transferOwnership

    __Arguments:__ bridge admin account, admin account, new admin account

    __Logic:__ 	Here we will change admin in the bridge admin account data

3. depositMetaplexNFT

    __[Arguments](./src/instruction.rs):__
    - bridge admin account,
    - token mint account,
    - owner token account,
    - bridge token account,
    - owner account,
    - deposit data account,
    - data(network, receiver address, seed, nonce)

    __Logic:__ Here we will transfer NFT to the program’s token account (owned by bridge) and store data about it.


4. withdrawMetaplexNFT

    __[Arguments](./src/instruction.rs):__
    - bridge admin account,
    - token mint account,
    - owner token account,
    - bridge token account,
    - withdraw data account,  
    - admin account,
    - data(deposit tx id, network from, sender address, seeds)

    __Logic:__ Here we will check that:
    1. admin account is signed and bridge admin is equal to the signed admin
    2. program token account is owned by bridge
    3. withdrawal account is not initialized.

    After all checks, we will transfer token to the user’s account, and initialize withdrawal account information. Nonce for withdrawal account public key will be derived as sha256 hash from tx string.
