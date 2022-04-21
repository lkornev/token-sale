use anchor_lang::prelude::{AnchorSerialize, AnchorDeserialize};

#[derive(PartialEq, Eq, AnchorSerialize, AnchorDeserialize, Clone)]
pub enum Round {
    Buying,
    Trading,
}
