use anchor_lang::prelude::*;
use anchor_spl::token::{self, TokenAccount, Token, Mint};
use anchor_spl::associated_token::AssociatedToken;
use crate::account::*;

#[derive(Accounts)]
pub struct SwitchToTrading<'info> {
    #[account(
        mut,
        seeds = [pool_account.selling_mint.as_ref()],
        bump = pool_account.bump,
    )]
    pub pool_account: Account<'info, PoolAccount>,
    pub clock: Sysvar<'info, Clock>,
}
