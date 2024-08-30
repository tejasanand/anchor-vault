use anchor_lang::prelude::*;
use anchor_spl::{
    token::{self, Mint, Token, TokenAccount},
    associated_token::AssociatedToken,
};

declare_id!("5zji4Yc5o2zAmC7zbhEtaMcCGLny4RAYwiaFeHKqFEbH");

#[program]
pub mod asset_vault {
    use super::*;

    pub fn initialize_vault(ctx: Context<InitializeVault>, vault_name: String) -> Result<()> {
        let vault = &mut ctx.accounts.vault;
        vault.admin = ctx.accounts.admin.key();
        vault.name = vault_name;
        vault.total_balance = 0;
        Ok(())
    }

    pub fn add_funds(ctx: Context<AddFunds>, amount: u64) -> Result<()> {
        let vault = &mut ctx.accounts.vault;
        let from = &ctx.accounts.user_token_account;
        let to = &ctx.accounts.vault_token_account;

        token::transfer(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                token::Transfer {
                    from: from.to_account_info(),
                    to: to.to_account_info(),
                    authority: ctx.accounts.user.to_account_info(),
                },
            ),
            amount,
        )?;

        vault.total_balance = vault.total_balance.checked_add(amount).unwrap();
        Ok(())
    }

    pub fn remove_funds(ctx: Context<RemoveFunds>, amount: u64) -> Result<()> {
        let vault = &mut ctx.accounts.vault;
        require!(vault.total_balance >= amount, VaultError::InsufficientBalance);

        let from = &ctx.accounts.vault_token_account;
        let to = &ctx.accounts.user_token_account;

        token::transfer(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                token::Transfer {
                    from: from.to_account_info(),
                    to: to.to_account_info(),
                    authority: vault.to_account_info(),
                },
            ),
            amount,
        )?;

        vault.total_balance = vault.total_balance.checked_sub(amount).unwrap();
        Ok(())
    }
}

#[derive(Accounts)]
pub struct InitializeVault<'info> {
    #[account(init, payer = admin, space = 8 + 32 + 32 + 8)]
    pub vault: Account<'info, Vault>,
    #[account(mut)]
    pub admin: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct AddFunds<'info> {
    #[account(mut)]
    pub vault: Account<'info, Vault>,
    #[account(mut)]
    pub user: Signer<'info>,
    #[account(mut)]
    pub user_token_account: Account<'info, TokenAccount>,
    #[account(
        init_if_needed,
        payer = user,
        associated_token::mint = token_mint,
        associated_token::authority = vault,
    )]
    pub vault_token_account: Account<'info, TokenAccount>,
    pub token_mint: Account<'info, Mint>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct RemoveFunds<'info> {
    #[account(mut)]
    pub vault: Account<'info, Vault>,
    #[account(mut)]
    pub user: Signer<'info>,
    #[account(mut)]
    pub vault_token_account: Account<'info, TokenAccount>,
    #[account(
        init_if_needed,
        payer = user,
        associated_token::mint = token_mint,
        associated_token::authority = user,
    )]
    pub user_token_account: Account<'info, TokenAccount>,
    pub token_mint: Account<'info, Mint>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}

#[account]
pub struct Vault {
    pub admin: Pubkey,
    pub name: String,
    pub total_balance: u64,
}

#[error_code]
pub enum VaultError {
    #[msg("Not enough funds in the vault")]
    InsufficientBalance,
}