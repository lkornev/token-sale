use anchor_lang::prelude::*;
use anchor_spl::token::{
    TokenAccount,
    Token,
    Mint,
    Burn,
    burn,
    CloseAccount,
    close_account,
};
use crate::account::*;

#[derive(Accounts)]
pub struct Terminate<'info> {
    #[account(
        mut,
        seeds = [selling_mint.to_account_info().key.as_ref()],
        bump = pool_account.bump,
        has_one = selling_mint,
        has_one = owner,
        close = owner,
    )]
    pub pool_account: Account<'info, PoolAccount>,
    #[account(mut)]
    pub selling_mint: Box<Account<'info, Mint>>,
    #[account(
        mut,
        associated_token::mint = selling_mint,
        associated_token::authority = pool_account,
    )]
    pub vault_selling: Account<'info, TokenAccount>,
    #[account(mut)]
    pub owner: Signer<'info>,
    pub clock: Sysvar<'info, Clock>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

impl<'info> Terminate<'info> {
    pub fn burn_left_tokens(&mut self) -> Result<()> {
        let seeds = &[
            self.selling_mint.to_account_info().key.as_ref(),
            &[self.pool_account.bump]
        ];

        burn(
            CpiContext::new_with_signer(
                self.token_program.to_account_info(),
                Burn {
                    mint: self.selling_mint.to_account_info(),
                    from: self.vault_selling.to_account_info(),
                    authority: self.pool_account.to_account_info(),
                },
                &[&seeds[..]]
            ),
            self.vault_selling.amount
        )
    }

    pub fn close_vault_selling(&mut self) -> Result<()> {
        let seeds = &[
            self.selling_mint.to_account_info().key.as_ref(),
            &[self.pool_account.bump]
        ];

        close_account(
            CpiContext::new_with_signer(
                self.token_program.to_account_info(),
                CloseAccount {
                    account: self.vault_selling.to_account_info(),
                    destination: self.owner.to_account_info(),
                    authority: self.pool_account.to_account_info(),
                },
                &[&seeds[..]]
            ),
        )
    }
}
