# GG Smart Contracts

## Overview
GG is a Solana-based smart contract implementing a bonding curve mechanism for minting and trading player cards. The contract manages pot, protocol, mint, and token accounts to facilitate transactions within the ecosystem.

## How It Works
1. **Initialization:** The owner creates `PotAccount` and `ProtocolAccount` to store the bonding curve liquidity and protocol fees.
2. **Minting Player Cards:** Players (hunters) mint their first player card, which starts the bonding curve for their address and stores their fees.
3. **Buying and Selling Cards:** Sponsors buy player cards from the bonding curve, distributing fees between the protocol and mint accounts.
4. **Withdrawing Fees:** The protocol owner and players can withdraw funds from the protocol and mint accounts, respectively.

## Repository Structure
```
solana/
├── migrations/                # Deployment scripts
│   └── deploy.ts             # Anchor migration script
├── programs/
│   └── gg/                   # Smart contract program
│       ├── src/              # Source code
│       │   ├── error.rs      # Custom error definitions
│       │   ├── lib.rs        # Main program logic
│       │   ├── state.rs      # Account structures
│       │   └── utils.rs      # Utility functions
│       ├── Cargo.toml        # Rust package configuration
│       └── Xargo.toml        # Xargo configuration
├── scripts/                   # Utility scripts
│   ├── initialize.ts         # Script to initialize the contract
│   ├── mints.ts              # Script to list mint accounts
│   ├── protocolAddress.ts    # Fetch protocol account address
│   └── withdraw.ts           # Withdraw funds from protocol
├── tests/                     # Unit and integration tests
│   ├── functions.ts          # Helper functions for testing
│   ├── test.ts               # Main test suite
│   └── utils.ts              # Utility functions
├── .env                      # Environment variables
├── Anchor.toml               # Anchor configuration
├── Cargo.toml                # Rust workspace configuration
├── package.json              # Dependencies and scripts
├── tsconfig.json             # TypeScript configuration
└── README.md                 # This documentation
```

## Installation & Setup
### Prerequisites
Ensure you have the following installed:
- [Rust](https://www.rust-lang.org/)
- [Solana CLI](https://docs.solana.com/cli/install-solana-cli-tools)
- [Anchor](https://www.anchor-lang.com/docs/installation)
- [Node.js](https://nodejs.org/)

### Setup Instructions
1. Clone the repository:
   ```sh
   git clone https://github.com/mirage-ar/solana.git
   cd solana
   ```
2. Install dependencies:
   ```sh
   yarn install
   ```
3. Build and deploy the contract:
   ```sh
   anchor build && anchor deploy
   ```
4. Run tests:
   ```sh
   anchor test
   ```

## Smart Contract Functions
### Initialization
- `initialize`: Creates `PotAccount` and `ProtocolAccount`.
- `mint`: Mints a new player card and initializes the bonding curve.

### Transactions
- `buy_shares`: Sponsors buy shares from a player's bonding curve.
- `sell_shares`: Sponsors sell shares back to the bonding curve.
- `withdraw_from_protocol`: The protocol owner withdraws accumulated fees.
- `withdraw_from_mint`: Players withdraw their earnings.

## License
This project is licensed under the MIT License.

