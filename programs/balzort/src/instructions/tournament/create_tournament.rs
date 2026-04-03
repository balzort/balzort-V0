use anchor_lang::prelude::*;
use anchor_spl::token_interface::{TokenInterface, TokenAccount, Mint};
use crate::states::{Game, Tournament};
use crate::types::constants::*;
use crate::types::{GameError, TournamentCreated};

pub fn handle_create_tournament(
    ctx: Context<CreateTournament>,
    p: CreateTournamentParams,
) -> Result<()> {
    require!(p.duration_secs > 0, GameError::ArithmeticOverflow);
    require!(p.max_time_secs > 0, GameError::InvalidInput);
    require!(p.num_tubes >= 2, GameError::InvalidInput);
    require!(p.balls_per_tube >= 2, GameError::InvalidInput);

    let clock = Clock::get()?;
    let cfg = &mut ctx.accounts.game;
    let t = &mut ctx.accounts.tournament;
    let id = cfg.tournament_count;
    cfg.tournament_count = cfg.tournament_count.saturating_add(1);

    t.authority = ctx.accounts.authority.key();
    t.token_mint = ctx.accounts.token_mint.key();
    t.entry_fee = p.entry_fee;
    t.prize_pool = 0;
    t.net_prize_pool = 0;
    t.treasury_fee_bps = cfg.treasury_fee_bps;
    t.difficulty = p.difficulty;
    t.num_tubes = p.num_tubes;
    t.balls_per_tube = p.balls_per_tube;
    t.max_time_secs = p.max_time_secs;
    t.start_time = clock.unix_timestamp;
    t.end_time = clock
        .unix_timestamp
        .checked_add(p.duration_secs)
        .ok_or(error!(GameError::ArithmeticOverflow))?;
    t.total_entries = 0;
    t.total_completers = 0;
    t.cumulative_weight = 0;
    t.is_closed = false;
    t.tournament_id = id;
    t.bump = ctx.bumps.tournament;

    emit!(TournamentCreated {
        tournament: t.key(),
        entry_fee: t.entry_fee,
        difficulty: t.difficulty,
        end_time: t.end_time,
        treasury_fee_bps: t.treasury_fee_bps,
    });
    Ok(())
}

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct CreateTournamentParams {
    pub entry_fee:      u64,
    pub difficulty:     u8,
    pub duration_secs:  i64,
    pub max_time_secs:  u64,
    pub num_tubes:      u8,
    pub balls_per_tube: u8,
}

#[derive(Accounts)]
pub struct CreateTournament<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,

    #[account(mut, seeds=[SEED_GAME], bump=game.bump,
              constraint = game.authority == authority.key() @ GameError::NotAuthority)]
    pub game: Account<'info, Game>,

    pub token_mint: InterfaceAccount<'info, Mint>,

    #[account(
        init,
        payer = authority,
        space = Tournament::SPACE,
        seeds = [SEED_TOURNAMENT, &game.tournament_count.to_le_bytes()],
        bump
    )]
    pub tournament: Account<'info, Tournament>,

    #[account(
        init,
        payer = authority,
        token::mint = token_mint,
        token::authority = tournament,
        seeds = [SEED_TOURNAMENT_VAULT, &game.tournament_count.to_le_bytes()],
        bump
    )]
    pub tournament_vault: InterfaceAccount<'info, TokenAccount>,

    pub token_program: Interface<'info, TokenInterface>,
    pub system_program: Program<'info, System>,
}