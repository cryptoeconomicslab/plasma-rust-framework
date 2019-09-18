### Build

```
cd clients
cargo build --example plasma_client_cli
cargo build --example plasma_aggregator
```

### Run commitment contract

```
ganache-cli --mnemonic 'candy maple cake sugar pudding eam honey rich smooth crumble sweet treat'
```

```
git clone https://github.com/cryptoeconomicslab/ovm-contracts.git
cd ovm-contracts
npm i
./node_modules/.bin/truffle migrate
```

### Run plasma aggregator

Return to plasma-rust-client/clients.

```
mkdir .plasma_db
../target/debug/examples/plasma_aggregator
```

### Run plasma client

Initilize client storage

```
../target/debug/examples/plasma_client_cli  -f 627306090abab3a6e1400e9345bc60c78a8bef57 init
```

Get balance

```
../target/debug/examples/plasma_client_cli  -f 627306090abab3a6e1400e9345bc60c78a8bef57 balance
```

Send token

```
../target/debug/examples/plasma_client_cli  -f 627306090abab3a6e1400e9345bc60c78a8bef57 send -s 0 -e 5 -t f17f52151ebef6c7334fad080c5704d77216b732
```
