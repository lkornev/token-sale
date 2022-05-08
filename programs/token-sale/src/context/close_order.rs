use anchor_lang::prelude::*;
use anchor_spl::token::{self, TokenAccount, Token, Mint, transfer, Transfer, CloseAccount};
use crate::account::*;
use crate::currency::Tokens;
use crate::error::ErrorCode;

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
        bump = order.bump,
        constraint = order.owner == order_owner.key() @ErrorCode::OnlyOwnerCanCloseOrder,
        close = order_owner,
    )]
    pub order: Account<'info, Order>,
    #[account(
        mut,
        constraint = order_token_vault.owner == order.key(),
        constraint = order_token_vault.mint == selling_mint.key(),
    )]
    pub order_token_vault: Account<'info, TokenAccount>,
    #[account(mut)]
    pub order_owner: Signer<'info>,
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
    pub fn sent_all_tokens_from_order_to_owner(&mut self) -> Result<()> {
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
        )?;

        self.order.token_amount = Tokens::new(0);

        Ok(())
    }

    pub fn close_order_token_vault(&mut self) -> Result<()> {
        let seeds = &[
            Order::PDA_SEED,
            self.order.owner.as_ref(),
            &[self.order.bump]
        ];

        token::close_account(
            CpiContext::new_with_signer(
                self.token_program.to_account_info(),
                CloseAccount {
                    account: self.order_token_vault.to_account_info(),
                    destination: self.order_owner.to_account_info(),
                    authority: self.order.to_account_info(),
                },
                &[&seeds[..]]
            ),
        )
    }
}
