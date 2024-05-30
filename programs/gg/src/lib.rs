use anchor_lang::prelude::*;
use anchor_lang::solana_program::program::invoke;
use anchor_lang::solana_program::system_instruction;
use anchor_lang::solana_program::sysvar::rent::Rent;
use std::str::FromStr;

pub mod error;
pub mod state;
pub mod utils;

use crate::{error::*, state::*, utils::*};

declare_id!("4NZwzHq6bS1LqhUPPr7LjDz5aV18CYugg6PSx6GBXgDe");

#[program]
pub mod gg {
    use super::*;

    // JON: As mentioned earlier, you could completely remove this if you don't
    // store the authority on the accounts, and always check against `OWNER_PUBKEY`
    // directly.
    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        // Check if the caller is the owner
        let authority = &mut ctx.accounts.authority;

        // JON: decoding base58 strings on-chain can take up a lot of compute,
        // so you're better off using `declare_id!` to do this before program
        // compilation. See what I did in `utils.rs`.
        //let owner_pubkey = Pubkey::from_str(owner::id).unwrap();

        // Check if the caller is the owner
        require!(authority.key() == owner::id(), GGError::Unauthorized);

        let pot = &mut ctx.accounts.pot;
        let protocol = &mut ctx.accounts.protocol;

        // set pot and protocol authority
        pot.authority = authority.key();
        protocol.authority = authority.key();

        msg!("Pot and Protocol accounts initialized");

