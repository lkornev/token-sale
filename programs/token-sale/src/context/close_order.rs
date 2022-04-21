use anchor_lang::prelude::*;
use anchor_spl::token::{TokenAccount, Token, Mint, transfer, Transfer};
use crate::account::*;

#[derive(Accounts)]
pub struct CloseOrder<'info> {
    #[account(
        mut,
        seeds = [selling_mint.to_account_info().key.as_ref()],
        bump = pool_account.bump,
        has_one = selling_mint,
    )]
    pub pool_account: Account<'info, PoolAccount>,
    pub selling_mint: Box<Account<'info, Mint>>,
    #[account(
        mut,
        seeds = [Order::PDA_SEED, order.owner.as_ref()],
        bump,
        constraint = order.owner == order_owner.key(),
        close = order_owner,
    )]
    pub order: Account<'info, Order>,
    #[account(
        mut,
        constraint = order_token_vault.owner == order.key(),
        constraint = order_token_vault.mint == selling_mint.key(),
        close = order_owner,
    )]
    pub order_token_vault: Account<'info, TokenAccount>,
    /// CHECK used only to transfer lamports into
    #[account(mut)]
    pub order_owner: SystemAccount<'info>,
    #[account(
        mut,
        constraint = owner_token_vault.owner == order.owner.key(),
        constraint = owner_token_vault.mint == pool_account.selling_mint.key(),
    )]
    pub owner_token_vault: Account<'info, TokenAccount>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

impl<'info> CloseOrder<'info> {
    pub fn sent_tokens_from_order_to_owner(&self) -> Result<()> {
        let seeds = &[
            Order::PDA_SEED,
            self.order.owner.as_ref(),
            &[self.order.bump]
        ];

        transfer(
            CpiContext::new_with_signer(
                self.token_program.to_account_info(),
                Transfer {
                    from: self.order_token_vault.to_account_info(),
                    to: self.owner_token_vault.to_account_info(),
                    authority: self.order.to_account_info(),
                },
                &[&seeds[..]]
            ),
            self.order_token_vault.amount
        )
    }
}
