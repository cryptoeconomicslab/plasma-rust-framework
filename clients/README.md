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
../target/debug/examples/plasma_client_cli -ss init
```

```
../target/debug/examples/plasma_client_cli -ss import -s c87509a1c067bbde78beb793e6fa76530b6382a4c0241e5e4a9ec0a0f44dc0d3
> session: {session}
```

Get balance

```
../target/debug/examples/plasma_client_cli  -s {session} balance
```

Send token

```
../target/debug/examples/plasma_client_cli  -s {session} send -s 0 -e 5 -t f17f52151ebef6c7334fad080c5704d77216b732
```
