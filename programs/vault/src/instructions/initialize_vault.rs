use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount, Mint};
use whirlpool_cpi::state::Whirlpool;
use crate::state::Vault;

#[derive(Accounts)]
pub struct InitializeVault<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,
    
    #[account(
        init,
        seeds = [
            b"vault".as_ref(),
            whirlpool.key().as_ref(),
        ],
        bump,
        payer = admin,
        space = 8 + Vault::LEN,
    )]
    pub vault: Box<Account<'info, Vault>>,
    
    pub whirlpool: Box<Account<'info, Whirlpool>>,
    #[account(constraint = token_a_mint.key() == whirlpool.token_mint_a)]
    pub token_a_mint: Account<'info, Mint>,
    #[account(constraint = token_b_mint.key() == whirlpool.token_mint_b)]
    pub token_b_mint: Account<'info, Mint>,
    
    #[account(init,
        payer = admin,
        token::mint = token_a_mint,
        token::authority = vault)]
    pub token_a_vault: Account<'info, TokenAccount>,
    #[account(init,
        payer = admin,
        token::mint = token_b_mint,
        token::authority = vault)]
    pub token_b_vault: Account<'info, TokenAccount>,
    #[account(
        init,
        payer = admin,
        mint::decimals = 6,
        mint::authority = vault,
        mint::freeze_authority = vault,
    )]
    pub share_mint: Account<'info, Mint>,
    
    #[account(address = token::ID)]
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

pub fn initialize_vault(
    ctx: Context<InitializeVault>,
    token_a_price: u64,
    token_b_price: u64,
    lower_tick: i32,
    upper_tick: i32,
) -> Result<()> {
    let vault = &mut ctx.accounts.vault;
    vault.admin = ctx.accounts.admin.key();
    vault.whirlpool = ctx.accounts.whirlpool.key();
    vault.token_a_mint = ctx.accounts.token_a_mint.key();
    vault.token_b_mint = ctx.accounts.token_b_mint.key();
    vault.token_a_vault = ctx.accounts.token_a_vault.key();
    vault.token_b_vault = ctx.accounts.token_b_vault.key();
    vault.share_mint = ctx.accounts.share_mint.key();
    vault.token_a_price = token_a_price;
    vault.token_b_price = token_b_price;
    vault.token_a_decimal = ctx.accounts.token_a_mint.decimals;
    vault.token_b_decimal = ctx.accounts.token_b_mint.decimals;
    vault.lower_tick = lower_tick;
    vault.upper_tick = upper_tick;
    vault.bump = ctx.bumps.vault;
    Ok(())
}