use anchor_lang::prelude::*;
use anchor_spl::token::{TokenAccount, Token, Mint, transfer, Transfer};
use anchor_spl::associated_token::AssociatedToken;
use crate::account::*;
use crate::Tokens;

#[derive(Accounts)]
pub struct PlaceOrder<'info> {
    #[account(
        mut,
        seeds = [selling_mint.to_account_info().key.as_ref()],
        bump = pool_account.bump,
        has_one = selling_mint
    )]
    pub pool_account: Account<'info, PoolAccount>,
    pub selling_mint: Box<Account<'info, Mint>>,
    #[account(mut)]
    pub seller: Signer<'info>,
    #[account(
        mut,
        constraint = seller_token_account.owner == seller.key(),
        constraint = seller_token_account.mint == selling_mint.key(),
    )]
    pub seller_token_account: Account<'info, TokenAccount>,
    #[account(
        init,
        payer = seller,
        space = 8 + Order::SPACE,
        seeds = [Order::PDA_SEED, seller.key().as_ref()],
        bump,
    )]
    pub order: Account<'info, Order>,
    #[account(
        init,
        payer = seller,
        associated_token::mint = selling_mint,
        associated_token::authority = order,
    )]
    pub order_token_vault: Account<'info, TokenAccount>,
    pub token_program: Program<'info, Token>,
    pub rent: Sysvar<'info, Rent>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
    pub clock: Sysvar<'info, Clock>,
}

impl<'info> PlaceOrder<'info> {
    pub fn send_tokens_from_seller_to_order(&self, amount_to_sell: Tokens) -> Result<()> {
        let cpi_accounts = Transfer {
            from:  self.seller_token_account.to_account_info(),
            to: self.order_token_vault.to_account_info(),
            authority: self.seller.to_account_info(),
        };
        transfer(
            CpiContext::new(self.token_program.to_account_info(), cpi_accounts),
            amount_to_sell.into()
        )
    }
}
