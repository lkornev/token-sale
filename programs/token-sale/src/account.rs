use anchor_lang::prelude::*;

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
    pub end_at: u64,
    /// Seconds to pass before the end of the sales round
    pub buying_duration: u32,
    /// Seconds to pass before the end of the trade round
    pub trading_duration: u32,
    /// Current price of the selling token. Could be changed after trade rounds.
    /// Represents the amount of lamports(!) for the one minimal part of the token
    pub token_price: u64,
    /// The amount of tokens to be sold in the selling round.
    /// The tokens for sale in a particular round than not sold will be burned.
    pub tokens_per_round: u64,
    /// Could be selling round (0) or trading round (1)
    pub current_round: u8, // enum Round
    /// UNIX timestamp when the current round started begins
    pub round_start_at: u64,
    /// Amount of lamports raised in the last trading round
    pub last_round_trading_amount: u64,
    /// The list of selling tokens orders
    pub orders: Vec<OrderAddress>,
}

// TODO take during initialization as an argument
pub const MAX_ORDERS_NUM: usize = 100;

impl PoolAccount {
    pub const SPACE: usize = 1 + 32 * 5 + 8 + 8 + 4 + 4 + 8 + 8 + 1 + 8
        + (OrderAddress::SPACE * MAX_ORDERS_NUM + 24);
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
    pub token_amount: u64,
}

impl Order {
    pub const SPACE: usize = 1 + 32 + 32 + 8 + 8;
    pub const PDA_KEY: &'static str = "order";
    pub const PDA_SEED: & 'static [u8] = Self::PDA_KEY.as_bytes();
}
