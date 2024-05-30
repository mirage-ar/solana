// CONSTANTS
// JON: decoding base58 strings on-chain can take up a lot of compute,
// so you're better off using `declare_id!` to do this before program
// compilation
pub mod owner {
    use anchor_lang::prelude::*;
    declare_id!("Ct9GFe2JSfucDJu7YPZ5dAbwANNkpKviiHxBAFx3Ypis");
}
// JON: feel free to use solana_program::native_token::LAMPORTS_PER_SOL
pub const LAMPORTS_PER_SOL: u64 = 1_000_000_000;
pub const PROTOCOL_FEE_PERCENT: u64 = 50_000_000; // Hardcoded protocol fee percent in lamports
pub const SUBJECT_FEE_PERCENT: u64 = 50_000_000; // Hardcoded subject fee percent in lamports

// JON: feel free to use solana_program::native_token::lamports_to_sol
// LAMPORTS TO SOL
pub fn lamports_to_sol(lamports: u64) -> f64 {
    lamports as f64 / 1_000_000_000_f64
}

// JON: feel free to use solana_program::native_token::sol_to_lamports
// SOL TO LAMPORTS
pub fn sol_to_lamports(sol: f64) -> u64 {
    (sol * 1_000_000_000_f64) as u64
}

// JON: !!!IMPORTANT!!! Use checked math here to avoid overflowing and making it
// possible to exploit. To be safe, you can use `u128` to do the math and then
// convert back, but there's still a possibility to overflow since you're doing
// (supply + amount) ^ 3, which could potentially use up to `u192`, which doesn't
// exist.
pub fn get_price(supply: u64, amount: u64) -> u64 {
    let factor = 1600;

    let sum1 = if supply == 0 {
        0
    } else {
        (supply - 1) * supply * (2 * (supply - 1) + 1) / 6
    };

    let sum2 = if supply == 0 && amount == 1 {
        0
    } else {
        (supply - 1 + amount) * (supply + amount) * (2 * (supply - 1 + amount) + 1) / 6
    };

    let summation = sum2 - sum1;
    (summation * LAMPORTS_PER_SOL) / factor
}

pub fn get_buy_price(shares_supply: u64, amount: u64) -> u64 {
    get_price(shares_supply, amount)
}

pub fn get_sell_price(shares_supply: u64, amount: u64) -> u64 {
    get_price(shares_supply - amount, amount)
}

#[cfg(test)]
mod tests{
    use {super::*, proptest::prelude::*};

    // JON: Be sure you can get this test to pass, or at least return an error
    // instead of overflowing. Otherwise, there's a chance for an exploit. I
    // don't have a whole attack, but overflow is regularly exploitable.
    proptest! {
        #[test]
        fn check_no_steal(supply in 1..u64::MAX, amount in 0..u64::MAX) {
            assert_eq!(get_buy_price(supply, amount), get_sell_price(supply + amount, amount));
        }
    }
}
