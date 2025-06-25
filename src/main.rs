use solana_program::{
    account_info::{next_account_info, AccountInfo}, entrypoint::{self, ProgramResult}, msg, program::invoke_signed, program_error::ProgramError, pubkey::Pubkey, system_instruction
};

entrypoint!(process_instruction);

pub fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8]
) -> ProgramResult {
    let instruction = instruction_data[0];

    match instruction {
        0 => initialize_vault(program_id, accounts),
        1 => withdraw_from_vault(program_id, accounts),
        _ => Err(ProgramError::InvalidInstructionData),
    }
}

fn initialize_vault(program_id: &Pubkey, accounts: &[AccountInfo]) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();

    let initializer = next_account_info(accounts_iter)?;
    let vault_account = next_account_info(accounts_iter)?;
    let system_program = next_account_info(accounts_iter)?;

    let (vault_pda, bump) = Pubkey::find_program_address(&[b"vault"], program_id);

    if vault_pda != *vault_account.key {
        msg!("Invalid vault pda");
        return Err(ProgramError::InvalidAccountData);
    }

    let amount = 1_000_000; // 0.001 SOL

    // Transfer some SOL from the user to the vault PDA account, and use a special signature because the vault is a PDA.
    /*
        Because the invoke_signed() function needs owned copies (not just references) of the AccountInfo structs.
        In Rust, if you pass something into a function by value, it takes ownership. But in our case:
        initializer, vault_account, and system_program are borrowed from the accounts array (next_account_info(accounts_iter)?).
        But invoke_signed wants to take ownership of those accounts to do its work safely (it may mutate them).
        So we clone them: .clone() creates a new reference-counted pointer to the same account
     */
    invoke_signed(&system_instruction::transfer(initializer.key, vault_account.key, amount),
        &[initializer.clone(), vault_account.clone(), system_program.clone()],
        &[&[b"vault", &[bump]]]
    )?;

    msg!("Deposited {} amount", amount);
    Ok(())
}

fn withdraw_from_vault(program_id: &Pubkey, accounts: &[AccountInfo]) -> ProgramResult {
    // Creates a mutable iterator to go through the list of accounts one by one.
    let accounts_iter = &mut accounts.iter();

    
}