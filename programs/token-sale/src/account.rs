use anchor_lang::prelude::*;
use crate::ErrorCode;
use crate::Round;
use crate::currency::{Tokens, Lamports};

/// The main state of the program
#[account]
pub struct PoolAccount {
    pub bump: u8,
    /// The owner of the token sale program.
    pub owner: Pubkey,
    /// Describes the type of the selling tokens.
    /// The mint itself does not need to be under control of the token sale owner.
    pub selling_mint: Pubkey,
    /// Describes the type of the tokens that are accepted as a payment for selling tokens.
    pub payment_mint: Pubkey,
    /// The vault with selling tokens.
    pub vault_selling: Pubkey,
    /// UNIX timestamp when the selling can be terminated
    pub end_at: u32,
    /// Seconds to pass before the end of the sales round
    pub buying_duration: u32,
    /// Seconds to pass before the end of the trade round
    pub trading_duration: u32,
    /// Current price of the selling token. Could be changed after trade rounds.
    /// Represents the amount of lamports for the one minimal part of the token
    pub token_price: u64,
    /// Could be selling round or trading round
    pub current_round: Round,
    /// UNIX timestamp when the current round started begins
    pub round_start_at: u32,
    /// The coefficients that define the value of the token in the next buying round
    /// using the formula: next_token_price = token_price * coeff_a + coeff_b
    pub coeff_a: f32,
    pub coeff_b: u32,
    /// The list of selling tokens orders
    /// https://book.anchor-lang.com/anchor_references/space.html
    pub orders: Vec<OrderAddress>,
}

pub const MAX_ORDERS_NUM: usize = 100;

impl PoolAccount {
    pub const SPACE: usize = 1 + 32 * 4 + 4 * 8 + 1 + 4 + 4 + 4
        + (OrderAddress::SPACE * MAX_ORDERS_NUM + 4);

    pub fn remove_order(&mut self, order_address: &Pubkey) -> Result<()> {
        let index = self.orders.iter().position(|x| &x.pubkey == order_address);
        match index {
            Some(index) => {
                self.orders.remove(index);
                Ok(())
            },
            None => err!(ErrorCode::OrderNotFoundInPool),
        }
    }

    pub fn add_order(&mut self, pubkey: Pubkey, bump: u8) {
        self.orders.push(OrderAddress { pubkey, bump });
    }
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct OrderAddress {
    pub bump: u8,
    pub pubkey: Pubkey,
}

impl OrderAddress {
    pub const SPACE: usize = 1 + 32;
}

#[account]
pub struct Order {
    pub bump: u8,
    /// The owner of the order. One will receive lamports to that address upon order redemption.
    pub owner: Pubkey,
    /// The temp storage with tokens for sale
    pub token_vault: Pubkey,
    /// The price for one token
    pub token_price: u64,
    /// Amount of tokens inside token_vault
    pub token_amount: Tokens,
}

impl Order {
    pub const SPACE: usize = 1 + 32 + 32 + 8 + 8;
    pub const PDA_KEY: &'static str = "order";
    pub const PDA_SEED: & 'static [u8] = Self::PDA_KEY.as_bytes();
}
