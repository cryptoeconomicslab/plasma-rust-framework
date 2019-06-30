# Plasma Rust Framework
[Draft] Plasma Chamber's Rust implemenation.
Plasma Chamber is now a compliant of Plasma Core.
This repositry must be conpatible with [pigi](https://github.com/plasma-group/pigi).


[![Build Status](https://travis-ci.org/cryptoeconomicslab/plasma-rust-framework.svg?branch=master)](https://travis-ci.org/cryptoeconomicslab/plasma-rust-framework)


## Overview
- Gradually try shifting from JS to Rust
- For browser and NodeJS: Publish WASM to WAPM, and publish generated wrapper & @types to NPM
- For App: Compile rust to each target environment


## Build Source Code

```
cargo build --release
```

### Run Manually

```
./target/release/plasma-chamber 
```
