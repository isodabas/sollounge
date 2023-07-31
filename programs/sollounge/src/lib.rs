use anchor_lang::prelude::*;
use anchor_spl::associated_token::AssociatedToken;
use anchor_spl::token::{Mint, Token, TokenAccount, Transfer as SplTransfer};
use std::str::FromStr;
// bet useradress field is wrong prbably bump
// bet sides dtype Option<>
// teamname max length 20 chars
// This is your program's public key and it will update
// automatically when you build the project.
declare_id!("4karKXsd6ukYynzsuXdeojUK8MqHRfUhFkWstLzwDJ1i");

mod constants;
use crate::constants::*;
mod errors;
use crate::errors::*;

#[program]
mod sol_lounge {
    use super::*;

    pub fn init_master(ctx: Context<InitMaster>) -> Result<()> {
        ctx.accounts.master.last_id = 0;
        ctx.accounts.master.authority = ctx.accounts.signer.key();
        ctx.accounts.master.bump = *ctx.bumps.get("master").unwrap();
        Ok(())
    }

    pub fn create_game(
        ctx: Context<CreateGame>,
        team_a: String,
        team_b: String,
        start_time: u64,
    ) -> Result<()> {
        ctx.accounts.master.last_id += 1;
        ctx.accounts.game.id = ctx.accounts.master.last_id;
        ctx.accounts.game.team_a = team_a;
        ctx.accounts.game.team_b = team_b;
        ctx.accounts.game.start_time = start_time;
        ctx.accounts.game.total_bet_team_a = 0;
        ctx.accounts.game.total_bet_team_b = 0;
        ctx.accounts.game.unique_bets = 0;
        ctx.accounts.game.has_ended = false;
        ctx.accounts.game.bump = *ctx.bumps.get("game").unwrap();
        Ok(())
    }

    pub fn place_bet(ctx: Context<PlaceBet>, bet_amount: u128, chosen_side: String) -> Result<()> {
        ctx.accounts.bet_account.game_id = ctx.accounts.game.id;
        ctx.accounts.bet_account.user_address = ctx.accounts.signer.key();
        ctx.accounts.bet_account.bet_amount = bet_amount;
        ctx.accounts.bet_account.bump = *ctx.bumps.get("bet_account").unwrap();

        if chosen_side == "a" {
            ctx.accounts.game.total_bet_team_a += bet_amount;
        } else if chosen_side == "b" {
            ctx.accounts.game.total_bet_team_b += bet_amount;
        }
        ctx.accounts.bet_account.chosen_side = chosen_side;
        ctx.accounts.game.unique_bets += 1;
        ctx.accounts.bet_account.id = ctx.accounts.game.unique_bets;
        let transfer_instruction = SplTransfer {
            from: ctx.accounts.user_ata.to_account_info(),
            to: ctx.accounts.game_ata.to_account_info(),
            authority: ctx.accounts.signer.to_account_info(),
        };

        let cpi_program = ctx.accounts.token_program.to_account_info();
        // Create the Context for our Transfer request
        let cpi_ctx = CpiContext::new(cpi_program, transfer_instruction);

        // Execute anchor's helper function to transfer tokens
        anchor_spl::token::transfer(cpi_ctx, bet_amount as u64)?;

        Ok(())
    }

    pub fn change_bet(ctx: Context<ChangeBet>) -> Result<()> {
        let side = &ctx.accounts.bet_account.chosen_side;
        let amount = ctx.accounts.bet_account.bet_amount;
        if side == "a" {
            ctx.accounts.bet_account.chosen_side = "b".to_string();
            ctx.accounts.game.total_bet_team_a -= amount;
            ctx.accounts.game.total_bet_team_b += amount;
        } else if side == "b" {
            ctx.accounts.bet_account.chosen_side = "a".to_string();
            ctx.accounts.game.total_bet_team_b -= amount;
            ctx.accounts.game.total_bet_team_a += amount;
        }
        Ok(())
    }

    pub fn end_game(ctx: Context<EndGame>, winner: String) -> Result<()> {
        require!(!ctx.accounts.game.has_ended, EndError::AlreadyEnded);
        let mut loser_pot: f64 = 0.0;
        let mut winner_pot: f64 = 0.0;
        if winner == "a".to_string() {
            loser_pot = ctx.accounts.game.total_bet_team_b as f64;
            winner_pot = ctx.accounts.game.total_bet_team_a as f64;
        } else if winner == "b".to_string() {
            loser_pot = ctx.accounts.game.total_bet_team_a as f64;
            winner_pot = ctx.accounts.game.total_bet_team_b as f64;
        } else {
            //throw err
            require!(!ctx.accounts.game.has_ended, EndError::InvalidWinner);
        }
        let house_fees: f64 = loser_pot * 0.03;
        loser_pot -= house_fees;
        ctx.accounts.game.reward_unit = loser_pot / winner_pot;
        ctx.accounts.game.winning_side = winner;
        ctx.accounts.game.has_ended = true;
        Ok(())
    }

