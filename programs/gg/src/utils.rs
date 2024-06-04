use anchor_lang::solana_program::native_token::LAMPORTS_PER_SOL;

// CONSTANTS
pub mod owner {
    use anchor_lang::prelude::*;
    declare_id!("Ct9GFe2JSfucDJu7YPZ5dAbwANNkpKviiHxBAFx3Ypis");
}

pub const PROTOCOL_FEE_PERCENT: u64 = 50_000_000; // Hardcoded protocol fee percent in lamports
pub const SUBJECT_FEE_PERCENT: u64 = 50_000_000; // Hardcoded subject fee percent in lamports

fn get_price(supply: u64, amount: u64) -> Option<u64> {
    let factor = 1600_u128;
    let amount = amount as u128;
    let supply = supply as u128;

    let sum1 = if supply == 0 {
        0_u128
    } else {
        supply
            .checked_sub(1)?
            .checked_mul(supply)?
            .checked_mul(supply.checked_sub(1)?.checked_mul(2)?.checked_add(1)?)?
            .checked_div(6)?
    };

    let sum2 = if supply == 0 && amount == 1 {
        0_u128
    } else {
        let supply_amount = supply.checked_add(amount)?;
        supply_amount
            .checked_sub(1)?
            .checked_mul(supply_amount)?
            .checked_mul(
                supply_amount
                    .checked_sub(1)?
                    .checked_mul(2)?
                    .checked_add(1)?,
            )?
            .checked_div(6)?
    };

    let summation = sum2.checked_sub(sum1)?;
    u64::try_from(
        summation
            .checked_mul(LAMPORTS_PER_SOL as u128)?
            .checked_div(factor)?,
    )
    .ok()
}

pub fn get_buy_price(shares_supply: u64, amount: u64) -> Option<u64> {
    get_price(shares_supply, amount)
}

pub fn get_sell_price(shares_supply: u64, amount: u64) -> Option<u64> {
    get_price(shares_supply.checked_sub(amount)?, amount)
}

// the get_price function with the overflow check removed - can be removed
// pub fn get_price(supply: u64, amount: u64) -> u64 {
//     let factor = 1600_u128;

//     let sum1 = if supply == 0 {
//         0_u128
//     } else {
//         let supply_u128 = supply as u128;
//         (supply_u128 - 1) * supply_u128 * (2 * (supply_u128 - 1) + 1) / 6
//     };

//     let sum2 = if supply == 0 && amount == 1 {
//         0_u128
//     } else {
//         let supply_amount_u128 = (supply + amount) as u128;
//         (supply_amount_u128 - 1) * supply_amount_u128 * (2 * (supply_amount_u128 - 1) + 1) / 6
//     };

//     let summation = sum2.checked_sub(sum1).expect("Overflow detected");
//     (summation * LAMPORTS_PER_SOL as u128 / factor) as u64
// }

#[cfg(test)]
mod tests {
    use {super::*, proptest::prelude::*};

    prop_compose! {
        pub fn sum_within(max_value: u64)(total in 1..max_value)
                        (intermediate in 1..total, total in Just(total))
                        -> (u64, u64) {
           (total - intermediate, intermediate)
       }
    }
    proptest! {
        #[test]
        fn check_no_steal((supply, amount) in sum_within(u64::MAX)) {
            let buy_price = get_buy_price(supply, amount);
            let sell_price = get_sell_price(supply + amount, amount);
            prop_assert_eq!(buy_price, sell_price);
        }
    }
}
