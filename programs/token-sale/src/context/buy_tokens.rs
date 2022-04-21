use anchor_lang::prelude::*;
use anchor_spl::token::{TokenAccount, Token, Mint, transfer, Transfer};
use crate::account::*;
use crate::Tokens;

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

impl<'info> BuyTokens<'info> {
    pub fn send_tokens_from_pool_to_buyer(&self, tokens_amount: Tokens) -> Result<()> {
        let seeds = &[
            self.selling_mint.to_account_info().key.as_ref(),
            &[self.pool_account.bump]
        ];

        transfer(
            CpiContext::new_with_signer(
                self.token_program.to_account_info(),
                Transfer {
                    from: self.vault_selling.to_account_info(),
                    to: self.buyer_token_account.to_account_info(),
                    authority: self.pool_account.to_account_info(),
                },
                &[&seeds[..]]
            ),
            tokens_amount.into()
        )
    }
}
