use anchor_lang::prelude::*;
use anchor_spl::token::{TokenAccount, Token, Mint, transfer, Transfer};
use anchor_spl::associated_token::AssociatedToken;
use crate::account::*;
use crate::currency::{Tokens};

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(
        init,
        payer = distribution_authority,
        space = 8 + PoolAccount::SPACE,
        seeds = [selling_mint.to_account_info().key.as_ref()],
        bump,
    )]
    pub pool_account: Account<'info, PoolAccount>,
    // The one who will sign the transaction of transferring tokens from the `tokens_for_distribution` account
    // to the `vault_selling` account.
    // And then allowed to withdraw payment from the `vault_payment`.
    #[account(mut)]
    pub distribution_authority: Signer<'info>,
    #[account(
        mut,
        constraint = tokens_for_distribution.owner == distribution_authority.key(),
    )]
    pub tokens_for_distribution: Account<'info, TokenAccount>,
    pub selling_mint: Box<Account<'info, Mint>>,
    #[account(
        init,
        payer = distribution_authority,
        associated_token::mint = selling_mint,
        associated_token::authority = pool_account,
    )]
    pub vault_selling: Account<'info, TokenAccount>,
    pub clock: Sysvar<'info, Clock>,
    pub token_program: Program<'info, Token>,
    pub rent: Sysvar<'info, Rent>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}

impl<'info> Initialize<'info> {
    pub fn send_tokens_to_pool(&self, amount_to_sell: Tokens) -> Result<()> {
        let cpi_accounts = Transfer {
            from: self.tokens_for_distribution.to_account_info(),
            to: self.vault_selling.to_account_info(),
            authority: self.distribution_authority.to_account_info(),
        };
        transfer(
            CpiContext::new(self.token_program.to_account_info(), cpi_accounts),
            amount_to_sell.into()
        )
    }
}
