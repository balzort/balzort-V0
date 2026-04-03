use anchor_lang::prelude::*;
use ephemeral_rollups_sdk::anchor::ephemeral;
pub mod instructions;
pub mod states;
pub mod types;
pub mod utils;

use instructions::*;

declare_id!("7F8bQvi3ppn4i7APswRX23hUFuq41dtkguy8aWLLtic8");

#[ephemeral]
#[program]
pub mod balzort {
    use super::*;

    pub fn initialize_game(
        ctx: Context<InitializeGame>,
        params: InitGameParams,
    ) -> Result<()> {
        handle_initialize_game(ctx, params)
    }

    pub fn update_game(
        ctx: Context<UpdateGame>,
        params: UpdateGameParams,
    ) -> Result<()> {
        handle_update_game(ctx, params)
    }

    pub fn create_player(ctx: Context<CreatePlayer>) -> Result<()> {
        handle_create_player(ctx)
    }

    pub fn init_puzzle(
        ctx: Context<InitPuzzle>,
        num_tubes: u8,
        balls_per_tube: u8,
        difficulty: u8,
    ) -> Result<()> {
        instructions::puzzle::init_puzzle::handle_init_puzzle(
            ctx,
            num_tubes,
            balls_per_tube,
            difficulty,
        )
    }
    pub fn consume_randomness(ctx: Context<ConsumeRandomness>, randomness: [u8; 32]) -> Result<()> {
        instructions::puzzle::consume_randomness::handle_consume_randomness(ctx, randomness)
    }
    pub fn start_puzzle(ctx: Context<StartPuzzle>) -> Result<()> {
        instructions::puzzle::start_puzzle::handle_start_puzzle(ctx)
    }
    pub fn apply_move(ctx: Context<ApplyMove>, from_tube: u8, to_tube: u8) -> Result<()> {
        instructions::puzzle::apply_move::handle_apply_move(ctx, from_tube, to_tube)
    }
    pub fn apply_undo(ctx: Context<ApplyUndo>) -> Result<()> {
        instructions::puzzle::apply_undo::handle_apply_undo(ctx)
    }
    pub fn finalize_puzzle(ctx: Context<FinalizePuzzle>) -> Result<()> {
        instructions::puzzle::finalize_puzzle::handle_finalize_puzzle(ctx)
    }
    pub fn abandon_puzzle(ctx: Context<AbandonPuzzle>) -> Result<()> {
        instructions::puzzle::abandon_puzzle::handle_abandon_puzzle(ctx)
    }

    pub fn create_puzzle_permissions(ctx: Context<CreatePuzzlePermissions>) -> Result<()> {
        instructions::puzzle::create_puzzle_permissions::handle_create_puzzle_permissions(ctx)
    }
    pub fn delegate_puzzle_permissions(ctx: Context<DelegatePuzzlePermissions>) -> Result<()> {
        instructions::puzzle::delegate_puzzle_permissions::handle_delegate_puzzle_permissions(ctx)
    }
    pub fn delegate_puzzle(ctx: Context<DelegatePuzzle>) -> Result<()> {
        instructions::puzzle::delegate_puzzle::handle_delegate_puzzle(ctx)
    }
    pub fn undelegate_puzzle(ctx: Context<UndelegatePuzzle>) -> Result<()> {
        instructions::puzzle::delegate_puzzle::handle_undelegate_puzzle(ctx)
    }

    pub fn open_session(
        ctx: Context<OpenSession>,
        session_key: Pubkey,
        expires_in_secs: u32,
    ) -> Result<()> {
        instructions::session::open_session::handle_open_session(ctx, session_key, expires_in_secs)
    }
    pub fn close_session(ctx: Context<CloseSession>) -> Result<()> {
        instructions::session::close_session::handle_close_session(ctx)
    }

    pub fn create_tournament(
        ctx: Context<CreateTournament>,
        params: CreateTournamentParams,
    ) -> Result<()> {
        instructions::tournament::create_tournament::handle_create_tournament(ctx, params)
    }
    pub fn join_tournament(ctx: Context<JoinTournament>, amount: u64) -> Result<()> {
        instructions::tournament::join_tournament::handle_join_tournament(ctx, amount)
    }

    pub fn record_tournament_result(ctx: Context<RecordTournamentResult>) -> Result<()> {
        instructions::tournament::record_and_close_claim::record_result_handler(ctx)
    }
    pub fn close_tournament(ctx: Context<CloseTournament>) -> Result<()> {
        instructions::tournament::record_and_close_claim::close_handler(ctx)
    }
    pub fn claim_prize(ctx: Context<ClaimPrize>) -> Result<()> {
        instructions::tournament::record_and_close_claim::claim_prize_handler(ctx)
    }
}