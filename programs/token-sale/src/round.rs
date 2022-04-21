use crate::error::ErrorCode;
use std::convert::TryFrom;
use anchor_lang::prelude::{Error, AnchorSerialize, AnchorDeserialize};
use anchor_lang::err;

#[derive(PartialEq, Eq, AnchorSerialize, AnchorDeserialize, Clone)]
pub enum Round {
    Buying,
    Trading,
}
