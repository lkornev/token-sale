use anchor_lang::prelude::*;
use crate::account::*;

#[derive(Accounts)]
pub struct SwitchToBuying<'info> {
    #[account(
        mut,
        seeds = [pool_account.selling_mint.as_ref()],
        bump = pool_account.bump,
    )]
    pub pool_account: Account<'info, PoolAccount>,
    pub clock: Sysvar<'info, Clock>,
}