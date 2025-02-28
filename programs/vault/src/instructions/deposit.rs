use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount, Mint, Transfer};
use whirlpool_cpi::state::{Position, Whirlpool};
use crate::util::calculate_total_tokens;
use crate::utils::calculate_shares;
use crate::state::Vault;

#[derive(Accounts)]
pub struct Deposit<'info> {
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

    #[account(mut, has_one = whirlpool)]
    pub position: Option<Account<'info, Position>>,
    
    #[account(mut, constraint = user_token_a.mint == vault.token_a_mint)]
    pub user_token_a: Account<'info, TokenAccount>,
    #[account(mut, constraint = user_token_b.mint == vault.token_b_mint)]
    pub user_token_b: Account<'info, TokenAccount>,
    #[account(mut, constraint = token_a_vault.key() == vault.token_a_vault)]
    pub token_a_vault: Account<'info, TokenAccount>,
    #[account(mut, constraint = token_b_vault.key() == vault.token_b_vault)]
    pub token_b_vault: Account<'info, TokenAccount>,
    
    #[account(mut, constraint = share_mint.key() == vault.share_mint)]
    pub share_mint: Account<'info, Mint>,
    #[account(mut, constraint = user_share.mint == vault.share_mint)]
    pub user_share: Account<'info, TokenAccount>,
    
    pub token_program: Program<'info, Token>,
}

pub fn deposit(
    ctx: Context<Deposit>,
    amount_a: u64,
    amount_b: u64,
) -> Result<()> {
    let vault = &ctx.accounts.vault;
    let total_shares = ctx.accounts.share_mint.supply;
    let whirlpool = &ctx.accounts.whirlpool;
    let position = &ctx.accounts.position;
    // Get current tick index and sqrt_price from whirlpool
    let current_tick_index = whirlpool.tick_current_index;
    let sqrt_price = whirlpool.sqrt_price;

    if amount_a > 0 {
        token::transfer(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                Transfer {
                    from: ctx.accounts.user_token_a.to_account_info(),
                    to: ctx.accounts.token_a_vault.to_account_info(),
                    authority: ctx.accounts.user.to_account_info(),
                },
            ),
            amount_a,
        )?;
    }

    if amount_b > 0 {
        token::transfer(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                Transfer {
                    from: ctx.accounts.user_token_b.to_account_info(),
                    to: ctx.accounts.token_b_vault.to_account_info(),
                    authority: ctx.accounts.user.to_account_info(),
                },
            ),
            amount_b,
        )?;
    }

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

    let max_decimal = ctx.accounts.vault.token_a_decimal.max(ctx.accounts.vault.token_b_decimal);
    let share_amount = calculate_shares(
        amount_a,
        amount_b,
        total_token_a_amount,
        total_token_b_amount,
        ctx.accounts.vault.token_a_price,
        ctx.accounts.vault.token_b_price,
        total_shares,
        max_decimal
    );

    token::mint_to(
        CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            token::MintTo {
                mint: ctx.accounts.share_mint.to_account_info(),
                to: ctx.accounts.user_share.to_account_info(),
                authority: ctx.accounts.vault.to_account_info(),
            },
            &[&[
                b"vault",
                ctx.accounts.whirlpool.key().as_ref(),
                &[ctx.accounts.vault.bump]
            ]],
        ),
        share_amount,
    )?;

    Ok(())
}
