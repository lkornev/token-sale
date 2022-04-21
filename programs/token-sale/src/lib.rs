use anchor_lang::prelude::*;
use anchor_spl::token::{self, TokenAccount, Token, Mint};
use anchor_spl::associated_token::AssociatedToken;
mod account; use account::*;
mod context; use context::*;
mod error; use error::ErrorCode;
mod round; use round::*;
mod access_control; use access_control::*;
mod send_assets; use send_assets::*;

declare_id!("Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS");

#[program]
pub mod token_sale {
    use super::*;

    pub fn initialize(
        ctx: Context<Initialize>,
        round_start_at: u32,
        end_at: u32,
        buying_duration: u32,
        trading_duration: u32,
        token_price: u32,
        tokens_per_round: u64,
        pool_bump: u8,
        amount_to_sell: u64,
        coeff_a: f32,
        coeff_b: u32,
    ) -> Result<()> {
        require!(token_price != 0, ErrorCode::TokenPriceZero);

        let tokens_for_sale = ctx.accounts.tokens_for_distribution.amount;
        require!(amount_to_sell <= tokens_for_sale, ErrorCode::NotEnoughTokensForSale);

        let now = ctx.accounts.clock.unix_timestamp as u32;
        let full_cycle = now + buying_duration + trading_duration;
        require!(round_start_at >= now, ErrorCode::FirstRoundAlreadyStarted);
        require!(end_at >= full_cycle, ErrorCode::EndsBeforeFullCircle);

        let pool_account = &mut ctx.accounts.pool_account;

        pool_account.bump = pool_bump;
        pool_account.owner = ctx.accounts.distribution_authority.key();
        pool_account.selling_mint = ctx.accounts.selling_mint.key();
        pool_account.vault_selling = ctx.accounts.vault_selling.key();
        pool_account.round_start_at = round_start_at;
        pool_account.end_at = end_at;
        pool_account.buying_duration = buying_duration;
        pool_account.trading_duration = trading_duration;
        pool_account.token_price = token_price;
        pool_account.tokens_per_round = tokens_per_round;
        pool_account.current_round = Round::Buying;
        pool_account.last_round_trading_amount = 0;
        pool_account.orders = Vec::new();
        pool_account.coeff_a = coeff_a;
        pool_account.coeff_b = coeff_b;

        ctx.accounts.send_tokens_to_pool(amount_to_sell)
    }

    #[access_control(round_buying(&ctx.accounts.pool_account, &ctx.accounts.clock))]
    pub fn buy(
        ctx: Context<BuyTokens>,
        tokens_amount: u64, // TODO use NewType pattern to distinguish Tokens and Lamports (and conversion).
    ) -> Result<()> {
        let vault_selling = &mut ctx.accounts.vault_selling;
        require!(vault_selling.amount >= tokens_amount, ErrorCode::InsufficientTokensInVault);

        let token_price = ctx.accounts.pool_account.token_price;
        let lamports_amount = tokens_amount * token_price as u64;
        let buyer_lamports = ** ctx.accounts.buyer.to_account_info().try_borrow_lamports()?;
        require!(buyer_lamports >= lamports_amount, ErrorCode::InsufficientLamportsToBuyTokens);

        let buyer = &mut ctx.accounts.buyer;
        let pool = &mut ctx.accounts.pool_account;
        send_lamports(buyer.to_account_info(), pool.to_account_info(), lamports_amount)?;
        ctx.accounts.send_tokens_from_pool_to_buyer(tokens_amount)?;

        Ok(())
    }

    #[access_control(can_switch_to_trading_round(&ctx.accounts.pool_account, &ctx.accounts.clock))]
    pub fn switch_to_trading(ctx: Context<SwitchToTrading>) -> Result<()> {
        let pool = &mut ctx.accounts.pool_account;
        pool.round_start_at = ctx.accounts.clock.unix_timestamp as u32;
        pool.current_round = Round::Trading;
        pool.last_round_trading_amount = 0;

        Ok(())
    }

