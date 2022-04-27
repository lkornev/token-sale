use anchor_lang::prelude::*;
use anchor_spl::token::{Token, Mint};
use crate::account::*;

#[derive(Accounts)]
pub struct WithdrawLamports<'info> {
    #[account(
        mut,
        seeds = [selling_mint.to_account_info().key.as_ref()],
        bump = pool_account.bump,
        has_one = selling_mint,
        has_one = owner,
    )]
    pub pool_account: Account<'info, PoolAccount>,
    pub selling_mint: Box<Account<'info, Mint>>,
    #[account(mut)]
    pub owner: Signer<'info>,
    pub clock: Sysvar<'info, Clock>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

impl<'info> WithdrawLamports<'info> {
    pub fn send_lamports_from_pool_to_owner(&mut self) -> Result<()> {
        let pool_info = self.pool_account.to_account_info();
        let pool_data_len = pool_info.try_data_len()?;
        let pool_minimum_rent_exempt_balance = self.rent.minimum_balance(pool_data_len);
        let all_pool_lamports = **pool_info.try_borrow_lamports()?;
        let available_lamports = all_pool_lamports - pool_minimum_rent_exempt_balance;

        **pool_info.try_borrow_mut_lamports()? -= available_lamports;
        **self.owner.try_borrow_mut_lamports()? += available_lamports;

        Ok(())
    }
}
