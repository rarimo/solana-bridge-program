# Rarimo Commission program

Rarimo commission program implements entrypoints and logic for charging commission from users 
before using Rarimo bridge.

All instructions and common methods are defined in the [Lib sub-crate](../lib/src/instructions).

That smart-contract exposes the following methods:

- `process_init_admin(program_id, accounts, args.acceptable_tokens)`

    Initialization of Commission admin entry that will store information about acceptable tokens and hold all charged tokens.
    Created account will be `PDA(["commission_admin".bytes(), Bridge admin key], program_id)` so only commission program can sign instructions from its name.


- `process_charge_commission(program_id, accounts, args.token)`

    Handler for charging commission in different types of tokens. 
    The list of required accounts is different and depends on charged token type.
  

- `process_add_token(program_id, accounts, args.signature, args.recovery_id, args.path, args.token)`
  
    Handler for adding new acceptable commission token. Requires valid signature for the provided data.


- `process_remove_token(program_id, accounts, args.signature, args.recovery_id, args.path, args.token)`

    Handler for removing acceptable commission token. Requires valid signature for the provided data.


- `process_update_token(program_id, accounts, args.signature, args.recovery_id, args.path, args.token)`

    Handler for updating acceptable commission token (changing of amount). Requires valid signature for the provided data.


- `process_withdraw(program_id, accounts,  args.signature, args.recovery_id, args.path, args.token, args.withdraw_amount)`

    Handler for withdrawal of collected tokens. Requires valid signature for the provided data.
