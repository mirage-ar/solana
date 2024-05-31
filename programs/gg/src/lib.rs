use anchor_lang::prelude::*;
use anchor_lang::solana_program::native_token::{
    lamports_to_sol, sol_to_lamports, LAMPORTS_PER_SOL,
};
use anchor_lang::solana_program::program::invoke;
use anchor_lang::solana_program::system_instruction;
use anchor_lang::solana_program::sysvar::rent::Rent;

pub mod error;
pub mod state;
pub mod utils;

use crate::{error::*, state::*, utils::*};

declare_id!("4NZwzHq6bS1LqhUPPr7LjDz5aV18CYugg6PSx6GBXgDe");

#[program]
pub mod gg {
    use super::*;

    pub fn initialize(_ctx: Context<Initialize>) -> Result<()> {
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

        // Use checked math to avoid overflow
        let protocol_fee = price
            .checked_mul(PROTOCOL_FEE_PERCENT)
            .ok_or(GGError::MathOverflow)?
            .checked_div(LAMPORTS_PER_SOL)
            .ok_or(GGError::MathOverflow)?;
        let subject_fee = price
            .checked_mul(SUBJECT_FEE_PERCENT)
            .ok_or(GGError::MathOverflow)?
            .checked_div(LAMPORTS_PER_SOL)
            .ok_or(GGError::MathOverflow)?;

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

        // Transfer price from authority to pot
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
            token.amount = token
                .amount
                .checked_add(amount)
                .ok_or(GGError::MathOverflow)?;
        } else {
            // Token account is not initialized, set initial values
            token.owner = authority.key();
            token.subject = subject;
            token.amount = amount;
        }

        // Use checked math to be safe and avoid overflow
        mint.amount = mint
            .amount
            .checked_add(amount)
            .ok_or(GGError::MathOverflow)?;

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

        // get sell price
        let price = get_sell_price(supply, amount);

        msg!("price: {}", lamports_to_sol(price));
        msg!("subject: {}", subject);
        msg!("buyer: {}", authority.key());

        let clock = Clock::get()?;
        let current_timestamp = clock.unix_timestamp;

        msg!("timestamp: {}", current_timestamp);

        // Use checked math to avoid overflow
        let protocol_fee = price
            .checked_mul(PROTOCOL_FEE_PERCENT)
            .ok_or(GGError::MathOverflow)?
            .checked_div(LAMPORTS_PER_SOL)
            .ok_or(GGError::MathOverflow)?;
        let subject_fee = price
            .checked_mul(SUBJECT_FEE_PERCENT)
            .ok_or(GGError::MathOverflow)?
            .checked_div(LAMPORTS_PER_SOL)
            .ok_or(GGError::MathOverflow)?;

        // Transfer fees
        **pot.to_account_info().try_borrow_mut_lamports()? = pot
            .to_account_info()
            .lamports()
            .checked_sub(price)
            .ok_or(GGError::MathOverflow)?;
        **mint.to_account_info().try_borrow_mut_lamports()? = mint
            .to_account_info()
            .lamports()
            .checked_add(subject_fee)
            .ok_or(GGError::MathOverflow)?;
        **protocol.to_account_info().try_borrow_mut_lamports()? = protocol
            .to_account_info()
            .lamports()
            .checked_add(protocol_fee)
            .ok_or(GGError::MathOverflow)?;
        **authority.to_account_info().try_borrow_mut_lamports()? = authority
            .to_account_info()
            .lamports()
            .checked_add(
                price
                    .checked_sub(subject_fee)
                    .ok_or(GGError::MathOverflow)?
                    .checked_sub(protocol_fee)
                    .ok_or(GGError::MathOverflow)?,
            )
            .ok_or(GGError::MathOverflow)?;

        // Decrease token amount in token account
        require!(token.amount >= amount, GGError::InsufficientShares);
        token.amount = token
            .amount
            .checked_sub(amount)
            .ok_or(GGError::MathOverflow)?;

        // Decrease share supply in mint account
        mint.amount = mint
            .amount
            .checked_sub(amount)
            .ok_or(GGError::MathOverflow)?;

        Ok(())
    }

    pub fn withdraw_from_protocol(ctx: Context<WithdrawFromProtocol>) -> Result<()> {
        let authority = &mut ctx.accounts.authority;
        let protocol = &mut ctx.accounts.protocol;

        // Check if the caller is the owner
        require!(authority.key() == owner::id(), GGError::Unauthorized);

        // Get the rent-exempt threshold
        let rent_exempt_threshold =
            Rent::get()?.minimum_balance(protocol.to_account_info().data_len());

        // Get the total amount of lamports in the protocol account
        let total_lamports = protocol.to_account_info().lamports();
        require!(
            total_lamports > rent_exempt_threshold,
            GGError::InsufficientFunds
        );

        // Calculate the amount that can be safely withdrawn
        let withdrawable_amount = total_lamports
            .checked_sub(rent_exempt_threshold)
            .ok_or(GGError::MathOverflow)?;

        msg!("Protocol balance: {}", withdrawable_amount);

        // Transfer withdrawable amount to the authority account
        **protocol.to_account_info().try_borrow_mut_lamports()? = protocol
            .to_account_info()
            .lamports()
            .checked_sub(withdrawable_amount)
            .ok_or(GGError::MathOverflow)?;
        **authority.try_borrow_mut_lamports()? = authority
            .to_account_info()
            .lamports()
            .checked_add(withdrawable_amount)
            .ok_or(GGError::MathOverflow)?;

        Ok(())
    }

    pub fn withdraw_from_mint(ctx: Context<WithdrawFromMint>) -> Result<()> {
        let authority = &mut ctx.accounts.authority;
        let mint = &mut ctx.accounts.mint;

        // Get the rent-exempt threshold
        let rent_exempt_threshold =
            Rent::get()?.minimum_balance(mint.to_account_info().data_len());

        // Check that the provided authority key matches the one stored in the mint
        require!(authority.key() == mint.subject, GGError::Unauthorized);

        // Get the total amount of lamports in the mint account
        let total_lamports = mint.to_account_info().lamports();
        require!(
            total_lamports > rent_exempt_threshold,
            GGError::InsufficientFunds
        );

        // Calculate the amount that can be safely withdrawn
        let withdrawable_amount = total_lamports
            .checked_sub(rent_exempt_threshold)
            .ok_or(GGError::MathOverflow)?;

        msg!("Mint balance: {}", total_lamports);
        msg!("Mint owner: {}", mint.subject);
        msg!("Withdrawable amount: {}", withdrawable_amount);

        // Transfer the withdrawable amount to the authority account
        **mint.to_account_info().try_borrow_mut_lamports()? = mint
            .to_account_info()
            .lamports()
            .checked_sub(withdrawable_amount)
            .ok_or(GGError::MathOverflow)?;
        **authority.try_borrow_mut_lamports()? = authority
            .to_account_info()
            .lamports()
            .checked_add(withdrawable_amount)
            .ok_or(GGError::MathOverflow)?;

        Ok(())
    }
}
