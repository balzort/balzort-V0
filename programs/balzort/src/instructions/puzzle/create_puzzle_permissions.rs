use crate::states::{Game, Player};
use crate::types::constants::*;
use crate::types::GameError;
use anchor_lang::prelude::*;
use ephemeral_rollups_sdk::access_control::instructions::CreatePermissionCpiBuilder;
use ephemeral_rollups_sdk::access_control::structs::{Member, MembersArgs, AUTHORITY_FLAG};

#[derive(Accounts)]
pub struct CreatePuzzlePermissions<'info> {
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

    pub system_program: Program<'info, System>,
}

pub fn handle_create_puzzle_permissions(ctx: Context<CreatePuzzlePermissions>) -> Result<()> {
    
    // The player's active session key acts as the authority. If none, wallet acts as authority.
    let authority = ctx.accounts.player.session_key.unwrap_or(ctx.accounts.player.wallet);

    let member = Member {
        pubkey: authority,
        flags: AUTHORITY_FLAG,
    };
    let args = MembersArgs {
        members: Some(vec![member]),
    };

    let puzzles_started = ctx.accounts.player.puzzles_started_nonce.saturating_sub(1);
    let puzzles_started_bytes = puzzles_started.to_le_bytes();
    
    // --- Puzzle Board Permission ---
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

    CreatePermissionCpiBuilder::new(&ctx.accounts.permission_program)
        .payer(&ctx.accounts.payer)
        .system_program(&ctx.accounts.system_program)
        .permission(&ctx.accounts.puzzle_board_permission)
        .permissioned_account(&ctx.accounts.puzzle_board)
        .args(args.clone())
        .invoke_signed(&[signer_seeds_board])?;

    // --- Puzzle Stats Permission ---
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

    CreatePermissionCpiBuilder::new(&ctx.accounts.permission_program)
        .payer(&ctx.accounts.payer)
        .system_program(&ctx.accounts.system_program)
        .permission(&ctx.accounts.puzzle_stats_permission)
        .permissioned_account(&ctx.accounts.puzzle_stats)
        .args(args)
        .invoke_signed(&[signer_seeds_stats])?;
    
    Ok(())
}



