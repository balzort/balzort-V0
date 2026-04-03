use anchor_lang::prelude::*;
use anchor_spl::token_interface::{self, Mint, TokenInterface, TokenAccount, TransferChecked};
use crate::states::{Player, Tournament, TournamentEntry};
use crate::types::constants::*;
use crate::types::{GameError, TournamentJoined};

pub fn handle_join_tournament(ctx: Context<JoinTournament>, amount: u64) -> Result<()> {
    let clock = Clock::get()?;
    let t = &mut ctx.accounts.tournament;

    require!(
        !t.is_closed
            && clock.unix_timestamp >= t.start_time
            && clock.unix_timestamp < t.end_time,
        GameError::InvalidState
    );

    require!(amount >= t.entry_fee, GameError::InvalidInput);

    let transfer_ctx = CpiContext::new(
        ctx.accounts.token_program.to_account_info(),
        TransferChecked {
            from: ctx.accounts.player_token_account.to_account_info(),
            mint: ctx.accounts.token_mint.to_account_info(),
            to: ctx.accounts.tournament_vault.to_account_info(),
            authority: ctx.accounts.signer.to_account_info(),
        },
    );
    token_interface::transfer_checked(transfer_ctx, amount, ctx.accounts.token_mint.decimals)?;

    t.prize_pool = t.prize_pool.saturating_add(amount);
    t.total_entries = t.total_entries.saturating_add(1);

    let entry = &mut ctx.accounts.tournament_entry;
    entry.tournament = t.key();
    entry.player = ctx.accounts.player.key();
    entry.entry_deposit = amount;
    entry.puzzle_nonce = ctx.accounts.player.puzzles_started_nonce;
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
    pub payer: Signer<'info>,

    #[account(mut)]
    pub signer: Signer<'info>,

    #[account(
        seeds = [SEED_PLAYER, signer.key().as_ref()],
        bump = player.bump,
        constraint = player.wallet == signer.key() @ GameError::Unauthorized
    )]
    pub player: Account<'info, Player>,

    #[account(
        mut,
        seeds = [SEED_TOURNAMENT, &tournament.tournament_id.to_le_bytes()],
        bump = tournament.bump
    )]
    pub tournament: Account<'info, Tournament>,

    #[account(address = tournament.token_mint)]
    pub token_mint: InterfaceAccount<'info, Mint>,

    #[account(
        mut,
        seeds = [SEED_TOURNAMENT_VAULT, &tournament.tournament_id.to_le_bytes()],
        bump,
        token::mint = token_mint,
        token::authority = tournament,
    )]
    pub tournament_vault: InterfaceAccount<'info, TokenAccount>,

    #[account(
        mut,
        constraint = player_token_account.mint == tournament.token_mint @ GameError::InvalidInput,
        constraint = player_token_account.owner == signer.key() @ GameError::Unauthorized,
    )]
    pub player_token_account: InterfaceAccount<'info, TokenAccount>,

    #[account(
        init,
        payer = payer,
        space = TournamentEntry::SPACE,
        seeds = [SEED_TOURNAMENT_ENTRY, tournament.key().as_ref(), signer.key().as_ref()],
        bump
    )]
    pub tournament_entry: Account<'info, TournamentEntry>,

    pub token_program: Interface<'info, TokenInterface>,
    pub system_program: Program<'info, System>,
}