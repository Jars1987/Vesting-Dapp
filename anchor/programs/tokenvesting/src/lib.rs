

use anchor_lang::prelude::*;
use anchor_spl::associated_token::AssociatedToken;
use anchor_spl::token_interface::{ self, Mint, TokenAccount, TokenInterface, TransferChecked };

declare_id!("3NZmUegbSaiwcmWnb6F2zGNyMs3eHF7yCK6u9mBaFvvD");

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
        total_withdrawn: 0,
        bump: context.bumps.employee_account,
      };

      Ok(())
    }

    pub fn claim_tokens(context: Context<ClaimTokens>, _company_name: String) -> Result<()> {

      let employee_account = &mut context.accounts.employee_account;
      let now = Clock::get()?.unix_timestamp; //check the time now

      // If we have not reached the cliff time yet, Claim Not Available yet
      if now < employee_account.cliff_time {
          return Err(ErrorCode::ClaimNotAvailableYet.into());
      }
      // Calculate the amount of tokens vested
      let time_since_start = now.saturating_sub(employee_account.start_time); //saturating sub prevent it go under zero, therefore will prevent underflows. This is i64 because its unix time
      let total_vesting_time = employee_account.end_time.saturating_sub(
          employee_account.start_time
      );
       //this if statement that was here is a bit redondant 
        if total_vesting_time == 0 {
          return Err(ErrorCode::InvalidVestingPeriod.into())
        }
       

      let vested_amount = if now >= employee_account.end_time {   // vesting period has ended, so all tokens are fully vested
          employee_account.total_amount
      } else {
          // Lets mach the chcked_mul cause otherwise it can return a none value  
          match employee_account.total_amount.checked_mul(time_since_start as u64){   //proportion of the total tokens that would have vested based on the elapsed time since the start 
          Some (vested) => vested/(total_vesting_time as u64),  //scales the amount so that it correctly represents a fraction of the total amount over the entire vesting duration.
          None => {
            return Err(ErrorCode::CalculationOverflow.into())       //we need to use the keyword return making rust existing the function so Err will never be part of vested_amount
          }
          }
      };

      //Calculate the amount that can be withdrawn
      let claimable_amount = vested_amount.saturating_sub(employee_account.total_withdrawn);
      // Check if there is anything left to claim
      if claimable_amount == 0 {
          return Err(ErrorCode::NothingToClaim.into());
      }
      let transfer_cpi_accounts = TransferChecked {
          from: context.accounts.treasury_token_account.to_account_info(),
          mint: context.accounts.mint.to_account_info(),
          to: context.accounts.employee_token_account.to_account_info(),
          authority: context.accounts.treasury_token_account.to_account_info(),
      };
      let cpi_program = context.accounts.token_program.to_account_info();
      let signer_seeds: &[&[&[u8]]] = &[
          &[
              b"vesting_treasury",
              context.accounts.vesting_account.company_name.as_ref(),
              &[context.accounts.vesting_account.treasury_bump],
          ],
      ];
      let cpi_context = CpiContext::new(cpi_program, transfer_cpi_accounts).with_signer(
          signer_seeds
      );
      let decimals = context.accounts.mint.decimals;
      token_interface::transfer_checked(cpi_context, claimable_amount as u64, decimals)?;
      employee_account.total_withdrawn += claimable_amount;

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
  

#[derive(Accounts)]
#[instruction(company_name: String)]
pub struct ClaimTokens<'info> {
    #[account(mut)]
    pub beneficiary: Signer<'info>,

    #[account(
        mut,
        seeds = [b"employee_vesting", beneficiary.key().as_ref(), vesting_account.key().as_ref()],
        bump = employee_account.bump,
        has_one = beneficiary,        //only the right employee has the access to it
        has_one = vesting_account     //making sure is connecetd to the right vesting account
    )]
    pub employee_account: Account<'info, EmployeeAccount>,

    #[account(
        mut,
        seeds = [company_name.as_ref()],
        bump = vesting_account.bump,
        has_one = treasury_token_account,
        has_one = mint
    )]
    pub vesting_account: Account<'info, VestingAccount>,

    pub mint: InterfaceAccount<'info, Mint>,  

    #[account(mut)]
    pub treasury_token_account: InterfaceAccount<'info, TokenAccount>,

    #[account(
        init_if_needed,
        payer = beneficiary,
        associated_token::mint = mint,
        associated_token::authority = beneficiary,
        associated_token::token_program = token_program
    )]
    pub employee_token_account: InterfaceAccount<'info, TokenAccount>,

    pub token_program: Interface<'info, TokenInterface>,
    pub associated_token_program: Program<'info, AssociatedToken>,
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
  pub total_withdrawn: u64, 
  pub bump: u8,
}


#[error_code]
pub enum ErrorCode {
    #[msg("Claiming is not available yet.")]
    ClaimNotAvailableYet,
    #[msg("There is nothing to claim.")]
    NothingToClaim,
    #[msg("Invalid Vesting Period")]
    InvalidVestingPeriod,
    #[msg("")]
    CalculationOverflow,
     
}