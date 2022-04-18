use crate::error::ErrorCode;
use std::convert::TryFrom;
use anchor_lang::prelude::{Error};
use anchor_lang::err;

#[derive(PartialEq, Eq)]
pub enum Round {
    Buying = 0,
    Trading = 1,
}

impl TryFrom<u8> for Round {
    type Error = Error;

    fn try_from(orig: u8) -> Result<Self, Self::Error> {
        match orig {
            0 => Ok(Round::Buying),
            1 => Ok(Round::Trading),
            _ => err!(ErrorCode::RoundTypeMismatch),
        }
    }
}