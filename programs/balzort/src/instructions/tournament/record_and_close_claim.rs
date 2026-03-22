use crate::states::{Game, Tournament, TournamentEntry};
use crate::types::constants::*;
use crate::types::{GameError, PrizeClaimed, TournamentClosed, TournamentResultRecorded};
use crate::utils::parimutuel_weight;
use anchor_lang::prelude::*;

pub fn record_result_handler(
    ctx: Context<RecordTournamentResult>,
    elapsed_secs: u64,
    move_count: u32,
) -> Result<()> {
    let clock = Clock::get()?;
    let t = &mut ctx.accounts.tournament;
    let entry = &mut ctx.accounts.tournament_entry;

    require!(
        t.is_open(clock.unix_timestamp), GameError::InvalidState
    );
    require!(!entry.completed, GameError::InvalidState);

    let weight = parimutuel_weight(elapsed_secs, move_count);
    entry.completed = true;
    entry.parimutuel_weight = weight;
    t.total_completers = t.total_completers.saturating_add(1);
    t.cumulative_weight = t.cumulative_weight.saturating_add(weight);

     emit!(TournamentResultRecorded {
        tournament: ctx.accounts.tournament.key(),
        player: entry.player,
        weight,
        elapsed_secs,
        move_count,
    });
    Ok(())
}

#[derive(Accounts)]
pub struct RecordTournamentResult<'info> {
    pub player: Signer<'info>,
    #[account(mut, seeds=[SEED_TOURNAMENT, &tournament.tournament_id.to_le_bytes()], bump=tournament.bump)]
    pub tournament: Account<'info, Tournament>,
    #[account(mut, seeds=[SEED_TOURNAMENT_ENTRY, tournament.key().as_ref(), player.key().as_ref()], bump=tournament_entry.bump,
              constraint = tournament_entry.player == player.key() @ GameError::Unauthorized)]
    pub tournament_entry: Account<'info, TournamentEntry>,
}

pub fn close_handler(ctx: Context<CloseTournament>) -> Result<()> {
    let clock = Clock::get()?;
    let t = &mut ctx.accounts.tournament;
    require!(!t.is_closed, GameError::InvalidState);
    require!(
        clock.unix_timestamp >= t.end_time, GameError::InvalidState
    );

    let (fee_amount, net_pool) = Tournament::calculate_fee(t.prize_pool, t.treasury_fee_bps);
    t.net_prize_pool = net_pool;
    t.is_closed = true;

    if fee_amount > 0 {
        **t.to_account_info().try_borrow_mut_lamports()? -= fee_amount;
        **ctx.accounts.treasury.try_borrow_mut_lamports()? += fee_amount;
    }

    emit!(TournamentClosed {
        tournament: t.key(),
        total_entries: t.total_entries,
        total_completers: t.total_completers,
        prize_pool: t.prize_pool,
    });
    Ok(())
}

#[derive(Accounts)]
pub struct CloseTournament<'info> {
    pub authority: Signer<'info>,
    #[account(mut, seeds=[SEED_TOURNAMENT, &tournament.tournament_id.to_le_bytes()], bump=tournament.bump,
              constraint = tournament.authority == authority.key() @ GameError::NotTournamentAuthority)]
    pub tournament: Account<'info, Tournament>,
    #[account(seeds=[SEED_GAME], bump=game.bump)]
    pub game: Account<'info, Game>,
    /// CHECK: verified against game.treasury
    #[account(mut, constraint = treasury.key() == game.treasury @ GameError::Unauthorized)]
    pub treasury: UncheckedAccount<'info>,
    pub system_program: Program<'info, System>,
}

pub fn claim_prize_handler(ctx: Context<ClaimPrize>) -> Result<()> {
    let t = &mut ctx.accounts.tournament;
    let entry = &mut ctx.accounts.tournament_entry;
    require!(t.is_closed, GameError::InvalidState);
    require!(entry.completed, GameError::InvalidState);
    require!(!entry.has_claimed, GameError::InvalidState);

    let amount = if t.cumulative_weight == 0 {
        0u64
    } else {
        ((entry.parimutuel_weight as u128).saturating_mul(t.net_prize_pool as u128)
            / t.cumulative_weight) as u64
    };

    entry.has_claimed = true;
    if amount > 0 {
        **ctx
            .accounts
            .tournament
            .to_account_info()
            .try_borrow_mut_lamports()? -= amount;
        **ctx.accounts.player.try_borrow_mut_lamports()? += amount;
    }
    emit!(PrizeClaimed {
        tournament: ctx.accounts.tournament.key(),
        player: entry.player,
        amount,
    });
    Ok(())
}

#[derive(Accounts)]
pub struct ClaimPrize<'info> {
    #[account(mut)]
    pub player: Signer<'info>,
    #[account(mut, seeds=[SEED_TOURNAMENT, &tournament.tournament_id.to_le_bytes()], bump=tournament.bump)]
    pub tournament: Account<'info, Tournament>,
    #[account(mut, seeds=[SEED_TOURNAMENT_ENTRY, tournament.key().as_ref(), player.key().as_ref()], bump=tournament_entry.bump,
              constraint = tournament_entry.player == player.key() @ GameError::Unauthorized)]
    pub tournament_entry: Account<'info, TournamentEntry>,
    pub system_program: Program<'info, System>,
}



