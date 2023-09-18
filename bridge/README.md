# Rarimo Bridge program

Rarimo bridge program implements entrypoints and logic for Rarimo bridging on Solana.

All instructions and common methods are defined in the [Lib sub-crate](../lib/src/instructions).

That smart-contract exposes the following methods:

- `process_init_admin(program_id, accounts, args.seeds, args.public_key, args.commission_program)`

    Initialization of Bridge admin entry that will store information about commission smart contract and public key.
    Also will hold all deposited tokens and liquidity pool. 
    Created account will be `PDA(provided_seed, program_id)` so only bridge program can sign instructions from its name.
  

- `process_transfer_ownership(program_id, accounts, args.seeds, args.new_public_key, args.signature, args.recovery_id)`
  
    Change public key that should sign withdrawal and management operations. 
    Requires the signature for new public key bytes by old public key.
  

- `process_deposit_native(program_id, accounts, args.seeds, args.network_to, args.receiver_address, args.amount)`
  
    Handler for native `Sol` token deposit. Verifies that commission was charged and then performs token transfer.
  

- `process_deposit_ft(program_id, accounts, args.seeds, args.network_to, args.receiver_address, args.amount, args.token_seed)`
  
    Handler for fungible token deposit. Verifies that commission was charged and then performs token transfer.
  

- `process_deposit_nft(program_id, accounts, args.seeds, args.network_to, args.receiver_address, args.token_seed)`
  
    Handler for non-fungible token deposit. Verifies that commission was charged and then performs token transfer.
  

- `process_withdraw_native(program_id, accounts, args.seeds, args.signature, args.recovery_id, args.path, args.origin, args.amount)`
  
    Handler for the native `Sol` token withdrawal. Verifies the provided signature and data, after - performs token transfer.
  

- `process_withdraw_ft(program_id, accounts, args.seeds, args.signature, args.recovery_id, args.path, args.origin, args.amount, args.token_seed, args.signed_meta)`
  
    Handler for the fungible token withdrawal. Verifies the provided signature and data, after - performs token transfer.
  

- `process_withdraw_nft(program_id, accounts, args.seeds, args.signature, args.recovery_id, args.path, args.origin, args.token_seed, args.signed_meta)`
  
    Handler for the non-fungible token withdrawal. Verifies the provided signature and data, after - performs token transfer.
  

- `process_create_collection(program_id, accounts, args.seeds, args.data, args.token_seed)`
  
    Creates a collection with bridge admin owner. Used to create collections for wrapped NFTs. 

---

Also, lets describe more precisely the logic of commission verification:

The `verify_commission_charged` method checks the previous instruction - it should exists and should be the 
`ChargeCommission` instruction to the stored commission program address. 

```rust
pub fn verify_commission_charged<'a>(
    bridge_admin_info: &AccountInfo<'a>,
    instruction_sysvar_info: &AccountInfo<'a>,
    admin: &BridgeAdmin,
    token: lib::TokenType,
    amount: u64,
) -> ProgramResult {
    let current_index = load_current_index_checked(instruction_sysvar_info)?;
    let commission_instruction = load_instruction_at_checked((current_index - 1) as usize, instruction_sysvar_info)?;

    if commission_instruction.program_id != admin.commission_program {
        return Err(LibError::WrongCommissionProgram.into());
    }

    let commission_key = Pubkey::create_program_address(&[lib::COMMISSION_ADMIN_PDA_SEED.as_bytes(), bridge_admin_info.key.as_ref()], &commission_instruction.program_id)?;
    if commission_key != commission_instruction.accounts[0].pubkey {
        return Err(LibError::WrongCommissionAccount.into());
    }

    let instruction = lib::instructions::commission::CommissionInstruction::try_from_slice(commission_instruction.data.as_slice())?;

    if let lib::instructions::commission::CommissionInstruction::ChargeCommission(args) = instruction {
        if args.deposit_token == token && args.deposit_token_amount == amount {
            return Ok(());
        }
    }

    return Err(LibError::WrongCommissionArguments.into());
}
```