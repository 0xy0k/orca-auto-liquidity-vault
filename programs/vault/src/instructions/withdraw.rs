use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount, Mint, Transfer};
use whirlpool_cpi::state::{Position, Whirlpool};
use crate::{errors::VaultError, state::Vault, util::{calculate_proportional_amount, calculate_total_tokens}};

#[derive(Accounts)]
pub struct Withdraw<'info> {
    #[account(mut)]
    pub user: Signer<'info>,

    #[account(constraint = whirlpool.key() == vault.whirlpool)]
    pub whirlpool: Box<Account<'info, Whirlpool>>,

    #[account(
        seeds = [
            b"vault".as_ref(),
            whirlpool.key().as_ref(),
        ],
        bump,
      )]
    pub vault: Box<Account<'info, Vault>>,

    #[account(has_one = whirlpool)]
    pub position: Option<Account<'info, Position>>,
    
    #[account(mut, constraint = user_token_a.mint == whirlpool.token_mint_a)]
    pub user_token_a: Account<'info, TokenAccount>,
    #[account(mut, constraint = user_token_b.mint == whirlpool.token_mint_b)]
    pub user_token_b: Account<'info, TokenAccount>,
    #[account(mut, address = vault.token_a_vault)]
    pub token_a_vault: Account<'info, TokenAccount>,
    #[account(mut, address = vault.token_b_vault)]
    pub token_b_vault: Account<'info, TokenAccount>,
    
    #[account(mut, constraint = share_mint.key() == vault.share_mint)]
    pub share_mint: Account<'info, Mint>,
    #[account(mut, constraint = user_share.mint == vault.share_mint)]
    pub user_share: Account<'info, TokenAccount>,
    
    pub token_program: Program<'info, Token>,
}

pub fn withdraw(
    ctx: Context<Withdraw>,
    share_amount: u64,
    token_min_a_amount: u64,
    token_min_b_amount: u64,
) -> Result<()> {
    let vault = &ctx.accounts.vault;
    let total_shares = ctx.accounts.share_mint.supply;
    let whirlpool = &ctx.accounts.whirlpool;
    let position = &ctx.accounts.position;
    // Get current tick index and sqrt_price from whirlpool
    let current_tick_index = whirlpool.tick_current_index;
    let sqrt_price = whirlpool.sqrt_price;
    
    // Calculate total value including tokens in Whirlpool position
    let (total_token_a_amount, total_token_b_amount) = if let Some(position) = position {
        calculate_total_tokens(
            ctx.accounts.token_a_vault.amount,
            ctx.accounts.token_b_vault.amount,
            position.liquidity,
            current_tick_index,
            sqrt_price,
            vault.lower_tick,
            vault.upper_tick,
        )?
    } else {
        (ctx.accounts.token_a_vault.amount, ctx.accounts.token_b_vault.amount)
    };

    // Calculate proportional amounts based on total tokens (including in position)
    let mut token_a_amount = calculate_proportional_amount(
        total_token_a_amount,
        share_amount,
        total_shares,
    )?;
    
    let mut token_b_amount = calculate_proportional_amount(
        total_token_b_amount,
        share_amount,
        total_shares,
    )?;

    if token_a_amount > ctx.accounts.token_a_vault.amount || token_b_amount > ctx.accounts.token_b_vault.amount {
         // Choose the more limiting token
        let a_ratio = token_a_amount as f64 / ctx.accounts.token_a_vault.amount as f64;
        let b_ratio = token_b_amount as f64 / ctx.accounts.token_b_vault.amount as f64;
         
        if a_ratio > b_ratio {
            token_a_amount = ctx.accounts.token_a_vault.amount;
            token_b_amount = calculate_proportional_amount(
                total_token_b_amount,
                token_a_amount,
                total_token_a_amount,
            )?;
        } else {
            token_b_amount = ctx.accounts.token_b_vault.amount;
            token_a_amount = calculate_proportional_amount(
                total_token_a_amount,
                token_b_amount,
                total_token_b_amount,
            )?;
        }
    }

    require!(
        token_a_amount >= token_min_a_amount && token_b_amount >= token_min_b_amount,
        VaultError::SlippageExceeded
    );

    let burn_amount = calculate_proportional_amount(
        share_amount,
        token_a_amount,
        total_token_a_amount,
    )?;

    if token_a_amount > 0 {
        token::transfer(
            CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                Transfer {
                    from: ctx.accounts.token_a_vault.to_account_info(),
                    to: ctx.accounts.user_token_a.to_account_info(),
                    authority: ctx.accounts.vault.to_account_info(),
                },
                &[&[b"vault".as_ref(), whirlpool.key().as_ref(), &[vault.bump]]],
            ),
            token_a_amount,
        )?;
    }

    if token_b_amount > 0 {
        token::transfer(
            CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                Transfer {
                    from: ctx.accounts.token_b_vault.to_account_info(),
                    to: ctx.accounts.user_token_b.to_account_info(),
                    authority: ctx.accounts.vault.to_account_info(),
                },
                &[&[b"vault".as_ref(), whirlpool.key().as_ref(), &[vault.bump]]],
            ),
            token_b_amount,
        )?;
    }

    // Burn shares
    token::burn(
        CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            token::Burn {
                mint: ctx.accounts.share_mint.to_account_info(),
                from: ctx.accounts.user_share.to_account_info(),
                authority: ctx.accounts.user.to_account_info(),
            },
        ),
        burn_amount,
    )?;

    Ok(())
}