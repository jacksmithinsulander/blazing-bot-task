# Instructions

in bot/ create a .env file with these values (PKEY = private key):

```sh
PKEY_ONE=
PKEY_TWO=
PKEY_THREE=
PKEY_FOUR=

PUBKEY_ONE=
```

If you want to deploy and mint test tokens you can create the same in contracts/
but with these added fields: 

```sh
SEPOLIA_API_KEY=
SEPOLIA_RPC=https://eth-sepolia.public.blastapi.io
```

then run the following command

```sh
forge script script/MockToken.s.sol --broadcast --legacy --slow --rpc-url sepolia --verify
```

after that the PUBKEY_ONE (which should be the public key of PKEY_ONE) is funded with newly minted test tokens

then, in the terminal cd into /bots and run `cargo run`

after that you can use the following apis for testing:

## Disperse token
```sh
curl -X POST http://127.0.0.1:8080/disperse_token \ 
-H "Content-Type: application/json" \
-d '{
  "token": "0xTOKENADDRESS",
  "wallets": ["0xWALLETTWO", "0xWALLETTHREE", "WALLETFOUR"],
  "amount": "5000",
  "percentage": true
}'
```

## Collect token
```sh
curl -X POST http://127.0.0.1:8080/collect_token \
-H "Content-Type: application/json" \
-d '{
  "address_with_amount": [
    {"address": "0xWALLETTWO", "amount": "1000"}, 
    {"address": "0xWALLETTHREE", "amount": "5000"},
    {"address": "0xWALLETFOUR", "amount": "10000"}
  ],
  "token": "0xTOKENADDRESS",
  "to": "0xWALLETONE",
  "percentage": true
}'
```

## Disperse Eth
```sh
bankroll-contracts % curl -X POST http://127.0.0.1:8080/disperse_eth \
-H "Content-Type: application/json" \
-d '{
  "wallets": [
    "0xWALLETTWO",
    "0xWALLETTHREE",
    "0xWALLETFOUR"
  ],
  "amount": "100000000000000",
  "percentage": false
}'
```
