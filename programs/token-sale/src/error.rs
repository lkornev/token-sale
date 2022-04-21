use anchor_lang::prelude::*;

#[error_code]
pub enum ErrorCode {
    #[msg("Amount of tokens for sale is larger than token amount inside `tokens_for_distribution` account.")]
    NotEnoughTokensForSale,
    #[msg("Not enough tokens in the vault.")]
    InsufficientTokensInVault,
    #[msg("Not enough lamports to buy the requested amount of tokens.")]
    InsufficientLamportsToBuyTokens,
    #[msg("Round type error. Available values Buying (0) and Trading (1)")]
    RoundTypeMismatch,
    #[msg("Current round is trading. You can only trade with other users.")]
    NotBuyingRound,
    #[msg("Current round is buying. You can only buy tokens from the platform.")]
    NotTradingRound,
    #[msg("Buying round is over.")]
    BuyingOver,
    #[msg("Trading round is over.")]
    TradingOver,
    #[msg("Current round is already trading")]
    AlreadyTrading,
    #[msg("Current round is already buying")]
    AlreadyBuying,
    #[msg("Buying cannot be stopped right now. Please wait till the end of the round.")]
    BuyingCannotBeStopped,
    #[msg("Trading cannot be stopped right now. Please wait till the end of the round.")]
    TradingCannotBeStopped,
    #[msg("IDO is over.")]
    IDOOver,
    #[msg("Minimal amount of tokens to buy is 1")]
    BuyingToFewTokens,
    #[msg("Minimal amount of tokens to sell is 1")]
    SellingToFewTokens,
    #[msg("Token price couldn't be zero")]
    TokenPriceZero,
    #[msg("The order is not found in the pool account")]
    OrderNotFoundInPool,
    #[msg("The first round must start in the future or now")]
    FirstRoundAlreadyStarted,
    #[msg("The IDO must be at least full circle long")]
    EndsBeforeFullCircle
}
