use anchor_lang::prelude::*;
use anchor_lang::solana_program::entrypoint::ProgramResult;
use anchor_lang::{prelude::*, solana_program::clock::Clock};
use anchor_spl::token::{self, Mint, Token, TokenAccount};

// declare_id!("6UBYXoEKvjvskoXazBUyBgSJ8maTUpd6jkuaCdr7Q1Zs");
declare_id!("6UBYXoEKvjvskoXazBUyBgSJ8maTUpd6jkuaCdr7Q1Zs");

#[program]
pub mod ico {
    pub const ICO_MINT_ADDRESS: &str = "49Jx7fP8rFRac8KSeTRYhAzyugFsiuhFTi1gbzY9Dqoh";
    // pub const ICO_MINT_ADDRESS: &str = "FBKhAghAqzttng8UAAf7VuX7msiNAtVxgEsY4PrfZxP4";
    use super::*;

    /* 
    ===========================================================
        create_ico_ata function use CreateIcoATA struct
    ===========================================================
*/
    pub fn create_ico_ata(
        ctx: Context<CreateIcoATA>,
        phase_one_tokens: u64,
        phase_one_price: u64,
        phase_one_time: u64,
        phase_two_tokens: u64,
        phase_two_price: u64,
        phase_two_time: u64,
        phase_three_tokens: u64,
        phase_three_price: u64,
        phase_three_time: u64,
    ) -> Result<()> {
        msg!("create program ATA for hold ICO");
        // // transfer ICO admin to program ata
        let cpi_ctx = CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            token::Transfer {
                from: ctx.accounts.ico_ata_for_admin.to_account_info(),
                to: ctx.accounts.ico_ata_for_ico_program.to_account_info(),
                authority: ctx.accounts.admin.to_account_info(),
            },
        );
        let ico_amount = phase_one_tokens + phase_two_tokens + phase_three_tokens;
        token::transfer(cpi_ctx, ico_amount)?;
        msg!("send {} ICO to program ATA.", ico_amount);

