// CONSTANTS
pub const OWNER_PUBKEY: &str = "Ct9GFe2JSfucDJu7YPZ5dAbwANNkpKviiHxBAFx3Ypis";
pub const LAMPORTS_PER_SOL: u64 = 1_000_000_000;
pub const PROTOCOL_FEE_PERCENT: u64 = 50_000_000; // Hardcoded protocol fee percent in lamports
pub const SUBJECT_FEE_PERCENT: u64 = 50_000_000; // Hardcoded subject fee percent in lamports

// LAMPORTS TO SOL
pub fn lamports_to_sol(lamports: u64) -> f64 {
    lamports as f64 / 1_000_000_000_f64
}

// SOL TO LAMPORTS
pub fn sol_to_lamports(sol: f64) -> u64 {
    (sol * 1_000_000_000_f64) as u64
}

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
