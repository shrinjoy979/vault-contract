use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint,
    entrypoint::ProgramResult,
    msg,
    program::{invoke_signed},
    program_error::ProgramError,
    pubkey::Pubkey,
    system_instruction
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

    let owner = next_account_info(accounts_iter)?;
    let vault_account = next_account_info(accounts_iter)?;
    let recipient = next_account_info(accounts_iter)?; // who should received SOL from the wallet

    // Calculates the expected PDA address using the seed "vault" and the program’s public key.
    let (vault_pda, bump) = Pubkey::find_program_address(&[b"vault"], program_id);

    if vault_pda != *vault_account.key {
        msg!("Invalid vault PDA");
        return Err(ProgramError::InvalidAccountData);
    }

    if !owner.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    let lamports = vault_account.lamports();

    /*
        safely borrow mutable reference of lamports of vault accounts
        subtract all the lamports from the vault, set it to 0
        Why **?
        try_borrow_mut_lamports will return <&mut u64, ProgramError>
        The first * dereferences the mutable reference: From &mut u64 → u64
        The second * is used because you want to modify the actual value: You’re saying: “take the number inside this reference and subtract from it.”
        
        Can be understood as:Borrow the lamports value mutably, then directly subtract from the actual u64 value it points to.
    */

    /* 
        let lamports_to_transfer = vault_account.lamports();

        let mut vault_lamports = vault_account.try_borrow_mut_lamports()?;
        let mut recipient_lamports = recipient.try_borrow_mut_lamports()?;

        // Subtract from the vault
        *vault_lamports -= lamports_to_transfer;

        // Add to the recipient
        *recipient_lamports += lamports_to_transfer;
    */
     
    **vault_account.try_borrow_mut_lamports()? -= lamports;
    **recipient.try_borrow_mut_lamports()? += lamports;

    msg!("Withdrawn {} lamports to recipient address", lamports);

    Ok(())
}