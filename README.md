# Rarimo Solana Bridge programs

That repository stores all contracts related to the Rarimo bridge on Solana. 

Currently they are described in the following sub-crates:

- [Bridge](./bridge/program) - Rarimo Bridge program 
- [Commission](./bridge/program) - Rarimo Bridge commission program.
- [Lib](./lib) - Rarimo bridge library

Check the [dev-docs](https://rarimo.gitlab.io/dev-docs/docs/developers/contracts) for more information about how bridge works.

## Build

```shell
npm run build:bridge
npm run build:commission
npm run build:upgrade
```

## Deploy
```shell
solana program deploy --program-id ./dist/program/bridge-keypair.json ./dist/program/bridge.so
solana program deploy --program-id ./dist/program/commission-keypair.json ./dist/program/commission.so
solana program deploy --program-id ./dist/program/upgrade-keypair.json ./dist/program/upgrade.so
```

