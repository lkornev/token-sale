use anchor_lang::prelude::*;
use anchor_spl::token::{self, TokenAccount, Token, Mint};
use anchor_spl::associated_token::AssociatedToken;
use crate::account::*;

#[derive(Accounts)]
pub struct RedeemOrder<'info> {
    #[account(
        mut,
        seeds = [selling_mint.to_account_info().key.as_ref()],
        bump = pool_account.bump,
        has_one = selling_mint
    )]
    pub pool_account: Account<'info, PoolAccount>,
    pub selling_mint: Box<Account<'info, Mint>>,
    #[account(mut)]
    pub buyer: Signer<'info>,
    #[account(
        mut,
        constraint = buyer_token_account.owner == buyer.key(),
        constraint = buyer_token_account.mint == selling_mint.key(),
    )]
    pub buyer_token_account: Account<'info, TokenAccount>,
    #[account(
        mut,
        seeds = [Order::PDA_SEED, order.owner.as_ref()],
        bump,
        constraint = order.owner == order_owner.key(),
    )]
    pub order: Account<'info, Order>,
    /// CHECK used only to transfer lamports into
    #[account(mut)]
    pub order_owner: SystemAccount<'info>,
    #[account(
        mut,
        constraint = order_token_vault.owner == order.key(),
        constraint = order_token_vault.mint == selling_mint.key(),
    )]
    pub order_token_vault: Account<'info, TokenAccount>,
    pub token_program: Program<'info, Token>,
    pub clock: Sysvar<'info, Clock>,
    pub system_program: Program<'info, System>,
}