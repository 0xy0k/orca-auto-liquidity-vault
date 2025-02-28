use anchor_lang::prelude::*;
use whirlpool_cpi::state::Whirlpool;
use crate::state::Vault;

#[derive(Accounts)]
pub struct UpdateTicks<'info> {
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

pub fn update_ticks(
    ctx: Context<UpdateTicks>,
    lower_tick: i32,
    upper_tick: i32,
) -> Result<()> {
    let vault = &mut ctx.accounts.vault;
    vault.lower_tick = lower_tick;
    vault.upper_tick = upper_tick;
    Ok(())
}