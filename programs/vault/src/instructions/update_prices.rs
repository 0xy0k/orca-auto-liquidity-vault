use anchor_lang::prelude::*;
use whirlpool_cpi::state::Whirlpool;
use crate::state::Vault;

#[derive(Accounts)]
pub struct UpdatePrices<'info> {
    #[account(mut, constraint = admin.key() == vault.admin)]
    pub admin: Signer<'info>,
    
    pub whirlpool: Box<Account<'info, Whirlpool>>,

    #[account(
        mut,
        seeds = [
            b"vault".as_ref(),
            whirlpool.key().as_ref(),
        ],
        bump,
    )]
    pub vault: Box<Account<'info, Vault>>,
}

pub fn update_prices(
    ctx: Context<UpdatePrices>,
    token_a_price: u64,
    token_b_price: u64,
) -> Result<()> {
    let vault = &mut ctx.accounts.vault;
    vault.token_a_price = token_a_price;
    vault.token_b_price = token_b_price;
    Ok(())
}