use anchor_lang::prelude::*;
use crate::account::*;
use crate::round::*;
use crate::error::ErrorCode;
use crate::Lamports;
use anchor_spl::token::TokenAccount;

// Is buying round running?
pub fn round_buying<'info>(
    pool: &Account<'info, PoolAccount>,
    clock: &Sysvar<'info, Clock>,
) -> Result<()> {
    if pool.current_round != Round::Buying {
        return err!(ErrorCode::NotBuyingRound);
    }

    let round_ends_at = pool.round_start_at + pool.buying_duration;

    if round_ends_at <= clock.unix_timestamp as u32 {
        return err!(ErrorCode::BuyingOver);
    }

    Ok(())
}

// Is trading round running?
pub fn round_trading<'info>(
    pool: &Account<'info, PoolAccount>,
    clock: &Sysvar<'info, Clock>,
) -> Result<()> {
    if pool.current_round != Round::Trading {
        return err!(ErrorCode::NotTradingRound);
    }

    let round_ends_at = pool.round_start_at + pool.trading_duration;

    if round_ends_at <= clock.unix_timestamp as u32 {
        return err!(ErrorCode::TradingOver);
    }

    Ok(())
}

// Is it available to switch from buying to trading round?
pub fn can_switch_to_trading_round<'info>(
    pool: &Account<'info, PoolAccount>,
    clock: &Sysvar<'info, Clock>,
) -> Result<()> {
    if pool.end_at <= clock.unix_timestamp as u32 {
        return err!(ErrorCode::IDOOver);
    }

    if pool.current_round == Round::Trading {
        return err!(ErrorCode::AlreadyTrading);
    }

    let round_ends_at = pool.round_start_at + pool.buying_duration;

    if round_ends_at < clock.unix_timestamp as u32 {
        return err!(ErrorCode::BuyingCannotBeStopped);
    }

    Ok(())
}

// Is it available to switch from trading to buying round?
pub fn can_switch_to_buying_round<'info>(
    pool: &Account<'info, PoolAccount>,
    clock: &Sysvar<'info, Clock>,
) -> Result<()> {
    if pool.end_at <= clock.unix_timestamp as u32 {
        return err!(ErrorCode::IDOOver);
    }

    if pool.current_round == Round::Buying {
        return err!(ErrorCode::AlreadyBuying);
    }

    let round_ends_at = pool.round_start_at + pool.trading_duration;

    if round_ends_at < clock.unix_timestamp as u32 {
        return err!(ErrorCode::TradingCannotBeStopped);
    }

    Ok(())
}

pub fn can_terminate<'info>(
    pool: &Account<'info, PoolAccount>,
    vault_selling: Account<'info, TokenAccount>,
    clock: &Sysvar<'info, Clock>,
) -> Result<()> {
    if pool.end_at > clock.unix_timestamp as u32 {
        return err!(ErrorCode::IDONotOver);
    }

    if pool.orders.len() > 0 {
        return err!(ErrorCode::IDOHasActiveOrders);
    }

    Ok(())
}
