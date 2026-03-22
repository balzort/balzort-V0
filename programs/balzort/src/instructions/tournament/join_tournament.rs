use crate::states::{Player, Tournament, TournamentEntry};
use crate::types::constants::*;
use crate::types::{GameError, TournamentJoined};
use anchor_lang::prelude::*;

pub fn handle_join_tournament(ctx: Context<JoinTournament>) -> Result<()> {
    let clock = Clock::get()?;
    let t = &mut ctx.accounts.tournament;
    require!(
        !t.is_closed && clock.unix_timestamp >= t.start_time && clock.unix_timestamp < t.end_time, GameError::InvalidState
    );

    let ix = anchor_lang::solana_program::system_instruction::transfer(
        &ctx.accounts.signer.key(),
        &t.key(),
        t.entry_fee,
    );
    anchor_lang::solana_program::program::invoke(
        &ix,
        &[
            ctx.accounts.signer.to_account_info(),
            t.to_account_info(),
            ctx.accounts.system_program.to_account_info(),
        ],
    )?;

    t.prize_pool = t.prize_pool.saturating_add(t.entry_fee);
    t.total_entries = t.total_entries.saturating_add(1);

    let entry = &mut ctx.accounts.tournament_entry;
    entry.tournament = t.key();
    entry.player = ctx.accounts.player.key();
    entry.entry_deposit = t.entry_fee;
    entry.parimutuel_weight = 0;
    entry.completed = false;
    entry.has_claimed = false;
    entry.bump = ctx.bumps.tournament_entry;

    emit!(TournamentJoined {
        tournament: ctx.accounts.tournament.key(),
        player: ctx.accounts.player.key(),
    });
    Ok(())
}

#[derive(Accounts)]
pub struct JoinTournament<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,
    #[account(seeds=[SEED_PLAYER, signer.key().as_ref()], bump=player.bump)]
    pub player: Account<'info, Player>,
    #[account(mut, seeds=[SEED_TOURNAMENT, &tournament.tournament_id.to_le_bytes()], bump=tournament.bump)]
    pub tournament: Account<'info, Tournament>,
    #[account(init, payer=signer, space=TournamentEntry::SPACE,
              seeds=[SEED_TOURNAMENT_ENTRY, tournament.key().as_ref(), signer.key().as_ref()], bump)]
    pub tournament_entry: Account<'info, TournamentEntry>,
    pub system_program: Program<'info, System>,
}



