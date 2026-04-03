use anchor_lang::prelude::*;
use anchor_spl::token_interface::{self, Mint, TokenInterface, TokenAccount, TransferChecked};
use crate::states::{Game, Player, PuzzleStats, Tournament, TournamentEntry};
use crate::types::constants::*;
use crate::types::{GameError, PrizeClaimed, TournamentClosed, TournamentResultRecorded};
use crate::utils::parimutuel_weight;

pub fn record_result_handler(ctx: Context<RecordTournamentResult>) -> Result<()> {
    let clock = Clock::get()?;

    // Allow either the player's wallet OR their active session key to sign
    require!(
        ctx.accounts.player.is_valid_signer(&ctx.accounts.signer.key(), clock.unix_timestamp),
        GameError::Unauthorized
    );

    let t = &mut ctx.accounts.tournament;
    let entry = &mut ctx.accounts.tournament_entry;
    let stats = &ctx.accounts.puzzle_stats;

    require!(t.is_open(clock.unix_timestamp), GameError::InvalidState);
    require!(!entry.completed, GameError::InvalidState);

    require!(stats.is_solved, GameError::InvalidState);

    require!(stats.num_tubes == t.num_tubes, GameError::InvalidInput);
    require!(stats.balls_per_tube == t.balls_per_tube, GameError::InvalidInput);
    require!(stats.difficulty == t.difficulty, GameError::InvalidInput);

    require!(
        stats.completed_at >= t.start_time && stats.completed_at < t.end_time,
        GameError::InvalidState
    );

    let elapsed_secs = (stats.completed_at - stats.started_at).max(0) as u64;
    require!(elapsed_secs <= t.max_time_secs, GameError::InvalidState);

    let weight = parimutuel_weight(elapsed_secs, stats.move_count);
    entry.completed = true;
    entry.parimutuel_weight = weight;
    t.total_completers = t.total_completers.saturating_add(1);
    t.cumulative_weight = t.cumulative_weight.saturating_add(weight);

    emit!(TournamentResultRecorded {
        tournament: ctx.accounts.tournament.key(),
        player: entry.player,
        weight,
        elapsed_secs,
        move_count: stats.move_count,
    });
    Ok(())
}

#[derive(Accounts)]
pub struct RecordTournamentResult<'info> {
    pub signer: Signer<'info>,

    #[account(
        seeds = [SEED_PLAYER, player.wallet.as_ref()],
        bump = player.bump,
    )]
    pub player: Account<'info, Player>,

    #[account(
        mut,
        seeds = [SEED_TOURNAMENT, &tournament.tournament_id.to_le_bytes()],
        bump = tournament.bump
    )]
    pub tournament: Account<'info, Tournament>,

    #[account(
        mut,
        seeds = [SEED_TOURNAMENT_ENTRY, tournament.key().as_ref(), player.wallet.as_ref()],
        bump = tournament_entry.bump,
        constraint = tournament_entry.player == player.key() @ GameError::Unauthorized
    )]
    pub tournament_entry: Account<'info, TournamentEntry>,

    #[account(
        seeds = [
            SEED_PUZZLE_STATS,
            player.key().as_ref(),
            &tournament_entry.puzzle_nonce.to_le_bytes()
        ],
        bump,
    )]
    pub puzzle_stats: Account<'info, PuzzleStats>,
}

pub fn close_handler(ctx: Context<CloseTournament>) -> Result<()> {
    let clock = Clock::get()?;

    require!(!ctx.accounts.tournament.is_closed, GameError::InvalidState);
    require!(
        clock.unix_timestamp >= ctx.accounts.tournament.end_time,
        GameError::InvalidState
    );

    let prize_pool = ctx.accounts.tournament.prize_pool;
    let treasury_fee_bps = ctx.accounts.tournament.treasury_fee_bps;
    let tournament_id_bytes = ctx.accounts.tournament.tournament_id.to_le_bytes();
    let tournament_bump = ctx.accounts.tournament.bump;
    let total_entries = ctx.accounts.tournament.total_entries;
    let total_completers = ctx.accounts.tournament.total_completers;
    let tournament_key = ctx.accounts.tournament.key();

    let (fee_amount, net_pool) = Tournament::calculate_fee(prize_pool, treasury_fee_bps);
    ctx.accounts.tournament.net_prize_pool = net_pool;
    ctx.accounts.tournament.is_closed = true;

    if fee_amount > 0 {
        let bump_bytes = [tournament_bump];
        let signer_seeds: &[&[&[u8]]] = &[&[
            SEED_TOURNAMENT,
            tournament_id_bytes.as_ref(),
            bump_bytes.as_ref(),
        ]];

        let transfer_ctx = CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            TransferChecked {
                from: ctx.accounts.tournament_vault.to_account_info(),
                mint: ctx.accounts.token_mint.to_account_info(),
                to: ctx.accounts.treasury_token_account.to_account_info(),
                authority: ctx.accounts.tournament.to_account_info(),
            },
            signer_seeds,
        );
        token_interface::transfer_checked(transfer_ctx, fee_amount, ctx.accounts.token_mint.decimals)?;
    }

    emit!(TournamentClosed {
        tournament: tournament_key,
        total_entries,
        total_completers,
        prize_pool,
    });
    Ok(())
}

