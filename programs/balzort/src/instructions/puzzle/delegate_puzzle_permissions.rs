use crate::states::{Game, Player};
use crate::types::constants::*;
use crate::types::GameError;
use anchor_lang::prelude::*;
use ephemeral_rollups_sdk::access_control::instructions::DelegatePermissionCpiBuilder;

#[derive(Accounts)]
pub struct DelegatePuzzlePermissions<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,

    #[account(
        seeds = [SEED_PLAYER, player.wallet.as_ref()], 
        bump = player.bump,
        constraint = player.wallet == payer.key() 
                  || player.session_key == Some(payer.key()) @ GameError::Unauthorized
    )]
    pub player: Account<'info, Player>,

    #[account(seeds=[SEED_GAME], bump=game.bump)]
    pub game: Account<'info, Game>,

    /// CHECK: The PuzzleBoard we want to protect
    #[account(mut)]
    pub puzzle_board: UncheckedAccount<'info>,

    /// CHECK: The PuzzleStats we want to protect 
    #[account(mut)]
    pub puzzle_stats: UncheckedAccount<'info>,

    /// CHECK: Validated by Permission Program
    #[account(mut)]
    pub puzzle_board_permission: UncheckedAccount<'info>,

    /// CHECK: Validated by Permission Program
    #[account(mut)]
    pub puzzle_stats_permission: UncheckedAccount<'info>,

    /// CHECK: The MagicBlock Permission Program ID
    pub permission_program: UncheckedAccount<'info>,

    /// CHECK: The MagicBlock Delegation Program
    pub delegation_program: UncheckedAccount<'info>,

    // === Board Delegation Derivations ===
    /// CHECK: Delegation buffer (Derived by client or SDK)
    #[account(mut)]
    pub board_delegation_buffer: UncheckedAccount<'info>,
    /// CHECK: Delegation record (Derived by client or SDK)
    #[account(mut)]
    pub board_delegation_record: UncheckedAccount<'info>,
    /// CHECK: Delegation metadata (Derived by client or SDK)
    #[account(mut)]
    pub board_delegation_metadata: UncheckedAccount<'info>,

    // === Stats Delegation Derivations ===
    /// CHECK: Delegation buffer (Derived by client or SDK)
    #[account(mut)]
    pub stats_delegation_buffer: UncheckedAccount<'info>,
    /// CHECK: Delegation record (Derived by client or SDK)
    #[account(mut)]
    pub stats_delegation_record: UncheckedAccount<'info>,
    /// CHECK: Delegation metadata (Derived by client or SDK)
    #[account(mut)]
    pub stats_delegation_metadata: UncheckedAccount<'info>,

    /// CHECK: The MagicBlock Ephemeral Rollup Validator (TEE)
    pub validator: UncheckedAccount<'info>,

    pub system_program: Program<'info, System>,
}

pub fn handle_delegate_puzzle_permissions(ctx: Context<DelegatePuzzlePermissions>) -> Result<()> {
    
    let puzzles_started = ctx.accounts.player.puzzles_started_nonce.saturating_sub(1);
    let puzzles_started_bytes = puzzles_started.to_le_bytes();
    
    // --- Delegate Board Permission ---
    let board_seeds = &[
        b"puzzle_board",
        ctx.accounts.player.to_account_info().key.as_ref(),
        &puzzles_started_bytes,
    ];
    let (_, board_bump) = Pubkey::find_program_address(board_seeds, ctx.program_id);
    let signer_seeds_board = &[
        b"puzzle_board",
        ctx.accounts.player.to_account_info().key.as_ref(),
        &puzzles_started_bytes,
        &[board_bump]
    ];

    DelegatePermissionCpiBuilder::new(&ctx.accounts.permission_program)
        .payer(&ctx.accounts.payer)
        .authority(&ctx.accounts.payer, false) 
        .permissioned_account(&ctx.accounts.puzzle_board, true) // board PDAs sign for their own permission
        .permission(&ctx.accounts.puzzle_board_permission)
        .system_program(&ctx.accounts.system_program)
        .owner_program(&ctx.accounts.permission_program)
        .delegation_buffer(&ctx.accounts.board_delegation_buffer)
        .delegation_record(&ctx.accounts.board_delegation_record)
        .delegation_metadata(&ctx.accounts.board_delegation_metadata)
        .delegation_program(&ctx.accounts.delegation_program)
        .validator(Some(&ctx.accounts.validator))
        .invoke_signed(&[signer_seeds_board])?;

    // --- Delegate Stats Permission ---
    let stats_seeds = &[
        b"puzzle_stats",
        ctx.accounts.player.to_account_info().key.as_ref(),
        &puzzles_started_bytes,
    ];
    let (_, stats_bump) = Pubkey::find_program_address(stats_seeds, ctx.program_id);
    let signer_seeds_stats = &[
        b"puzzle_stats",
        ctx.accounts.player.to_account_info().key.as_ref(),
        &puzzles_started_bytes,
        &[stats_bump]
    ];

    DelegatePermissionCpiBuilder::new(&ctx.accounts.permission_program)
        .payer(&ctx.accounts.payer)
        .authority(&ctx.accounts.payer, false) 
        .permissioned_account(&ctx.accounts.puzzle_stats, true) // stats PDAs sign for their own permission
        .permission(&ctx.accounts.puzzle_stats_permission)
        .system_program(&ctx.accounts.system_program)
        .owner_program(&ctx.accounts.permission_program)
        .delegation_buffer(&ctx.accounts.stats_delegation_buffer)
        .delegation_record(&ctx.accounts.stats_delegation_record)
        .delegation_metadata(&ctx.accounts.stats_delegation_metadata)
        .delegation_program(&ctx.accounts.delegation_program)
        .validator(Some(&ctx.accounts.validator))
        .invoke_signed(&[signer_seeds_stats])?;

    Ok(())
}



