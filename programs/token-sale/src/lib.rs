use anchor_lang::prelude::*;
use anchor_spl::token::{self, TokenAccount, Token, Mint};
use anchor_spl::associated_token::AssociatedToken;
mod account; use account::*;
mod context; use context::*;
mod error; use error::ErrorCode;
mod round; use round::*;
use std::convert::TryFrom;

declare_id!("Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS");

#[program]
pub mod token_sale {
    use super::*;

    pub fn initialize(
        ctx: Context<Initialize>,
        round_start_at: u64,
        end_at: u64,
        buying_duration: u32,
        trading_duration: u32,
        token_price: u64,
        tokens_per_round: u64,
        pool_bump: u8,
        amount_to_sell: u64,
    ) -> Result<()> {
        if token_price == 0 {
            return err!(ErrorCode::TokenPriceZero);
        }

        if amount_to_sell > ctx.accounts.tokens_for_distribution.amount {
            return err!(ErrorCode::NotEnoughTokensForSale);
        }

        // TODO check round_start_at is in the future or now
        // TODO check end_at is in the future and more than one full period

        let pool_account = &mut ctx.accounts.pool_account;

        pool_account.bump = pool_bump; // TODO change to *ctx.bumps.get(???)
        pool_account.owner = ctx.accounts.distribution_authority.key();
        pool_account.selling_mint = ctx.accounts.selling_mint.key();
        pool_account.vault_selling = ctx.accounts.vault_selling.key();
        pool_account.round_start_at = round_start_at;
        pool_account.end_at = end_at;
        pool_account.buying_duration = buying_duration;
        pool_account.trading_duration = trading_duration;
        pool_account.token_price = token_price;
        pool_account.tokens_per_round = tokens_per_round;
        pool_account.current_round = Round::Buying as u8;
        pool_account.last_round_trading_amount = 0;
        pool_account.orders = Vec::new();

        let token_program = ctx.accounts.token_program.to_account_info();
        let from = ctx.accounts.tokens_for_distribution.to_account_info();
        let to = ctx.accounts.vault_selling.to_account_info();
        let authority = ctx.accounts.distribution_authority.to_account_info();

        token::transfer(
            CpiContext::new(
                token_program,
                token::Transfer { from, to, authority },
            ),
            amount_to_sell
        )?;

        Ok(())
    }

    #[access_control(round_buying(&ctx.accounts.pool_account, &ctx.accounts.clock))]
    pub fn buy(
        ctx: Context<BuyTokens>,
        tokens_amount: u64,
    ) -> Result<()> {
        let pool = &mut ctx.accounts.pool_account;
        let vault_selling = &mut ctx.accounts.vault_selling;
        let buyer = &mut ctx.accounts.buyer;
        let buyer_token_account = &mut ctx.accounts.buyer_token_account;
        let token_price = pool.token_price;
        let token_program = &ctx.accounts.token_program;

        if vault_selling.amount < tokens_amount {
            return err!(ErrorCode::InsufficientTokensInVault);
        }

        let lamports_amount = tokens_amount * token_price;

        if **buyer.to_account_info().try_borrow_lamports()? < lamports_amount {
            return err!(ErrorCode::InsufficientLamportsToBuyTokens);
        }

        // Transfer lamport form the buyer to the pool
        send_lamports(buyer.to_account_info(), pool.to_account_info(), lamports_amount)?;

        // Transfer tokens form the pool to the buyer
        let seeds = &[
            ctx.accounts.selling_mint.to_account_info().key.as_ref(),
            &[pool.bump]
        ];

        token::transfer(
            CpiContext::new_with_signer(
                token_program.to_account_info(),
                token::Transfer {
                    to: buyer_token_account.to_account_info(),
                    from: vault_selling.to_account_info(),
                    authority: pool.to_account_info(),
                },
                &[&seeds[..]],
            ),
            tokens_amount
        )?;

        Ok(())
    }

    #[access_control(can_switch_to_trading_round(&ctx.accounts.pool_account, &ctx.accounts.clock))]
    pub fn switch_to_trading(ctx: Context<SwitchToTrading>) -> Result<()> {
        let pool = &mut ctx.accounts.pool_account;
        pool.round_start_at = ctx.accounts.clock.unix_timestamp as u64;
        pool.current_round = Round::Trading as u8;
        // Store the new round trading amount. For now it's zero.
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
        let pool = &mut ctx.accounts.pool_account;
        let order = &mut ctx.accounts.order;

        if amount_to_sell < 1 {
            return err!(ErrorCode::SellingToFewTokens);
        }

        if ctx.accounts.seller_token_account.amount < amount_to_sell {
            return err!(ErrorCode::InsufficientTokensInVault);
        }

        let token_program = ctx.accounts.token_program.to_account_info();
        let from = ctx.accounts.seller_token_account.to_account_info();
        let to = ctx.accounts.order_token_vault.to_account_info();
        let authority = ctx.accounts.seller.to_account_info();

        token::transfer(
            CpiContext::new(
                token_program,
                token::Transfer { from, to, authority },
            ),
            amount_to_sell
        )?;

        order.bump = order_bump;
        order.token_price = price_for_token;
        order.token_vault = ctx.accounts.order_token_vault.key();
        order.owner = ctx.accounts.seller.key();
        order.token_amount = amount_to_sell;

        pool.orders.push(OrderAddress {
            pubkey: *order.to_account_info().unsigned_key(),
            bump: order_bump,
        });

        Ok(())
    }

