# Rarimo Solana bridge contracts

## Build

```shell
npm run build:bridge
npm run build:commission
```

## Deploy
```shell
solana program deploy --program-id ./dist/program/bridge-keypair.json ./dist/program/bridge.so
solana program deploy --program-id ./dist/program/commission-keypair.json ./dist/program/commission.so
```

