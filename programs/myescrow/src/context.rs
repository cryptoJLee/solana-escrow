use anchor_lang::prelude::*;
use anchor_spl::token::{CloseAccount, Mint, SetAuthority, TokenAccount, Transfer};
// use anchor_lang::solana_program::system_program;
use crate::accountdefs::EscrowAccount;

#[derive(Accounts)]
#[instruction(vault_account_bump: u8, initializer_amount: u64)]
pub struct InitializeEscrow<'info> {
  #[account(mut, signer)]
  pub initializer: AccountInfo<'info>,
  pub mint: Account<'info, Mint>,
  #[account(
        init,
        seeds = [b"token-seed".as_ref()],
        bump = vault_account_bump,
        payer = initializer,
        token::mint = mint,
        token::authority = initializer,
    )]
  pub vault_account: Account<'info, TokenAccount>,
  #[account(
        mut,
        constraint = initializer_deposit_token_account.amount >= initializer_amount
    )]
  pub initializer_deposit_token_account: Account<'info, TokenAccount>,
  pub initializer_receive_token_account: Account<'info, TokenAccount>,
  #[account(zero)]
  pub escrow_account: ProgramAccount<'info, EscrowAccount>,
  pub system_program: AccountInfo<'info>,
  pub rent: Sysvar<'info, Rent>,
  pub token_program: AccountInfo<'info>,
}

#[derive(Accounts)]
pub struct CancelEscrow<'info> {
  #[account(mut, signer)]
  pub initializer: AccountInfo<'info>,
  #[account(mut)]
  pub vault_account: Account<'info, TokenAccount>,
  pub vault_authority: AccountInfo<'info>,
  #[account(mut)]
  pub initializer_deposit_token_account: Account<'info, TokenAccount>,
  #[account(
        mut,
        constraint = escrow_account.initializer_key == *initializer.key,
        constraint = escrow_account.initializer_deposit_token_account == *initializer_deposit_token_account.to_account_info().key,
        close = initializer
    )]
  pub escrow_account: ProgramAccount<'info, EscrowAccount>,
  pub token_program: AccountInfo<'info>,
}

#[derive(Accounts)]
pub struct Exchange<'info> {
  #[account(signer)]
  pub taker: AccountInfo<'info>,
  #[account(mut)]
  pub taker_deposit_token_account: Account<'info, TokenAccount>,
  #[account(mut)]
  pub taker_receive_token_account: Account<'info, TokenAccount>,
  #[account(mut)]
  pub initializer_deposit_token_account: Account<'info, TokenAccount>,
  #[account(mut)]
  pub initializer_receive_token_account: Account<'info, TokenAccount>,
  #[account(mut)]
  pub initializer: AccountInfo<'info>,
  #[account(
        mut,
        constraint = escrow_account.taker_amount <= taker_deposit_token_account.amount,
        constraint = escrow_account.initializer_deposit_token_account == *initializer_deposit_token_account.to_account_info().key,
        constraint = escrow_account.initializer_receive_token_account == *initializer_receive_token_account.to_account_info().key,
        constraint = escrow_account.initializer_key == *initializer.key,
        close = initializer
    )]
  pub escrow_account: ProgramAccount<'info, EscrowAccount>,
  #[account(mut)]
  pub vault_account: Account<'info, TokenAccount>,
  pub vault_authority: AccountInfo<'info>,
  pub token_program: AccountInfo<'info>,
}

impl<'info> InitializeEscrow<'info> {
  pub fn into_transfer_to_pda_context(&self) -> CpiContext<'_, '_, '_, 'info, Transfer<'info>> {
    let cpi_accounts = Transfer {
      from: self
        .initializer_deposit_token_account
        .to_account_info()
        .clone(),
      to: self.vault_account.to_account_info().clone(),
      authority: self.initializer.clone(),
    };
    CpiContext::new(self.token_program.clone(), cpi_accounts)
  }

  pub fn into_set_authority_context(&self) -> CpiContext<'_, '_, '_, 'info, SetAuthority<'info>> {
    let cpi_accounts = SetAuthority {
      account_or_mint: self.vault_account.to_account_info().clone(),
      current_authority: self.initializer.clone(),
    };
    let cpi_program = self.token_program.to_account_info();
    CpiContext::new(cpi_program, cpi_accounts)
  }
}

impl<'info> CancelEscrow<'info> {
  pub fn into_transfer_to_initializer_context(
    &self,
  ) -> CpiContext<'_, '_, '_, 'info, Transfer<'info>> {
    let cpi_accounts = Transfer {
      from: self.vault_account.to_account_info().clone(),
      to: self
        .initializer_deposit_token_account
        .to_account_info()
        .clone(),
      authority: self.vault_authority.clone(),
    };
    let cpi_program = self.token_program.to_account_info();
    CpiContext::new(cpi_program, cpi_accounts)
  }

  pub fn into_close_context(&self) -> CpiContext<'_, '_, '_, 'info, CloseAccount<'info>> {
    let cpi_accounts = CloseAccount {
      account: self.vault_account.to_account_info().clone(),
      destination: self.initializer.clone(),
      authority: self.vault_authority.clone(),
    };
    let cpi_program = self.token_program.to_account_info();
    CpiContext::new(cpi_program, cpi_accounts)
  }
}

impl<'info> Exchange<'info> {
  pub fn into_transfer_to_initializer_context(
    &self,
  ) -> CpiContext<'_, '_, '_, 'info, Transfer<'info>> {
    let cpi_accounts = Transfer {
      from: self.taker_deposit_token_account.to_account_info().clone(),
      to: self
        .initializer_receive_token_account
        .to_account_info()
        .clone(),
      authority: self.taker.clone(),
    };
    let cpi_program = self.token_program.to_account_info();
    CpiContext::new(cpi_program, cpi_accounts)
  }

  pub fn into_transfer_to_taker_context(&self) -> CpiContext<'_, '_, '_, 'info, Transfer<'info>> {
    let cpi_accounts = Transfer {
      from: self.vault_account.to_account_info().clone(),
      to: self.taker_receive_token_account.to_account_info().clone(),
      authority: self.vault_authority.clone(),
    };
    CpiContext::new(self.token_program.clone(), cpi_accounts)
  }

  pub fn into_close_context(&self) -> CpiContext<'_, '_, '_, 'info, CloseAccount<'info>> {
    let cpi_accounts = CloseAccount {
      account: self.vault_account.to_account_info().clone(),
      destination: self.initializer.clone(),
      authority: self.vault_authority.clone(),
    };
    CpiContext::new(self.token_program.clone(), cpi_accounts)
  }
}
