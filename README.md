# GG CONTRACTS

## How it Works

1. Owner creates pot and protocol accounts to store bonding curve and fees
2. Players (hunters) "mint" the first player card which starts the curve for their address and stores fees
3. Sponsors buy cards from the pot bonding curve, fees are distributed to protocol and mint accounts

## Setup

1. Install rust, solana, and anchor
2. *Update owner address in gg/src/utils*
3. deploy with `anchor build && anchor deploy`
3. Run tests with `anchor test`