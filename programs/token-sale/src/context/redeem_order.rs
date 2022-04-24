use anchor_lang::prelude::*;
use anchor_spl::token::{self, TokenAccount, Token, Mint, CloseAccount, transfer, Transfer};
use crate::account::*;
use crate::Tokens;

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


impl<'info> RedeemOrder<'info> {
    pub fn send_tokens_from_order_to_buyer(&self, tokens_amount: Tokens) -> Result<()> {
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
                    to: self.buyer_token_account.to_account_info(),
                    authority: self.order.to_account_info(),
                },
                &[&seeds[..]]
            ),
            tokens_amount.into()
        )
    }
}
