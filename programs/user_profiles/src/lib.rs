pub mod state;

use anchor_lang::prelude::*;

pub use state::*;

declare_id!("C7LUvHjb9ZNnrm6W6zWhagmLyTC8iySSUnkFvbT26y2j");

#[account]
pub struct UserProfile{
    pub address: Pubkey,
    pub name: String,
    pub bio: String,
}
#[derive(Accounts)]
pub struct InitializeProfile<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,

    #[account(
        init,
        payer=authority,
        space=8+32+(4+30)+(4+100),
        seeds = [b"profile", authority.key().as_ref()],
        bump,

    )]

    pub user_profile: Account<'info, UserProfile>,
    pub system_program: Program<'info, System>, 
}

#[derive(Accounts)]
pub struct CloseProfile<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    #[account(
        mut,
        close = authority,
        seeds = [b"profile", authority.key().as_ref()],
        bump,
        constraint = user_profile.address == authority.key()
    )]
    pub user_profile: Account<'info, UserProfile>,
}


#[derive(Accounts)]
pub struct UpdateProfile<'info> {
    pub authority: Signer<'info>,
    #[account(
        mut,
        seeds = [b"profile", authority.key().as_ref()],
        bump,
        constraint = user_profile.address == authority.key()
    )]
    pub user_profile: Account<'info, UserProfile>,
}

#[program]
pub mod user_profiles {
    use super::*;

    pub fn initialize_profile(
        ctx: Context<InitializeProfile>,
        name: String,
        bio: String
    ) -> Result<()> {

        require!(name.len() <= 30, ProfileError::NameTooLong);
        require!(bio.len() <= 100, ProfileError::BioTooLong);

        let user_profile = &mut ctx.accounts.user_profile;
        
        user_profile.name=name;
        user_profile.bio=bio;
        user_profile.address= ctx.accounts.authority.key();

        Ok(())
    }

    pub fn update_profile(ctx: Context<UpdateProfile>, name: String, bio: String) -> Result<()>  {

        require!(name.len() <= 30, ProfileError::NameTooLong);
        require!(bio.len() <= 100, ProfileError::BioTooLong);

        let user_profile = &mut ctx.accounts.user_profile;
        user_profile.name=name;
        user_profile.bio=bio;

        Ok(())
    }

    pub fn close_profile(_ctx: Context<CloseProfile>) -> Result<()> {
        Ok(())
    }
}

#[error_code]
pub enum ProfileError {
    #[msg("Name cannot exceed 30 bytes")]
    NameTooLong,
    #[msg("Bio cannot exceed 100 bytes")]
    BioTooLong,
}