#[derive(Accounts)]
pub struct CloseTournament<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,

    #[account(
        mut,
        seeds = [SEED_TOURNAMENT, &tournament.tournament_id.to_le_bytes()],
        bump = tournament.bump,
    )]
    pub tournament: Account<'info, Tournament>,

    #[account(address = tournament.token_mint)]
    pub token_mint: InterfaceAccount<'info, Mint>,

    #[account(
        mut,
        seeds = [SEED_TOURNAMENT_VAULT, &tournament.tournament_id.to_le_bytes()],
        bump,
        token::mint = tournament.token_mint,
        token::authority = tournament,
    )]
    pub tournament_vault: InterfaceAccount<'info, TokenAccount>,

    #[account(seeds = [SEED_GAME], bump = game.bump)]
    pub game: Account<'info, Game>,

    #[account(
        mut,
        constraint = treasury_token_account.owner == game.treasury @ GameError::Unauthorized,
        constraint = treasury_token_account.mint == tournament.token_mint @ GameError::InvalidInput,
    )]
    pub treasury_token_account: InterfaceAccount<'info, TokenAccount>,

    pub token_program: Interface<'info, TokenInterface>,
    pub system_program: Program<'info, System>,
}

pub fn claim_prize_handler(ctx: Context<ClaimPrize>) -> Result<()> {
    let t = &ctx.accounts.tournament;
    let entry = &ctx.accounts.tournament_entry;

    require!(t.is_closed, GameError::InvalidState);
    require!(!entry.has_claimed, GameError::InvalidState);

    let tournament_id_bytes = t.tournament_id.to_le_bytes();
    let tournament_bump = t.bump;
    let cumulative_weight = t.cumulative_weight;
    let net_prize_pool = t.net_prize_pool;
    let total_entries = t.total_entries;
    let parimutuel_weight_val = entry.parimutuel_weight;
    let tournament_key = ctx.accounts.tournament.key();
    let player_key = entry.player;

    let amount: u64 = if cumulative_weight == 0 {
        if total_entries == 0 {
            0
        } else {
            net_prize_pool / total_entries as u64
        }
    } else if !entry.completed {
        0
    } else {
        ((parimutuel_weight_val as u128)
            .saturating_mul(net_prize_pool as u128)
            .checked_div(cumulative_weight)
            .unwrap_or(0)) as u64
    };

    ctx.accounts.tournament_entry.has_claimed = true;

    if amount > 0 {
        let bump_bytes = [tournament_bump];
        let signer_seeds: &[&[&[u8]]] = &[&[
            SEED_TOURNAMENT,
            tournament_id_bytes.as_ref(),
            bump_bytes.as_ref(),
        ]];

        let transfer_ctx = CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            TransferChecked {
                from: ctx.accounts.tournament_vault.to_account_info(),
                mint: ctx.accounts.token_mint.to_account_info(),
                to: ctx.accounts.player_token_account.to_account_info(),
                authority: ctx.accounts.tournament.to_account_info(),
            },
            signer_seeds,
        );
        token_interface::transfer_checked(transfer_ctx, amount, ctx.accounts.token_mint.decimals)?;
    }

    emit!(PrizeClaimed {
        tournament: tournament_key,
        player: player_key,
        amount,
    });
    Ok(())
}

#[derive(Accounts)]
pub struct ClaimPrize<'info> {
    #[account(mut)]
    pub player: Signer<'info>,

    #[account(
        seeds = [SEED_PLAYER, player.key().as_ref()],
        bump = player_account.bump,
        constraint = player_account.wallet == player.key() @ GameError::Unauthorized
    )]
    pub player_account: Box<Account<'info, Player>>,

    #[account(
        mut,
        seeds = [SEED_TOURNAMENT, &tournament.tournament_id.to_le_bytes()],
        bump = tournament.bump
    )]
    pub tournament: Box<Account<'info, Tournament>>,

    #[account(address = tournament.token_mint)]
    pub token_mint: Box<InterfaceAccount<'info, Mint>>,

    #[account(
        mut,
        seeds = [SEED_TOURNAMENT_VAULT, &tournament.tournament_id.to_le_bytes()],
        bump,
        token::mint = tournament.token_mint,
        token::authority = tournament,
    )]
    pub tournament_vault: Box<InterfaceAccount<'info, TokenAccount>>,

    #[account(
        mut,
        seeds = [SEED_TOURNAMENT_ENTRY, tournament.key().as_ref(), player.key().as_ref()],
        bump = tournament_entry.bump,
        constraint = tournament_entry.player == player_account.key() @ GameError::Unauthorized
    )]
    pub tournament_entry: Box<Account<'info, TournamentEntry>>,

    #[account(
        mut,
        constraint = player_token_account.owner == player.key() @ GameError::Unauthorized,
        constraint = player_token_account.mint == tournament.token_mint @ GameError::InvalidInput,
    )]
    pub player_token_account: Box<InterfaceAccount<'info, TokenAccount>>,

    pub token_program: Interface<'info, TokenInterface>,
    pub system_program: Program<'info, System>,
}