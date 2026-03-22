use crate::states::Game;
use crate::types::constants::*;
use crate::types::{GameError, GameInitialized};
use anchor_lang::prelude::*;

pub fn handle_initialize_game(
    ctx: Context<InitializeGame>,
    params: InitGameParams,
) -> Result<()> {
    require!(
        params.treasury_fee_bps <= MAX_TREASURY_FEE_BPS, GameError::InvalidInput
    );
    let cfg = &mut ctx.accounts.game;
    cfg.authority = ctx.accounts.authority.key();
    cfg.treasury = params.treasury;
    cfg.treasury_fee_bps = params.treasury_fee_bps;
    cfg.is_paused = false;
    cfg.tournament_count = 0;
    cfg.bump = ctx.bumps.game;
    emit!(GameInitialized {
        authority: cfg.authority,
        treasury: cfg.treasury,
    });
    Ok(())
}

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct InitGameParams {
    pub treasury: Pubkey,
    pub treasury_fee_bps: u16,
}

#[derive(Accounts)]
pub struct InitializeGame<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    #[account(init, payer=authority, space=Game::SPACE,
              seeds=[SEED_GAME], bump)]
    pub game: Account<'info, Game>,
    pub system_program: Program<'info, System>,
}




