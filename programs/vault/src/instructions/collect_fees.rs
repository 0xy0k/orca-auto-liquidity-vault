use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount};
use whirlpool_cpi::{self, state::*, program::Whirlpool as WhirlpoolProgram};

use crate::state::Vault;

#[derive(Accounts)]
pub struct CollectFees<'info> {
  #[account(mut, constraint = admin.key() == vault.admin)]
  pub admin: Signer<'info>,

  pub whirlpool_program: Program<'info, WhirlpoolProgram>,

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
  pub position: Box<Account<'info, Position>>,
  #[account(
      constraint = position_token_account.mint == position.position_mint,
      constraint = position_token_account.amount == 1
  )]
  pub position_token_account: Box<Account<'info, TokenAccount>>,

  #[account(mut, constraint = token_vault_a.mint == whirlpool.token_mint_a)]
  pub token_vault_a: Box<Account<'info, TokenAccount>>,
  #[account(mut, address = whirlpool.token_vault_a)]
  pub pool_token_vault_a: Box<Account<'info, TokenAccount>>,

  #[account(mut, constraint = token_vault_b.mint == whirlpool.token_mint_b)]
  pub token_vault_b: Box<Account<'info, TokenAccount>>,
  #[account(mut, address = whirlpool.token_vault_b)]
  pub pool_token_vault_b: Box<Account<'info, TokenAccount>>,

  #[account(address = token::ID)]
  pub token_program: Program<'info, Token>,
}

pub fn collect_fees(
  ctx: Context<CollectFees>,
) -> Result<()> {
  let cpi_program = ctx.accounts.whirlpool_program.to_account_info();

  let cpi_accounts = whirlpool_cpi::cpi::accounts::CollectFees {
    whirlpool: ctx.accounts.whirlpool.to_account_info(),
    position_authority: ctx.accounts.vault.to_account_info(),
    position: ctx.accounts.position.to_account_info(),
    position_token_account: ctx.accounts.position_token_account.to_account_info(),
    token_owner_account_a: ctx.accounts.token_vault_a.to_account_info(),
    token_vault_a: ctx.accounts.pool_token_vault_a.to_account_info(),
    token_owner_account_b: ctx.accounts.token_vault_b.to_account_info(),
    token_vault_b: ctx.accounts.pool_token_vault_b.to_account_info(),
    token_program: ctx.accounts.token_program.to_account_info(),
  };

  let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);

  // execute CPI
  msg!("CPI: whirlpool collect_fees instruction");
  whirlpool_cpi::cpi::collect_fees(cpi_ctx)?;

  Ok(())
}