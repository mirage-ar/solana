use anchor_lang::prelude::*;

// The pot account holds the total amount of lamports in the pot
#[account]
pub struct PotAccount {}

// The protocol account holds the protocol fees
#[account]
pub struct ProtocolAccount {}

// The mint holds number of tokens that exist for a subject and their fee income from transactions
#[account]
pub struct MintAccount {
    pub subject: Pubkey,
    pub amount: u64,
}

// A new token account is created for every user that holds subject tokens
#[account]
pub struct TokenAccount {
    pub owner: Pubkey,
    pub subject: Pubkey,
    pub amount: u64,
}

// FUNCTION CONTEXTS
#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(
        init,
        payer = authority,
        seeds = [b"POT"],
        bump,
        space = 8 // no fields
    )]
    pub pot: Account<'info, PotAccount>,

    #[account(
        init,
        payer = authority,
        seeds = [b"PROTOCOL"],
        bump,
        space = 8 // no fields
    )]
    pub protocol: Account<'info, ProtocolAccount>,

    #[account(mut)]
    pub authority: Signer<'info>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct Mint<'info> {
    #[account(
        init,
        payer = authority,
        seeds = [b"MINT", authority.key().as_ref()],
        bump,
        space = 8 + 32 + 8 // 32 (subject) + 8 (amount)
    )]
    pub mint: Account<'info, MintAccount>,

    #[account(
        mut,
        seeds = [b"PROTOCOL"],
        bump,
    )]
    pub protocol: Account<'info, ProtocolAccount>,

    #[account(mut)]
    pub authority: Signer<'info>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(subject: Pubkey, amount: u64)]
pub struct BuyShares<'info> {
    #[account(
        init_if_needed,
        payer = authority,
        seeds = [b"TOKEN", authority.key().as_ref(), subject.as_ref()],
        bump,
        space = 8 + 32 + 32 + 8 // 32 (owner) + 32 (subject) + 8 (amount)
    )]
    pub token: Account<'info, TokenAccount>,

    #[account(
        mut,
        seeds = [b"MINT", subject.as_ref()],
        bump,
    )]
    pub mint: Account<'info, MintAccount>,

    #[account(
        mut,
        seeds = [b"PROTOCOL"],
        bump,
    )]
    pub protocol: Account<'info, ProtocolAccount>,

    #[account(
        mut,
        seeds = [b"POT"],
        bump,
    )]
    pub pot: Account<'info, PotAccount>,

    #[account(mut)]
    pub authority: Signer<'info>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(subject: Pubkey, amount: u64)]
pub struct SellShares<'info> {
    #[account(
        mut,
        seeds = [b"TOKEN", authority.key().as_ref(), subject.as_ref()],
        bump,
    )]
    pub token: Account<'info, TokenAccount>,

    #[account(
        mut,
        seeds = [b"MINT", subject.as_ref()],
        bump,
    )]
    pub mint: Account<'info, MintAccount>,

    #[account(
        mut,
        seeds = [b"PROTOCOL"],
        bump,
    )]
    pub protocol: Account<'info, ProtocolAccount>,

    #[account(
        mut,
        seeds = [b"POT"],
        bump,
    )]
    pub pot: Account<'info, PotAccount>,

    #[account(mut)]
    pub authority: Signer<'info>,
}

#[derive(Accounts)]
pub struct WithdrawFromProtocol<'info> {
    #[account(
        mut,
        seeds = [b"PROTOCOL"],
        bump,
    )]
    pub protocol: Account<'info, ProtocolAccount>,

    #[account(mut)]
    pub authority: Signer<'info>,
}

#[derive(Accounts)]
pub struct WithdrawFromMint<'info> {
    #[account(
        mut,
        seeds = [b"MINT", authority.key().as_ref()],
        bump,
    )]
    pub mint: Account<'info, MintAccount>,

    #[account(mut)]
    pub authority: Signer<'info>,

    pub system_program: Program<'info, System>,
}
