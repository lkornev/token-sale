use anchor_lang::prelude::*;
use crate::Lamports;

pub fn send_lamports<'a>(from: AccountInfo<'a>, to: AccountInfo<'a>, amount: Lamports) -> Result<()> {
    let ix = anchor_lang::solana_program::system_instruction::transfer(
        &from.key(),
        &to.key(),
        amount.into(),
    );

    anchor_lang::solana_program::program::invoke(
        &ix,
        &[
            from.to_account_info(),
            to.to_account_info(),
        ],
    ).map_err(|err| err.into())
}
