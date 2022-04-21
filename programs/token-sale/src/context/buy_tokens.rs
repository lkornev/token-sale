use anchor_lang::prelude::*;
use anchor_spl::token::{self, TokenAccount, Token, Mint};
use anchor_spl::associated_token::AssociatedToken;
use crate::account::*;

#[derive(Accounts)]
pub struct BuyTokens<'info> {
    #[account(
        mut,
        seeds = [selling_mint.to_account_info().key.as_ref()],
        bump = pool_account.bump,
        has_one = selling_mint,
        has_one = vault_selling,
    )]
    pub pool_account: Account<'info, PoolAccount>,
    pub selling_mint: Box<Account<'info, Mint>>,
    #[account(mut)]
    pub vault_selling: Account<'info, TokenAccount>,
    #[account(mut)]
    pub buyer: Signer<'info>,
    #[account(
        mut,
        constraint = buyer_token_account.owner == buyer.key(),
        constraint = buyer_token_account.mint == selling_mint.key(),
    )]
    pub buyer_token_account: Account<'info, TokenAccount>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
    pub clock: Sysvar<'info, Clock>,
}