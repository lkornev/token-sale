use anchor_lang::prelude::*;
use crate::Round;
use crate::currency::Tokens;

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
    pub end_at: i64,
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
    pub round_start_at: i64,
    /// The coefficients that define the value of the token in the next buying round
    /// using the formula: next_token_price = token_price * coeff_a + coeff_b
    pub coeff_a: f32,
    pub coeff_b: u32,
}

impl PoolAccount {
    pub const SPACE: usize = 1 + 32 * 4 + 8 + 4 + 4 + 8 + 1 + 8 + 4 + 4;
}

#[account]
pub struct Order {
    /// Have all the tokens already been sold?
    pub is_empty: bool,
    /// The time when the order created
    pub created_at: i64,
    /// The owner of the order. One will receive lamports to that address upon order redemption.
    pub owner: Pubkey,
    /// Amount of tokens inside token_vault
    pub token_amount: Tokens,
    /// The price for one token
    pub token_price: u64,
    /// The temp storage with tokens for sale
    pub token_vault: Pubkey,
    pub bump: u8,
}

impl Order {
    pub const SPACE: usize = 1 + 8 + 32 + 8 + 8 + 32 + 1;
    pub const PDA_KEY: &'static str = "order";
    pub const PDA_SEED: & 'static [u8] = Self::PDA_KEY.as_bytes();
}
