# Tesserax Protocol

<div align="center">

**Adaptive Scarcity & Quantum-Resistant Blockchain**

[![License](https://img.shields.io/badge/license-MIT--0-blue.svg)](LICENSE)
[![Built with Substrate](https://img.shields.io/badge/Built%20with-Substrate-e6007a)](https://substrate.io/)

</div>

## Overview

Tesserax Protocol is a next-generation blockchain built on Substrate, featuring:

- **🔢 Adaptive Scarcity Mechanism (ASM)** - Supply controlled by universal constants (π, e, φ)
- **🔐 Quantum-Resistant Cryptography** - Hybrid signature scheme (ECDSA + ML-DSA/Dilithium)
- **⚡ BABE + GRANDPA Consensus** - Fast block production with deterministic finality
- **📈 Sigmoid Emission Curve** - Natural growth pattern instead of harsh halving
- **🔗 Full EVM Compatibility** - Deploy Solidity contracts seamlessly

## The Tesserax Constant

The maximum supply of $TSRX is derived from universal mathematical constants:

$$S_{max} = \lfloor \pi \times e \times \phi \times 10^6 \rfloor = 13,817,422 \text{ TSRX}$$

Where:
- **π** (Pi) ≈ 3.14159... - Represents cycles
- **e** (Euler's number) ≈ 2.71828... - Represents growth  
- **φ** (Golden Ratio) ≈ 1.61803... - Represents proportion

## Quick Start

### Prerequisites

- Rust (stable)
- Protobuf compiler

### Build

```bash
git clone https://github.com/tesserax-protocol/tesserax-node.git
cd tesserax-node
cargo build --release
```

### Run Development Node

```bash
./target/release/tesserax-node --dev
```

### Run with Detailed Logging

```bash
RUST_BACKTRACE=1 ./target/release/tesserax-node -ldebug --dev
```

### Purge Development Chain

```bash
./target/release/tesserax-node purge-chain --dev
```

## Architecture

```
tesserax-node/
├── node/           # Node client implementation
├── pallets/        # Custom FRAME pallets
│   ├── template/   # Example pallet (to be replaced with Tesserax pallets)
│   └── ...
├── runtime/        # Runtime configuration
└── yellow-paper.md # Technical specification
```

## Documentation

- [Yellow Paper](./yellow-paper.md) - Complete technical specification

## Pallets (Planned)

- [ ] `pallet-tokenomics` - Sigmoid emission & ASM implementation
- [ ] `pallet-sentinel` - Quantum signature validation & intrusion detection
- [ ] `pallet-quantum-registry` - PQC key management & upgrade protocol
- [ ] Custom Precompiles for EVM integration

## Connect to Polkadot.js Apps

Once your node is running, connect to it using [Polkadot.js Apps](https://polkadot.js.org/apps/?rpc=ws%3A%2F%2F127.0.0.1%3A9944#/explorer).

## License

MIT-0

---

<div align="center">

**"Mathematics-as-Money"**

*Where supply meets the universal constants*

</div>
