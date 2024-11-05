# token-vesting

## Proposal

This dapp proposes to build a vesting contract that will be done between 2
parties, the employer and the employee.

## Method

The first step is to initialize the Vesting contract. Then we need to give the
employer the ability to add employees and lastly allow employees to to claim the
tokens once they are vested.

### Walkthrough

The first thing that was built in the program were the AccountVesting Accounts
and the create_vesting_account instructions.

Second, the employee also need a vesting account to hold the information
regarding their vesting tokens, like clif time and how many tokens alocated. So
we build a state account, the accounts struct and the the instructions.

Then we proceed to buil the create_employee_account instructions, respective
context and state account.

The program final step is to create the ClaimToken accounts and instructions.
This is where most of the logic will be held.
