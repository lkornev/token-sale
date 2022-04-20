use anchor_lang::prelude::*;
use crate::account::*;
use crate::context::*;
use anchor_spl::token::{self, CloseAccount, Transfer, transfer};

pub fn send_lamports<'a>(from: AccountInfo<'a>, to: AccountInfo<'a>, amount: u64) -> Result<()> {
    let ix = anchor_lang::solana_program::system_instruction::transfer(
        &from.key(),
        &to.key(),
        amount,
    );

    anchor_lang::solana_program::program::invoke(
        &ix,
        &[
            from.to_account_info(),
            to.to_account_info(),
        ],
    ).map_err(|err| err.into())
}

impl<'info> Initialize<'info> {
    pub fn send_tokens_to_pool(&self, amount_to_sell: u64) -> Result<()> {
        let cpi_accounts = Transfer {
            from: self.tokens_for_distribution.to_account_info(),
            to: self.vault_selling.to_account_info(),
            authority: self.distribution_authority.to_account_info(),
        };
        transfer(
            CpiContext::new(self.token_program.to_account_info(), cpi_accounts),
            amount_to_sell
        )
    }
}

impl<'info> BuyTokens<'info> {
    pub fn send_tokens_from_pool_to_buyer(&self, tokens_amount: u64) -> Result<()> {
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
            tokens_amount
        )
    }
}

impl<'info> PlaceOrder<'info> {
    pub fn send_tokens_from_seller_to_order(&self, amount_to_sell: u64) -> Result<()> {
        let cpi_accounts = Transfer {
            from:  self.seller_token_account.to_account_info(),
            to: self.order_token_vault.to_account_info(),
            authority: self.seller.to_account_info(),
        };
        transfer(
            CpiContext::new(self.token_program.to_account_info(), cpi_accounts),
            amount_to_sell
        )
    }
}

impl<'info> RedeemOrder<'info> {
    pub fn send_tokens_from_order_to_buyer(&self, tokens_amount: u64) -> Result<()> {
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
            tokens_amount
        )
    }

    /// Close order account, order token account and remove order from the pool orders storage
    pub fn close_order(&mut self) -> Result<()> {
        self.pool_account.remove_order(self.order.to_account_info().key)?;

        // self.accounts.order.to_account_info().close(); TODO close order account

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