        Ok(())
    }

    pub fn mint(ctx: Context<Mint>) -> Result<()> {
        let mint = &mut ctx.accounts.mint;
        let protocol = &mut ctx.accounts.protocol;
        let authority = &mut ctx.accounts.authority;

        // create a new token_count account
        mint.subject = authority.key();

        // transfer 0.5 SOL from authority to protocol
        let amount = sol_to_lamports(0.5);
        let transfer_instruction =
            system_instruction::transfer(&authority.key(), &protocol.key(), amount);
        invoke(
            &transfer_instruction,
            &[
                authority.to_account_info(),
                protocol.to_account_info(),
                ctx.accounts.system_program.to_account_info(),
            ],
        )?;

        // increase share supply in mint account
        mint.amount = 1;

        Ok(())
    }

    pub fn buy_shares(ctx: Context<BuyShares>, subject: Pubkey, amount: u64) -> Result<()> {
        let token = &mut ctx.accounts.token;
        let mint = &mut ctx.accounts.mint;
        let protocol = &mut ctx.accounts.protocol;
        let pot = &mut ctx.accounts.pot;
        let authority = &mut ctx.accounts.authority;

        // set supply
        let supply = mint.amount;
        require!(supply > 0, GGError::InvalidSupply);

        // get buy price
        let price = get_buy_price(supply, amount);

        msg!("price: {}", lamports_to_sol(price));
        msg!("subject: {}", subject);
        msg!("buyer: {}", authority.key());

        let clock = Clock::get()?;
        let current_timestamp = clock.unix_timestamp;

        msg!("timestamp: {}", current_timestamp);

        // JON: !!!IMPORTANT!!! Use checked math here to avoid overflowing and
        // making it possible to exploit.
        // calculate fees
        let protocol_fee = price * PROTOCOL_FEE_PERCENT / LAMPORTS_PER_SOL;
        let subject_fee = price * SUBJECT_FEE_PERCENT / LAMPORTS_PER_SOL;

        // Ensure enough lamports are provided
        require!(
            authority.lamports() >= price + protocol_fee + subject_fee,
            GGError::InsufficientFunds
        );

        // Transfer fees
        let transfer_instruction =
            system_instruction::transfer(&authority.key(), &protocol.key(), protocol_fee);
        invoke(
            &transfer_instruction,
            &[
                authority.to_account_info(),
                protocol.to_account_info(),
                ctx.accounts.system_program.to_account_info(),
            ],
        )?;

        let transfer_instruction =
            system_instruction::transfer(&authority.key(), &mint.key(), subject_fee);
        invoke(
            &transfer_instruction,
            &[
                authority.to_account_info(),
                mint.to_account_info(),
                ctx.accounts.system_program.to_account_info(),
            ],
        )?;

        // // transfer price from authority to pot
        let transfer_instruction =
            system_instruction::transfer(&authority.key(), &pot.key(), price);
        invoke(
            &transfer_instruction,
            &[
                authority.to_account_info(),
                pot.to_account_info(),
                ctx.accounts.system_program.to_account_info(),
            ],
        )?;

        // Determine if the token account is already initialized
        if token.amount > 0 || token.owner != Pubkey::default() {
            // Token account is already initialized, update it as needed
            token.amount += amount;
        } else {
            // Token account is not initialized, set initial values
            token.owner = authority.key();
            token.subject = subject;
            token.amount = amount;
        }

        // JON: !!!IMPORTANT!!! Use checked math here too to be safe and avoid
        // overflowing
        mint.amount += amount;

        Ok(())
    }

    pub fn sell_shares(ctx: Context<SellShares>, subject: Pubkey, amount: u64) -> Result<()> {
        let token = &mut ctx.accounts.token;
        let mint = &mut ctx.accounts.mint;
        let protocol = &mut ctx.accounts.protocol;
        let pot = &mut ctx.accounts.pot;
        let authority = &mut ctx.accounts.authority;

        // set supply
        let supply = mint.amount;
        require!(supply > amount, GGError::InvalidSupply);

        // get buy price
        let price = get_sell_price(supply, amount);

        msg!("price: {}", lamports_to_sol(price));
        msg!("subject: {}", subject);
        msg!("buyer: {}", authority.key());

        let clock = Clock::get()?;
        let current_timestamp = clock.unix_timestamp;

        msg!("timestamp: {}", current_timestamp);

        // calculate fees
        // JON: !!!IMPORTANT!!! use checked math here
        let protocol_fee = price * PROTOCOL_FEE_PERCENT / LAMPORTS_PER_SOL;
        let subject_fee = price * SUBJECT_FEE_PERCENT / LAMPORTS_PER_SOL;

        // JON: The earlier check is the correct one -- since the mint is
        // initialized with a supply of `1` but has no corresponding tokens,
        // it's not correct for `mint.amount` to ever equal `0`.
        require!(mint.amount >= amount, GGError::InsufficientShares);

        // Transfer fees

        // transfer price from pot to authority
        // JON: The runtime will stop you from doing something bad here since
        // lamports need to be checked, but be sure to use checked math here too.
        **pot.to_account_info().try_borrow_mut_lamports()? -= price;
        **mint.to_account_info().try_borrow_mut_lamports()? += subject_fee;
        **protocol.to_account_info().try_borrow_mut_lamports()? += protocol_fee;
        **authority.to_account_info().try_borrow_mut_lamports()? +=
            price - subject_fee - protocol_fee;

        // decrease token amount in token account
        // JON: !!!IMPORTANT!!! Be sure to check that `token.amount >= amount`
        // or use checked math -- as it stands, without checked math, I can put
        // a huge amount and steal everyone else's SOL from the pot :-)
        token.amount -= amount;

        // decrease share supply in mint account
        mint.amount -= amount;

        Ok(())
    }

    pub fn withdraw_from_protocol(ctx: Context<WithdrawFromProtocol>) -> Result<()> {
        let authority = &mut ctx.accounts.authority;
        let protocol = &mut ctx.accounts.protocol;
        let owner_pubkey = owner::id();
        let rent = &ctx.accounts.rent;

        // Check if the caller is the owner
        // JON: not important, but same as earlier, since the `owner_pubkey` is
        // hard-coded in the source code, the `protocol.authority` field is
        // totally unused.
        require!(authority.key() == owner_pubkey, GGError::Unauthorized);

        // Get the rent-exempt threshold
        let rent_exempt_threshold =
            Rent::from_account_info(rent)?.minimum_balance(protocol.to_account_info().data_len());

        // Get the total amount of lamports in the mint account
        let total_lamports = protocol.to_account_info().lamports();
        require!(
            total_lamports > rent_exempt_threshold,
            GGError::InsufficientFunds
        );

        // Calculate the amount that can be safely withdrawn
        let withdrawable_amount = total_lamports - rent_exempt_threshold;

        msg!("Protocol balance: {}", withdrawable_amount);
        msg!("Protocol owner: {}", protocol.authority);

        // Ensure enough lamports are provided
        // JON: not important, but since you've already checked
        // `total_lamports > rent_exempt_threshold`, this check is unnecessary.
        require!(withdrawable_amount > 0, GGError::InsufficientFunds);

        // Transfer fees
        **protocol.to_account_info().try_borrow_mut_lamports()? -= withdrawable_amount;
        **authority.try_borrow_mut_lamports()? += withdrawable_amount;

        Ok(())
    }

    pub fn withdraw_from_mint(ctx: Context<WithdrawFromMint>) -> Result<()> {
        let authority = &mut ctx.accounts.authority;
        let mint = &mut ctx.accounts.mint;
        let rent = &ctx.accounts.rent;

        // Get the rent-exempt threshold
        let rent_exempt_threshold =
            Rent::from_account_info(rent)?.minimum_balance(mint.to_account_info().data_len());

        // JON: !!!IMPORTANT!!! you *must* check that the provided `authority`
        // key matches the one stored in the mint. As it stands, anyone can call
        // this instruction to withdraw SOL from the mint.

        // Get the total amount of lamports in the mint account
        let total_lamports = mint.to_account_info().lamports();
        require!(
            total_lamports > rent_exempt_threshold,
            GGError::InsufficientFunds
        );

        // Calculate the amount that can be safely withdrawn
        let withdrawable_amount = total_lamports - rent_exempt_threshold;

        msg!("Mint balance: {}", total_lamports);
        msg!("Mint owner: {}", mint.subject);
        msg!("Withdrawable amount: {}", withdrawable_amount);

        // Transfer the withdrawable amount to the authority account
        **mint.to_account_info().try_borrow_mut_lamports()? -= withdrawable_amount;
        **authority.try_borrow_mut_lamports()? += withdrawable_amount;

        Ok(())
    }
}