    pub fn claim_prize(ctx: Context<ClaimPrize>) -> Result<()> {
        require!(
            ctx.accounts.bet_account.game_id == ctx.accounts.game_account.id,
            ClaimError::GameDoesntMatch
        );
        require!(ctx.accounts.game_account.has_ended, ClaimError::GameIsOn);
        require!(
            ctx.accounts.game_account.winning_side == ctx.accounts.bet_account.chosen_side,
            ClaimError::WinningSideDoesntMatch
        );
        require!(
            !ctx.accounts.bet_account.has_claimed,
            ClaimError::AlreadyClaimed
        );
        require!(
            ctx.accounts.bet_account.user_address == ctx.accounts.signer.key(),
            ClaimError::WrongSigner
        );
        let seeds = &[
            GAME_SEED.as_bytes(),
            &ctx.accounts.game_account.id.to_le_bytes(),
            &[ctx.accounts.game_account.bump],
        ];
        let signer = &[&seeds[..]];
        let transfer_instruction = SplTransfer {
            from: ctx.accounts.game_ata.to_account_info(),
            to: ctx.accounts.user_ata.to_account_info(),
            authority: ctx.accounts.game_account.to_account_info(),
        };

        let cpi_program = ctx.accounts.token_program.to_account_info();
        // Create the Context for our Transfer request
        let cpi_ctx = CpiContext::new_with_signer(cpi_program, transfer_instruction, signer);

        anchor_spl::token::transfer(
            cpi_ctx,
            (ctx.accounts.bet_account.bet_amount as f64
                * (ctx.accounts.game_account.reward_unit + 1.) as f64) as u64,
        )?;
        ctx.accounts.bet_account.has_claimed = true;
        Ok(())
    }
}

#[derive(Accounts)]
pub struct InitMaster<'info> {
    #[account(init, payer = signer, space = 4 + 32 + 1 + 8, seeds = [MASTER_SEED.as_bytes()], bump)]
    pub master: Account<'info, Master>,
    #[account(mut, address = Pubkey::from_str(AUTHORITY).unwrap().key())]
    pub signer: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[account]
pub struct Master {
    pub last_id: u32,
    pub authority: Pubkey,
    pub bump: u8,
}

#[derive(Accounts)]
pub struct CreateGame<'info> {
    #[account(init, payer = signer, space = 4 + 24 + 24 + 16 + 16 + 8 + 4 + 4 + 1 + 1 + 8, seeds = [GAME_SEED.as_bytes(), &(master.last_id + 1).to_le_bytes()], bump)]
    pub game: Account<'info, Game>,
    #[account(mut)]
    pub master: Account<'info, Master>,
    #[account(mut, address = Pubkey::from_str(AUTHORITY).unwrap().key())]
    pub signer: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[account]
pub struct Game {
    pub id: u32,
    pub team_a: String,
    pub team_b: String,
    pub total_bet_team_a: u128,
    pub total_bet_team_b: u128,
    pub start_time: u64,
    pub winning_side: String,
    pub unique_bets: u32,
    pub has_ended: bool,
    pub reward_unit: f64,
    pub bump: u8,
}

#[derive(Accounts)]
pub struct PlaceBet<'info> {
    #[account(init, payer = signer, space = 4 + 4 + 32 + 16 + 8 + 1 + 8, seeds = [BET_SEED.as_bytes(), signer.key().as_ref(), &game.id.to_le_bytes()], bump, has_one=game)]
    pub bet_account: Account<'info, Bet>,
    #[account(mut)]
    pub game: Account<'info, Game>,
    #[account(mut)]
    pub master: Account<'info, Master>,
    #[account(mut)]
    pub user_ata: Account<'info, TokenAccount>,
    #[account(init_if_needed,
        payer = signer,
        associated_token::mint = mint,
        associated_token::authority = game,
    )]
    pub game_ata: Account<'info, TokenAccount>,
    pub mint: Account<'info, Mint>,
    #[account(mut)]
    pub signer: Signer<'info>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

#[account]
pub struct Bet {
    pub id: u32,
    pub game: Pubkey,
    pub game_id: u32,
    pub user_address: Pubkey,
    pub bet_amount: u128,
    pub chosen_side: String,
    pub has_claimed: bool,
    pub bump: u8,
}

#[derive(Accounts)]
pub struct ChangeBet<'info> {
    #[account(mut)]
    pub game: Account<'info, Game>,
    #[account(mut)]
    pub bet_account: Account<'info, Bet>,
    #[account(mut)]
    pub signer: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct EndGame<'info> {
    #[account(mut)]
    pub game: Account<'info, Game>,
    #[account(mut)]
    pub master: Account<'info, Master>,
    #[account(mut, address = Pubkey::from_str("4K2dERdSDHzCM7g2h9pc82shJU4AMtvtLpa3vk3yrf8C").unwrap().key())]
    pub signer: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct ClaimPrize<'info> {
    #[account(mut)]
    pub bet_account: Account<'info, Bet>,
    #[account(mut)]
    pub game_account: Account<'info, Game>,
    #[account(mut)]
    pub user_ata: Account<'info, TokenAccount>,
    // #[account(mut, owner=game_account.key())]owner constraint was violated error
    #[account(mut)]
    pub game_ata: Account<'info, TokenAccount>,
    pub mint: Account<'info, Mint>,
    #[account(mut)]
    pub signer: Signer<'info>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}