    #[access_control(round_trading(&ctx.accounts.pool_account, &ctx.accounts.clock))]
    pub fn place_order(
        ctx: Context<PlaceOrder>,
        order_bump: u8,
        amount_to_sell: u64,
        price_for_token: u64,
    ) -> Result<()> {
        require!(amount_to_sell >= 1, ErrorCode::SellingToFewTokens);

        let seller_tokens = ctx.accounts.seller_token_account.amount;
        require!(seller_tokens > amount_to_sell, ErrorCode::InsufficientTokensInVault);

        ctx.accounts.send_tokens_from_seller_to_order(amount_to_sell)?;

        let order = &mut ctx.accounts.order;
        order.bump = order_bump;
        order.token_price = price_for_token;
        order.token_vault = ctx.accounts.order_token_vault.key();
        order.owner = ctx.accounts.seller.key();
        order.token_amount = amount_to_sell;

        ctx.accounts.pool_account.add_order(order.to_account_info().key(), order_bump);

        Ok(())
    }

    #[access_control(round_trading(&ctx.accounts.pool_account, &ctx.accounts.clock))]
    pub fn redeem_order(
        ctx: Context<RedeemOrder>,
        tokens_amount: u64, // amount of tokens to buy
    ) -> Result<()> {
        let is_buying_everything = ctx.accounts.order_token_vault.amount == tokens_amount;
        require!(tokens_amount >= 1, ErrorCode::BuyingToFewTokens);

        let order_tokens = ctx.accounts.order_token_vault.amount;
        require!(order_tokens >= tokens_amount, ErrorCode::InsufficientTokensInVault);

        let lamports_amount = tokens_amount * ctx.accounts.order.token_price;
        let buyer_lamports = **ctx.accounts.buyer.to_account_info().try_borrow_lamports()?;
        require!(buyer_lamports >= lamports_amount, ErrorCode::InsufficientLamportsToBuyTokens);

        // Send lamports to the order's owner, send tokens to the buyer
        send_lamports(ctx.accounts.buyer.to_account_info(), ctx.accounts.order_owner.to_account_info(), lamports_amount)?;
        ctx.accounts.send_tokens_from_order_to_buyer(tokens_amount)?;

        // Reduce the token amount in the order
        let order = &mut ctx.accounts.order;
        order.token_amount -= tokens_amount;

        // Save the total trading amount in lamports
        let pool = &mut ctx.accounts.pool_account;
        pool.last_round_trading_amount += lamports_amount;

        // Return rent-exempt lamports to the owner of the order by closing account
        if is_buying_everything { ctx.accounts.close_order()? }

        Ok(())
    }

    pub fn close_order(ctx: Context<CloseOrder>) -> Result<()> {
        ctx.accounts.pool_account.remove_order(ctx.accounts.order.to_account_info().key)?;
        ctx.accounts.sent_tokens_from_order_to_owner()
    }

    #[access_control(can_switch_to_buying_round(&ctx.accounts.pool_account, &ctx.accounts.clock))]
    pub fn switch_to_buying(ctx: Context<SwitchToBuying>) -> Result<()> {
        let pool = &mut ctx.accounts.pool_account;
        pool.round_start_at = ctx.accounts.clock.unix_timestamp as u32;
        pool.current_round = Round::Buying;

        const PRECISENESS: u32 = 10000;
        pool.token_price = pool.token_price
            .checked_mul((pool.coeff_a * PRECISENESS as f32) as u32).unwrap()
            .checked_div(PRECISENESS).unwrap()
            .checked_add(pool.coeff_b).unwrap();

        Ok(())
    }

    /// The program could be terminated after the `pool_account.end_at` time has passed
    /// or if no one deal have taken place during the last trade round.
    pub fn terminate(ctx: Context<Terminate>) -> Result<()> {
        // TODO withdraw tokens from vault_payment. Sign by pool_account.owner
        // TODO burn all unsold tokens
        // TODO destroy all accounts and the program itself
        unimplemented!();
    }
}
