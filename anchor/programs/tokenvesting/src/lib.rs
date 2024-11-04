#![allow(clippy::result_large_err)]

use anchor_lang::prelude::*;
use anchor_spl::associated_token::AssociatedToken;
use anchor_spl::token_interface::{ self, Mint, TokenAccount, TokenInterface, TransferChecked };

declare_id!("AsjZ3kWAUSQRNt2pZVeJkywhZ6gpLpHZmJjduPmKZDZZ");

#[program]
pub mod tokenvesting {
    use super::*;


    pub fn create_vesting_account (context: Context<CreateVestingAccount>, company_name: String ) -> Result<()> {
      //this instruction is to save all the data we need to the VestingAccount 

    //we dereference the vesting account and allows you to modify the account and this is needed because vesting_Account is an account reference. So * tells rust you want to work withe the actual data and not just the referece
    *context.accounts.vesting_account = VestingAccount { 
      owner: context.accounts.signer.key(),
      mint: context.accounts.mint.key(),
      treasury_token_account: context.accounts.treasury_token_account.key(),
      company_name,
      treasury_bump: context.bumps.treasury_token_account,
      bump: context.bumps.vesting_account,
      };
      Ok(())
    }

    pub fn create_employee_account (context: Context<CreateEmployeeAccount>, start_time: i64, end_time: i64, total_amount: u64, cliff_time: i64) -> Result<()> {
      *context.accounts.employee_account = EmployeeAccount {
        beneficiary: context.accounts.beneficiary.key(),
        start_time: start_time,
        end_time: end_time,
        cliff_time: cliff_time,
        vesting_account: context.accounts.vesting_account.key(),
        total_amount: total_amount,
        total_withdraw: 0,
        bump: context.bumps.employee_account,
      };

      Ok(())
    }


}




#[derive(Accounts)]
#[instruction(company_name: String)]
pub struct CreateVestingAccount<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,

    #[account(
        init,
        space = 8 + VestingAccount::INIT_SPACE,
        payer = signer,
        seeds = [company_name.as_ref()],
        bump
    )]
    pub vesting_account: Account<'info, VestingAccount>,

    pub mint: InterfaceAccount<'info, Mint>,

    #[account(
        init,
        token::mint = mint,
        token::authority = treasury_token_account, //the authority is the token account its self so it has the ability to transfer tokens to the employees
        payer = signer,
        seeds = [b"vesting_treasury", company_name.as_bytes()], //its a token account and not an associated token account because its going to be a token account specified for this vesting contract
        bump
    )]
    pub treasury_token_account: InterfaceAccount<'info, TokenAccount>,

    pub token_program: Interface<'info, TokenInterface>,
    pub system_program: Program<'info, System>,
}

    
#[derive(Accounts)]
pub struct CreateEmployeeAccount<'info> {
  #[account(mut)]
  pub owner: Signer<'info>, //the signer will be the employer and not the employee, its the owner that is creating the contract

  pub beneficiary: SystemAccount<'info>, //employee

  #[account(
    has_one = owner   //we set this constraint to make sure the owner of the vesting account is the signer of this instruction (the right person has the access)
  )]
  pub vesting_account: Account<'info, VestingAccount>,

  #[account(   //just holding the state so we don't need to use the token constraint
    init,
    space = 8 + EmployeeAccount::INIT_SPACE,
    payer = owner,
    seeds = [b"employee_vesting", beneficiary.key().as_ref(), vesting_account.key().as_ref()],
    bump
  )]
  pub employee_account: Account<'info, EmployeeAccount>,


  pub token_program: Interface<'info, TokenInterface>,
  pub system_program: Program<'info, System>,

}
  




#[account]
#[derive(InitSpace)]

pub struct VestingAccount {
  pub owner: Pubkey,  //employee
  pub mint: Pubkey,   //token vesting
  pub treasury_token_account: Pubkey, // where we will be storing our tokens
  #[max_len(50)]
  pub company_name: String,
  pub treasury_bump: u8,
  pub bump: u8,           //bump for the vesting account
}


#[account]
#[derive(InitSpace)]
pub struct EmployeeAccount {
  pub beneficiary: Pubkey, //employee pubkey for this specific account
  pub start_time: i64, 
  pub end_time: i64,
  pub cliff_time: i64, //how long the employee needs to wait till their tokens unlock
  pub vesting_account: Pubkey,
  pub total_amount: u64, //total amount of tokens allocated to the employee
  pub total_withdraw: u64, 
  pub bump: u8,
}
