use anchor_lang::prelude::*;

declare_id!("GMTKAcYEXAqnokiLhmc8S1kiRgmCSFptYkUjZwEzwUmF");

#[program]
pub mod nft_staking {
    use std::thread::current;

    use anchor_lang::{prelude::sysvar::rewards, system_program::Transfer};

    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        let pool = &mut ctx.accounts.stake_pool;
        pool.authority = *ctx.accounts.authority.key;
        pool.reward_mint = ctx.accounts.reward_mint.key();
        pool.reward_vault = ctx.accounts.reward_vault.key();
        pool.reward_rate = reward_rate;

        msg!("Greetings from: {:?}", ctx.program_id);
        Ok(())
    }

    pub fn stake(ctx:Context<StakeNft>)-> Result<()>{
        let clcok = Clock::get()?;
        let entry = &mut ctx.accounts.stake_entry;
        entry.nft_mint = ctx.accounts.nft_mint.key();
        entry.stake_time = clock.unix_timestamp();
        entry.last_claim_time = clock.unix_timestamp();
        entry.is_staked = true;

        let cpi_accounts = Transfer{
            from:ctx.accounts.user_nft_token_account.to_account_info(),
            to:ctx.accounts.nft_vault.to_account_info(),
        };
        let cpi_program = ctx.accounts.token_program.to_account_info();
        let cpi_ctx = CpiContext::new(
            cpi_program,cpi_accounts
        );
        token::transfer(cpi_ctx, 1)?; 

        Ok(())
    }


    pub fn unstake(ctx:CpiContext<Unstake>)->Result<()>{
        let stake_entry = &ctx.accounts.stake_entry;
        let clock = Clock::get()?;
        let time_elapsed = clock.unix_timestamp.checked_sub(stake_entry.last_claim_time).unwrap();
        const MAX_UNCLAMED_TIME: i64 = 5;
        if time_elapsed > MAX_UNCLAMED_TIME{
            let rewards_pending = (ctx.accounts.stake_entry.reward_rate as u128)
                .checked_mul(time_elapsed as u128)
                .unwrap();
            if rewards_pending > 0{
                return Err(ErrorCode::UnclaimedRewards.into());
            }
        }
        let vault_seeds = &[
            b"vault",
            &stake_entry.nft_mint.to_bytes(),
            &stake_entry.staker.to_bytes(),
            &[ctx.bumps.nft_vault]
        ];
        let signer_seeds = &[&vault_seeds[..]];

        let cpi_accounts = Transfer {
            from: ctx.accounts.nft_vault.to_account_info(),
            to: ctx.accounts.user_nft_account.to_account_info(),
        };
        let vault_seeds = &[
            b"vault",
            &stake_entry.nft_mint.to_bytes(),
            &stake_entry.staker.to_bytes(),
            &[ctx.bumps.nft_vault]
        ];
        let signer_seeds = &[&vault_seeds[..]];

        let cpi_accounts = Transfer {
            from: ctx.accounts.nft_vault.to_account_info(),
            to: ctx.accounts.user_nft_account.to_account_info(),
           
        };
        let  authority=  ctx.accounts.nft_vault.to_account_info();
        let cpi_program = ctx.accounts.token_program.to_account_info();
        let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer_seeds);

        token::transfer(cpi_ctx, 1)?; 
        msg!("NFT unstaked. All accounts closed.");

        Ok(());
        let cpi_program = ctx.accounts.token_program.to_account_info();
        let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer_seeds);

        token::transfer(cpi_ctx, 1)?;
        
        msg!("NFT unstaked. All accounts closed.");

        Ok(())

    }

    pub fn claim_reward(ctx:Context<ClaimReward>)-> Result<()>{
        let clock = Clock::get()?;
        let current_time = clock.unix_timestamp;
        let stake_entry = &mut ctx.accounts.stake_entry;
        let stake_pool = &ctx.accounts.stake_pool;

        let time_elapsed = current_time.checked_sub(stake_entry.last_claim_time).unwrap();
        let rewards_to_pay = (stake_pool.reward_rate as u128)
            .checked_mul(time_elapsed as u128)
            .unwrap();

        let rewards_as_u64 = rewards_to_pay as u64;

        if rewards_as_u64 == 0 {
            msg!("No rewards to claim.");
            return Ok(());
        }


        let pool_seeds = &[
            b"stake_pool",
            &[ctx.bumps.stake_pool]
        ];
        let signer_seeds = &[&pool_seeds[..]];
        
        let cpi_accounts = Transfer {
            from: ctx.accounts.reward_vault.to_account_info(),
            to: ctx.accounts.user_reward_account.to_account_info(),
        };
        let cpi_program = ctx.accounts.token_program.to_account_info();
        let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer_seeds);

        token::transfer(cpi_ctx, rewards_as_u64)?;
        stake_entry.last_claim_time = current_time;

        msg!("Rewards claimed. New last claim time: {}", current_time);

        Ok(())
    }



}





