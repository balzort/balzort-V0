use anchor_lang::prelude::*;
use ephemeral_vrf_sdk::instructions::{create_request_randomness_ix, RequestRandomnessParams};
use ephemeral_vrf_sdk::types::SerializableAccountMeta;

pub fn build_vrf_request_ix(
    payer: Pubkey,
    oracle_queue: Pubkey,
    player: Pubkey,
    puzzle_stats: Pubkey,  
) -> anchor_lang::solana_program::instruction::Instruction {
    create_request_randomness_ix(RequestRandomnessParams {
        payer,
        oracle_queue,
        callback_program_id: crate::ID,
        callback_discriminator: crate::instruction::ConsumeRandomness::DISCRIMINATOR.to_vec(),
        caller_seed: player.to_bytes(),
        accounts_metas: Some(vec![
            SerializableAccountMeta {
                pubkey: player,
                is_signer: false,
                is_writable: true,
            },
            SerializableAccountMeta {   
                pubkey: puzzle_stats,
                is_signer: false,
                is_writable: true,
            },
        ]),
        ..Default::default()
    })
}

