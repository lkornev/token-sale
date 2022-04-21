use anchor_lang::prelude::*;
use crate::account::PoolAccount;
use std::ops::{Add, AddAssign, Sub, SubAssign};

// Currently the syntax Tokens(u64) is not supported
// https://github.com/project-serum/anchor/issues/1719
#[derive(PartialEq, Eq, PartialOrd, Ord, AnchorSerialize, AnchorDeserialize, Clone, Copy)]
pub struct Tokens { tokens: u64 }

impl Tokens {
    pub fn new(amount: u64) -> Self {
        Tokens { tokens: amount }
    }
}

impl From<Tokens> for u64 {
    fn from(tokens: Tokens) -> Self {
        tokens.tokens
    }
}

impl Add for Tokens {
    type Output = Self;

    fn add(self, other: Self) -> Self::Output {
        Self { tokens: self.tokens + other.tokens }
    }
}

impl AddAssign for Tokens {
    fn add_assign(&mut self, other: Self) {
        *self = Self { tokens: self.tokens + other.tokens};
    }
}

impl Sub for Tokens {
    type Output = Self;

    fn sub(self, other: Self) -> Self::Output {
        Self { tokens: self.tokens - other.tokens }
    }
}

impl SubAssign for Tokens {
    fn sub_assign(&mut self, other: Self) {
        *self = Self { tokens: self.tokens - other.tokens};
    }
}

#[derive(PartialEq, Eq, PartialOrd, Ord, AnchorSerialize, AnchorDeserialize, Clone, Copy)]
pub struct Lamports { lamports: u64 }

impl Lamports {
    pub fn new(amount: u64) -> Self {
        Lamports { lamports: amount }
    }
}

impl From<Lamports> for u64 {
    fn from(lamports: Lamports) -> Self {
        lamports.lamports
    }
}

impl Add for Lamports {
    type Output = Self;

    fn add(self, other: Self) -> Self::Output {
        Self { lamports: self.lamports + other.lamports }
    }
}

impl AddAssign for Lamports {
    fn add_assign(&mut self, other: Self) {
        *self = Self { lamports: self.lamports + other.lamports};
    }
}

impl Sub for Lamports {
    type Output = Self;

    fn sub(self, other: Self) -> Self::Output {
        Self { lamports: self.lamports - other.lamports }
    }
}

impl SubAssign for Lamports {
    fn sub_assign(&mut self, other: Self) {
        *self = Self { lamports: self.lamports - other.lamports};
    }
}

impl PoolAccount {
    pub fn try_tokens_to_lamports(&self, tokens: Tokens) -> Option<Lamports> {
        let lamports_amount = u64::from(tokens.clone()).checked_mul(self.token_price as u64);
        lamports_amount.map(|lamports_amount| Lamports::new(lamports_amount))
    }

    pub fn try_lamports_to_tokens(&self, lamports: Lamports) -> Option<Tokens> {
        let tokens_amount = u64::from(lamports).checked_div(self.token_price as u64);
        tokens_amount.map(|tokens_amount| Tokens::new(tokens_amount))
    }
}
