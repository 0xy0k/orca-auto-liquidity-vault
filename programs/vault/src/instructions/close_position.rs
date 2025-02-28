use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, Mint, TokenAccount};
use whirlpool_cpi::{self, state::*, program::Whirlpool as WhirlpoolProgram};

use crate::state::Vault;

#[derive(Accounts)]
pub struct ClosePosition<'info> {
  #[account(mut, constraint = admin.key() == vault.admin)]
  pub admin: Signer<'info>,
  
  pub whirlpool_program: Program<'info, WhirlpoolProgram>,
  
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

  /// CHECK: safe (the account to receive the remaining balance of the closed account)
  #[account(mut)]
  pub receiver: UncheckedAccount<'info>,

  #[account(mut, has_one = whirlpool)]
  pub position: Account<'info, Position>,

  #[account(mut, address = position.position_mint)]
  pub position_mint: Account<'info, Mint>,

  #[account(mut,
      constraint = position_token_account.amount == 1,
      constraint = position_token_account.mint == position.position_mint)]
  pub position_token_account: Box<Account<'info, TokenAccount>>,

  #[account(address = token::ID)]
  pub token_program: Program<'info, Token>,
}

pub fn close_position(
  ctx: Context<ClosePosition>,
) -> Result<()> {
  let cpi_program = ctx.accounts.whirlpool_program.to_account_info();

  let cpi_accounts = whirlpool_cpi::cpi::accounts::ClosePosition {
    position_authority: ctx.accounts.vault.to_account_info(),
    receiver: ctx.accounts.receiver.to_account_info(),
    position: ctx.accounts.position.to_account_info(),
    position_mint: ctx.accounts.position_mint.to_account_info(),
    position_token_account: ctx.accounts.position_token_account.to_account_info(),
    token_program: ctx.accounts.token_program.to_account_info(),
  };

  let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);

  // execute CPI
  msg!("CPI: whirlpool close_position instruction");
  whirlpool_cpi::cpi::close_position(cpi_ctx)?;

  Ok(())
}