#[derive(Accounts)]
pub struct Initialize<'info>{

    // what is this pda is for 
    #[account(
        init, 
        payer = admin,
        space = 8 + 32 + 32 + 8 + 1,// this is the staking account of the user
        seeds = [b"config"],
        bump
    )]
    pub stake_pool: Account<'info,StakePool>,
    pub authority:Signer<'info >,
    pub reward_mint:Account<'info, Mint>,

    #[account(
        init,
        payer = authority,
        token::mint = reward_mint,
        token::authority =authority,// this is the reward token of the user
        seeds = [b"reward_mint"],
        bump
    )]
    pub reward_vault: Account<'info , TokenaAccount>,

    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
pub struct StakeNft<'info> {
    #[account(mut)]
    pub user:Signer<'info>,

    #[account(
        mut ,
        constraint = user_nft_account.mint == nft_mint.key(),
        constraint = user_nft_account.owner = user.key(),
        ccontraint = user_nft_account.account.amount == 1
        
    )]
    pub user_nft_account :Account<'info , TokenAccount>,
    pub nft_mint : Account<'info , Mint>,

    #[account(
        init, 
        payer = user,
        space = 8 + 32 + 32 + 8 + 8 + 1,
        seeds = [b"stake_entry",nft_mint.key().as_ref(),user.key().as_ref()],
        bump
    )]
    pub stake_entry:Account<'info , StakeEntry>,

    #[account(
        init,
        payer = user,
        token::mint = nft_mint,
        token::authority = nft_vault, 
        seeds = [b"vault", nft_mint.key().as_ref(), user.key().as_ref()],
        bump
    )]
    pub nft_vault: Account<'info, TokenAccount>,
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub rent: Sysvar<'info, Rent>,
}

#[derive(account)]
pub struct Unstake<'info>{
    #[accounnt(mut)]
    pub user:Signer<'info>,

    #[account(
        mut,
        token::mint = nft_mint.key()
    )]
    pub user_nft_account : Account<'info , TokenAccount>,
    pub nft_mint: Account<'info , Mint>,
    #[account(
        mut,
        close = user,
        seeds = b["stake_entry", nft_mint.key().as_ref, user.key().as_ref()],
        bump,
        constraint = stake_entry.staker == user.key()
    )]
    pub stake_entry :Account<'info, StakeEntry>,
    #[account(
        mut,
        close = user,
        seeds = [b"stake_entry",nft_mint.key().as_ref(), user.key().as_ref()]
    )]
    pub nft_vault : Account<'info , TokenAccount>,
    pub token_program : Program<'info, Token>,

}

#[derive(account)]
pub struct ClaimReward<'info>{
    #[acccount]
    pub user :Signer<'info>,
    #[account(
        mut,
        constraint = stale_entry.owner = user.key(),
        seeds = [b"stake_entry", nft_mint.key().as_ref(), user.key().as_ref()],
        bump
    )]
    pub stake_entry :Account<'info, StakeEntry>,

    #[account(
        seeds = [b"stake_pool"],
        bump,
    )]
    pub stake_pool:Account<'info, StakePool>,
    
    #[account(
        mut, 
        seeds = [b"reward_vault"],
        bump,
        constraint = rewars_vault.key() == stake_pool.reward_vault
    )]
    pub reward_vault : Account<'info, TokenAccount>,
    #[account(
        mut,
        constraint = user_reward_account.owner == user.key(),
        constraint = user_reward_account.mint == stake_pool.reward_mint
    )]
    pub user_reward_account: Account<'info, TokenAccount>,

    pub token_program: Program<'info, Token>,


}


#[account]
#[derive(Default)]
pub struct StakePool {
    pub authority: Pubkey,
    pub reward_mint: Pubkey,
    pub reward_vault: Pubkey,
    pub reward_rate: u64,
    pub bump:u8
}

#[account]
#[derive(Default)]
pub struct StakeEntry {
    pub staker: Pubkey,
    pub nft_mint: Pubkey,
    pub stake_time: i64,
    pub last_claim_time: i64,
    pub is_staked: bool,
}
#[error_code]
pub enum ErrorCode {
    #[msg("Arithmetic overflow.")]
    Overflow,
}