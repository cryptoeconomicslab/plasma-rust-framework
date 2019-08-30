# OVM Rust Framework

OVM client's Rust implementation.
This repositry must be conpatible with [ovm](https://github.com/plasma-group/ovm).

[![Build Status](https://travis-ci.org/cryptoeconomicslab/plasma-rust-framework.svg?branch=master)](https://travis-ci.org/cryptoeconomicslab/plasma-rust-framework)

## Overview

We are aiming to general L2 development framework based on Optimistic Virtual Machine.
The primary goal of this repository is to implement Rust client following OVM standard and to build specific L2 constructions such as Channel, Plasma and Optimistic rollup. Furthermore, this client enables more generalized applications on these constructions.

**This is an experimental software, does not run in a production yet.**

## Introduction

CryptoeconomicsLab have been researching and developping Plasma and application framework on it.
Now we are aiming to development framework for general second layer based on [Optimistic Virtual Machine](https://github.com/plasma-group/ovm).
As [our development direction](https://medium.com/cryptoeconomics-lab/cel-development-direction-to-the-greater-abstraction-6860f87ce0eb), this repository supports "Client" of second layer. We determined to use Rust from the point of code reuse and secure code.

## What we do in the repository

### OVM Client implementation

This repository don't include smart contract, but includes components below.

- The decision mechanism and core deciders
- Networking utilities for both Layer 1 and Layer 2
- Plasma and State Channel client implementation

### Multi platform

- Run on Linux and Mac
- Android Integration: https://github.com/cryptoeconomicslab/plasma-android-sdk
- Browser: TBD

### Smart contracts

OVM core smart contract(Universal Adjudicator contract), predicate and deposit contract is [here](https://github.com/cryptoeconomicslab/ovm-contracts).

## Development

### Test Source Code

Testing all crates.

```
cargo test --all
```

## About us

[Company Site](https://www.cryptoeconomicslab.com)
