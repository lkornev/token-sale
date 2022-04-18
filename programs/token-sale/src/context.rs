use anchor_lang::prelude::*;
use anchor_spl::token::{self, TokenAccount, Token, Mint};
use anchor_spl::associated_token::AssociatedToken;
use crate::account::*;

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
    pub token_program: Program<'info, Token>,
    pub rent: Sysvar<'info, Rent>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}

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

#[derive(Accounts)]
pub struct SwitchToTrading<'info> {
    #[account(
        mut,
        seeds = [pool_account.selling_mint.as_ref()],
        bump = pool_account.bump,
    )]
    pub pool_account: Account<'info, PoolAccount>,
    pub clock: Sysvar<'info, Clock>,
}

#[derive(Accounts)]
pub struct PlaceOrder<'info> {
    #[account(
        mut,
        seeds = [selling_mint.to_account_info().key.as_ref()],
        bump = pool_account.bump,
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


#[derive(Accounts)]
pub struct RedeemOrder<'info> {
    #[account(
        mut,
        seeds = [selling_mint.to_account_info().key.as_ref()],
        bump = pool_account.bump,
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
    pub order_owner: AccountInfo<'info>,
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

#[derive(Accounts)]
pub struct SwitchToBuying<'info> {
    #[account(
        mut,
        seeds = [pool_account.selling_mint.as_ref()],
        bump = pool_account.bump,
    )]
    pub pool_account: Account<'info, PoolAccount>,
    pub clock: Sysvar<'info, Clock>,
}

#[derive(Accounts)]
pub struct End {}
