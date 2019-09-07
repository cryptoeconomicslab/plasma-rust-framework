# OVM Rust Framework

OVM client's Rust implementation.
This repositry must be compatible with [ovm](https://github.com/plasma-group/ovm).

[![Build Status](https://travis-ci.org/cryptoeconomicslab/plasma-rust-framework.svg?branch=master)](https://travis-ci.org/cryptoeconomicslab/plasma-rust-framework)

## Overview

We are aiming to general L2 development framework based on Optimistic Virtual Machine.
The primary goal of this repository is to implement Rust client following OVM standard and to build specific L2 constructions such as Channel, Plasma and Optimistic rollup. Furthermore, this client enables more generalized applications on these constructions.

**This is an experimental software, does not run in a production yet.**

## Introduction

CryptoeconomicsLab has been researching and developping Plasma and application framework on it.
Now we are aiming to develop a general-purpose framework for the second layer based on [Optimistic Virtual Machine](https://github.com/plasma-group/ovm).
As [our development direction](https://medium.com/cryptoeconomics-lab/cel-development-direction-to-the-greater-abstraction-6860f87ce0eb) illustrates here, this repository is to support the Client part of the second layer. We decided to use the Rust language since it will enable us to write secure and reusable source codes for multiple platforms.

## What we do in the repository

### OVM Client implementation

This repository don't include smart contract, but includes components below.

- The decision mechanism and core deciders
- Networking utilities for both Layer 1 and Layer 2
- Plasma and State Channel client implementation

### Multiplatform

- Run on Linux and Mac
- Android Integration: https://github.com/cryptoeconomicslab/plasma-android-sdk
- Browser: TBD

### Smart contracts

OVM core smart contract(Universal Adjudication Contract), Predicates, and Deposit And Exit Contract are [here](https://github.com/cryptoeconomicslab/ovm-contracts).

## Development

### Test Source Code

Testing all crates.

```
cargo test --all
```

## About us

[Corporate Website](https://www.cryptoeconomicslab.com)
[Medium](https://medium.com/cryptoeconomics-lab)