        // save data in data PDA
        let clock = Clock::get()?;
        let data = &mut ctx.accounts.data;
        data.phaseOnePrice = phase_one_price;
        data.phaseOneTokens = phase_one_tokens;
        data.phaseOneTime = (clock.unix_timestamp + phase_one_time as i64) as u64;
        data.phaseTwoPrice = phase_two_price;
        data.phaseTwoTokens = phase_two_tokens;
        data.phaseTwoTime = (clock.unix_timestamp + phase_two_time as i64) as u64;
        data.phaseThreePrice = phase_three_price;
        data.phaseThreeTokens = phase_three_tokens;
        data.phaseThreeTime = (clock.unix_timestamp + phase_three_time as i64) as u64;
        data.admin = *ctx.accounts.admin.key;
        msg!("save data in program PDA.");
        Ok(())
    }

    /* 
    ===========================================================
        deposit_ico_in_ata function use DepositIcoInATA struct
    ===========================================================
*/
    pub fn deposit_ico_in_ata(ctx: Context<DepositIcoInATA>, ico_amount: u64) -> ProgramResult {
        if ctx.accounts.data.admin != *ctx.accounts.admin.key {
            return Err(ProgramError::IncorrectProgramId);
        }
        // transfer ICO admin to program ata
        let cpi_ctx = CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            token::Transfer {
                from: ctx.accounts.ico_ata_for_admin.to_account_info(),
                to: ctx.accounts.ico_ata_for_ico_program.to_account_info(),
                authority: ctx.accounts.admin.to_account_info(),
            },
        );
        token::transfer(cpi_ctx, ico_amount)?;
        msg!("deposit {} ICO in program ATA.", ico_amount);
        Ok(())
    }

    /* 
    ===========================================================
        buy_with_sol function use BuyWithSol struct
    ===========================================================
*/
    pub fn buy_with_sol(
        ctx: Context<BuyWithSol>,
        _ico_ata_for_ico_program_bump: u8,
        sol_amount: u64,
        phase: u8,
    ) -> Result<()> {
        let data = &mut ctx.accounts.data;
        let current_time = Clock::get()?;
        if phase == 1 {
            (current_time.unix_timestamp as u64) < data.phaseOneTime
        } else if phase == 2 {
            (current_time.unix_timestamp as u64) < data.phaseTwoTime
        } else if phase == 3 {
            (current_time.unix_timestamp as u64) < data.phaseThreeTime
        } else {
            return Err(ProgramError::InvalidArgument.into());
        };
        // transfer sol from user to admin
        let ix = anchor_lang::solana_program::system_instruction::transfer(
            &ctx.accounts.user.key(),
            &ctx.accounts.admin.key(),
            sol_amount,
        );
        anchor_lang::solana_program::program::invoke(
            &ix,
            &[
                ctx.accounts.user.to_account_info(),
                ctx.accounts.admin.to_account_info(),
            ],
        )?;
        msg!("transfer {} sol to admin.", sol_amount);

        // transfer ICO from program to user ATA
        let ico_amount;
        if phase == 1 {
            ico_amount = (sol_amount * data.phaseOnePrice) / 1_000_000_000;
        } else if phase == 2 {
            ico_amount = (sol_amount * data.phaseTwoPrice) / 1_000_000_000;
        } else {
            ico_amount = (sol_amount * data.phaseThreePrice) / 1_000_000_000;
        };
        let ico_mint_address = ctx.accounts.ico_mint.key();
        let seeds = &["ico5".as_bytes(), &[_ico_ata_for_ico_program_bump]];
        let signer = [&seeds[..]];
        let cpi_ctx = CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            token::Transfer {
                from: ctx.accounts.ico_ata_for_ico_program.to_account_info(),
                to: ctx.accounts.ico_ata_for_user.to_account_info(),
                authority: ctx.accounts.ico_ata_for_ico_program.to_account_info(),
            },
            &signer,
        );
        token::transfer(cpi_ctx, ico_amount)?;
        if phase == 1 {
            data.phaseOneTokens -= ico_amount;
            data.phaseOneSoldTokens += ico_amount;
            data.phaseOneSol += sol_amount;
        } else if phase == 2 {
            data.phaseTwoTokens -= ico_amount;
            data.phaseTwoSoldTokens += ico_amount;
            data.phaseTwoSol += sol_amount;
        } else {
            data.phaseThreeTokens -= ico_amount;
            data.phaseThreeSoldTokens += ico_amount;
            data.phaseThreeSol += sol_amount;
        }
        msg!("transfer {} ico to buyer/user.", ico_amount);
        Ok(())
    }

    /* 
    ===========================================================
        update_data function use UpdateData struct
    ===========================================================
*/
    pub fn update_data(ctx: Context<UpdateData>, phase: u8, new_price: u64) -> ProgramResult {
        if ctx.accounts.data.admin != *ctx.accounts.admin.key {
            return Err(ProgramError::IncorrectProgramId);
        }
        let data = &mut ctx.accounts.data;
        if phase == 1 {
            data.phaseOnePrice = new_price;
        } else if phase == 2 {
            data.phaseTwoPrice = new_price;
        } else {
            return Err(ProgramError::InvalidArgument.into());
        };
        msg!("update SOL/ICO {} ", new_price);
        Ok(())
    }

    /* 
    -----------------------------------------------------------
        CreateIcoATA struct for create_ico_ata function
    -----------------------------------------------------------
*/
    #[derive(Accounts)]
    pub struct CreateIcoATA<'info> {
        // 1. PDA (pubkey) for ico ATA for our program.
        // seeds: [ico_mint + current program id] => "HashMap[seeds+bump] = pda"
        // token::mint: Token Program wants to know what kind of token this ATA is for
        // token::authority: It's a PDA so the authority is itself!
        #[account(
        init_if_needed,
        payer = admin,
        seeds = [b"ico5"],
        bump,
        token::mint = ico_mint,
        token::authority = ico_ata_for_ico_program,
    )]
        pub ico_ata_for_ico_program: Account<'info, TokenAccount>,

        #[account(init_if_needed, payer=admin, space=600, seeds=[b"data5"], bump)]
        pub data: Account<'info, Data>,

        #[account(
        address = ICO_MINT_ADDRESS.parse::<Pubkey>().unwrap(),
    )]
        pub ico_mint: Account<'info, Mint>,

        #[account(mut)]
        pub ico_ata_for_admin: Account<'info, TokenAccount>,

        #[account(mut)]
        pub admin: Signer<'info>,

        pub system_program: Program<'info, System>,
        pub token_program: Program<'info, Token>,
        pub rent: Sysvar<'info, Rent>,
    }

    /* 
    -----------------------------------------------------------
        DepositIcoInATA struct for deposit_ico_in_ata function
    -----------------------------------------------------------
*/
    #[derive(Accounts)]
    pub struct DepositIcoInATA<'info> {
        #[account(
        mut,
        seeds = [b"ico5"],
        bump,
    )]
        pub ico_ata_for_ico_program: Account<'info, TokenAccount>,

        #[account(
        mut,
        seeds = [b"data5"],
        bump,
    )]
        pub data: Account<'info, Data>,

        #[account(
        address = ICO_MINT_ADDRESS.parse::<Pubkey>().unwrap(),
    )]
        pub ico_mint: Account<'info, Mint>,

        #[account(mut)]
        pub ico_ata_for_admin: Account<'info, TokenAccount>,

        #[account(mut)]
        pub admin: Signer<'info>,
        pub token_program: Program<'info, Token>,
    }

    /* 
    -----------------------------------------------------------
        BuyWithSol struct for buy_with_sol function
    -----------------------------------------------------------
*/
    #[derive(Accounts)]
    #[instruction(_ico_ata_for_ico_program_bump: u8)]
    pub struct BuyWithSol<'info> {
        #[account(
        mut,
        seeds = [b"ico5"],
        bump,
    )]
        pub ico_ata_for_ico_program: Account<'info, TokenAccount>,

        #[account(
        mut,
        seeds = [b"data5"],
        bump,
    )]
        pub data: Account<'info, Data>,

        #[account(
        address = ICO_MINT_ADDRESS.parse::<Pubkey>().unwrap(),
    )]
        pub ico_mint: Account<'info, Mint>,

        #[account(mut)]
        pub ico_ata_for_user: Account<'info, TokenAccount>,

        #[account(mut)]
        pub user: Signer<'info>,

        /// CHECK:
        #[account(mut)]
        pub admin: AccountInfo<'info>,

        pub token_program: Program<'info, Token>,
        pub system_program: Program<'info, System>,
    }
    // -----------------------------------------------------------
    //     UpdateData struct for update_data function
    // -----------------------------------------------------------
    #[derive(Accounts)]
    pub struct UpdateData<'info> {
        #[account(mut)]
        pub data: Account<'info, Data>,
        #[account(mut)]
        pub admin: Signer<'info>,
        pub system_program: Program<'info, System>,
    }

    /* 
    -----------------------------------------------------------
        Data struct for PDA Account
    -----------------------------------------------------------
*/
    #[account]
    pub struct Data {
        pub phaseOneTime: u64,
        pub phaseOnePrice: u64,
        pub phaseOneTokens: u64,
        pub phaseOneSoldTokens: u64,
        pub phaseOneSol: u64,
        pub phaseTwoTime: u64,
        pub phaseTwoPrice: u64,
        pub phaseTwoTokens: u64,
        pub phaseTwoSoldTokens: u64,
        pub phaseTwoSol: u64,
        pub phaseThreeTime: u64,
        pub phaseThreePrice: u64,
        pub phaseThreeTokens: u64,
        pub phaseThreeSoldTokens: u64,
        pub phaseThreeSol: u64,
        pub admin: Pubkey,
    }
}