    #[access_control(round_trading(&ctx.accounts.pool_account, &ctx.accounts.clock))]
    pub fn redeem_order(
        ctx: Context<RedeemOrder>,
        tokens_amount: u64, // amount of tokens to buy
    ) -> Result<()> {
        let buyer = &mut ctx.accounts.buyer;
        let order_owner = &ctx.accounts.order_owner;
        let order = &ctx.accounts.order;

        if tokens_amount < 1 {
            return err!(ErrorCode::BuyingToFewTokens);
        }

        if ctx.accounts.order_token_vault.amount < tokens_amount {
            return err!(ErrorCode::InsufficientTokensInVault);
        }

        let lamports_amount = tokens_amount * order.token_price;

        if **buyer.to_account_info().try_borrow_lamports()? < lamports_amount {
            return err!(ErrorCode::InsufficientLamportsToBuyTokens);
        }

        // Transfer lamport form the buyer to the order's owner
        send_lamports(buyer.to_account_info(), order_owner.to_account_info(), lamports_amount)?;

        // Transfer tokens from the order to the buyer
        let token_program = ctx.accounts.token_program.to_account_info();
        let from = ctx.accounts.order_token_vault.to_account_info();
        let to = ctx.accounts.buyer_token_account.to_account_info();
        let authority = ctx.accounts.order.to_account_info();

        let seeds = &[
            Order::PDA_SEED,
            order.owner.as_ref(),
            &[order.bump]
        ];

        token::transfer(
            CpiContext::new_with_signer(
                token_program,
                token::Transfer { to, from, authority },
                &[&seeds[..]],
            ),
            tokens_amount
        )?;

        // Reduce the token amount in the order
        let order = &mut ctx.accounts.order;
        order.token_amount -= tokens_amount;

        // Save the total trading amount in lamports
        let pool = &mut ctx.accounts.pool_account;
        pool.last_round_trading_amount += lamports_amount;

        // TODO return rent-exempt lamports to order owner if the order is empty

        Ok(())
    }

    pub fn close_order(ctx: Context<CloseOrder>) -> Result<()> {
        // TODO remove order from pool.orders
        // TODO return tokens to the account provided by the caller,
        //   it has to be owned by the order's owner
        // TODO return rent (destroy accounts)

        unimplemented!()
    }

    #[access_control(can_switch_to_buying_round(&ctx.accounts.pool_account, &ctx.accounts.clock))]
    pub fn switch_to_buying(ctx: Context<SwitchToBuying>) -> Result<()> {
        let pool = &mut ctx.accounts.pool_account;
        pool.round_start_at = ctx.accounts.clock.unix_timestamp as u64;
        pool.current_round = Round::Buying as u8;

        // TODO change token price (newPrice = exPrice * 1.03 + 0.000004) cooefs to the pool
        // TODO change token amount to sale (total trading amount in lamports)
            // TODO change last_number_of_trades no this field

        Ok(())
    }

    /// The program could be terminated after the `end_at` time has passed
    /// or if no one deal have taken place during the last trade round.
    pub fn terminate(ctx: Context<End>) -> Result<()> {
        unimplemented!();
    }
}

// Is buying round running?
fn round_buying<'info>(
    pool: &Account<'info, PoolAccount>,
    clock: &Sysvar<'info, Clock>,
) -> Result<()> {
    let round= Round::try_from(pool.current_round).unwrap();

    if round != Round::Buying {
        return err!(ErrorCode::NotBuyingRound);
    }

    let round_ends_at = pool.round_start_at + pool.buying_duration as u64;

    if round_ends_at <= clock.unix_timestamp as u64 {
        return err!(ErrorCode::BuyingOver);
    }

    Ok(())
}

// Is trading round running?
fn round_trading<'info>(
    pool: &Account<'info, PoolAccount>,
    clock: &Sysvar<'info, Clock>,
) -> Result<()> {
    let round = Round::try_from(pool.current_round).unwrap();

    if round != Round::Trading {
        return err!(ErrorCode::NotTradingRound);
    }

    let round_ends_at = pool.round_start_at + pool.trading_duration as u64;

    if round_ends_at <= clock.unix_timestamp as u64 {
        return err!(ErrorCode::TradingOver);
    }

    Ok(())
}

// Is it available to switch from buying to trading round?
fn can_switch_to_trading_round<'info>(
    pool: &Account<'info, PoolAccount>,
    clock: &Sysvar<'info, Clock>,
) -> Result<()> {
    if pool.end_at <= clock.unix_timestamp as u64 {
        return err!(ErrorCode::IDOOver);
    }

    let round= Round::try_from(pool.current_round).unwrap();

    if round == Round::Trading {
        return err!(ErrorCode::AlreadyTrading);
    }

    let round_ends_at = pool.round_start_at + pool.buying_duration as u64;

    if round_ends_at < clock.unix_timestamp as u64 {
        return err!(ErrorCode::BuyingCannotBeStopped);
    }

    Ok(())
}

// Is it available to switch from trading to buying round?
fn can_switch_to_buying_round<'info>(
    pool: &Account<'info, PoolAccount>,
    clock: &Sysvar<'info, Clock>,
) -> Result<()> {
    if pool.end_at <= clock.unix_timestamp as u64 {
        return err!(ErrorCode::IDOOver);
    }

    let round= Round::try_from(pool.current_round).unwrap();

    if round == Round::Buying {
        return err!(ErrorCode::AlreadyBuying);
    }

    let round_ends_at = pool.round_start_at + pool.trading_duration as u64;

    if round_ends_at < clock.unix_timestamp as u64 {
        return err!(ErrorCode::TradingCannotBeStopped);
    }

    Ok(())
}

fn send_lamports<'a>(from: AccountInfo<'a>, to: AccountInfo<'a>, amount: u64) -> Result<()> {
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
