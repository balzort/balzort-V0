# Ball Sort Privacy Game Protocol

## 1. Overview

Ball Sort is a privacy-preserving, fully on-chain puzzle game built on Solana using Anchor, Magicblock VRF, and MagicBlock Private Ephemeral Rollups (ER).

Unlike traditional Web3 games where every move requires a wallet popup and exposes player strategy, Ball Sort utilizes Session Keys and Private TEE (Trusted Execution Environment) execution. Players solve complex generated puzzles instantly and privately.

The protocol separates private gameplay from public settlement:

- Sensitive user actions (moves, undos, puzzle state) happen inside a private ephemeral rollup.
- Only finalized results (completion time, move count) are committed to Solana L1 for tournament settlement.

### Links

Frontend Repo  
https://github.com/samuelhorjet/ball-sort/tree/main/frontend

Smart Contract Repo  
https://github.com/samuelhorjet/ball-sort

Live Demo  
https://ball-sort.netlify.app/

---

# 2. Core Architecture

## 2.1 On-chain Program (L1)

Written in Rust using the Anchor framework and deployed on Solana.

Manages:

- Global game configuration and treasury
- MagicBlock VRF integration for provably fair puzzle generation
- Tournament lifecycle and parimutuel prize pools
- PDA ownership of tournament entries

---

## 2.2 Private Execution Layer (MagicBlock)

Enabled via MagicBlock Ephemeral Rollups and Permission Accounts.

Handles:

- Private `apply_move` and `apply_undo` execution
- Gasless transactions via Session Keys

Prevents:

Strategy Leakage / Copy-playing  
Competitors cannot see how you solved the puzzle on-chain.

RPC Lag  
Provides instant real-time gameplay without waiting for L1 block finality.

---

## 2.3 Public Settlement Layer

Final states are flushed back to Solana via TEE undelegation.

This layer:

- Verifies puzzle completion
- Calculates Parimutuel Weights
- Enables permissionless tournament reward claiming

---

# 3. Protocol Lifecycle (End-to-End Flow)

## 3.1 Protocol Initialization (Admin)

```rust
initialize_game_config(params)
```

Creates the global game config PDA.

Sets treasury wallet, protocol fees (`treasury_fee_bps`), and links the VRF oracle authorities.

---

## 3.2 Tournament Creation (Admin)

```rust
create_tournament(entry_fee, difficulty, duration_secs)
```

Defines a gameplay window:

`start_time → end_time`

Initializes the prize pool and tracks total completers.

---

## 3.3 Player Setup & Session Activation (Users)

```rust
create_player_auth()
open_session(session_key, expires_in_secs)
```

Creates the PlayerAuth PDA.

Injects a local Session Key authorized to sign gameplay transactions on the player's behalf, eliminating wallet popups.

---

## 3.4 Puzzle Generation (VRF)

```rust
init_puzzle(num_tubes, balls_per_tube, difficulty)
consume_randomness(randomness)
```

Player requests a new puzzle board.

MagicBlock VRF securely provides true randomness.

The `start_puzzle` instruction shuffles the balls based on the VRF seed.

---

## 3.5 Privacy Activation (TEE Delegation)

```rust
create_puzzle_permissions()
delegate_puzzle_permissions()
delegate_puzzle()
```

Creates permission accounts locking the `puzzle_board` and `puzzle_stats`.

Transfers PDA authority to the MagicBlock TEE.

From this point:

- Gameplay becomes privately mutable
- No public on-chain writes are emitted for individual moves

---

## 3.6 Private Gameplay

```rust
apply_move(from_tube, to_tube)
apply_undo()
```

Executed securely inside the TEE using the Session Key.

---

# 4. Pool Resolution & Settlement

## 4.1 Flush State Back to L1

```rust
undelegate_puzzle()
```

Called once the puzzle is solved or abandoned.

The TEE commits the final state:

- move_count
- undo_count
- elapsed_secs

This ends the private execution phase.

---

## 4.2 Tournament Finalization

```rust
finalize_puzzle()
record_tournament_result(elapsed_secs, move_count)
```

Computes the player's performance weight based on speed and efficiency.

Adds the player's weight to the tournament's cumulative_weight.

---

# 5. Scoring Model (Core Math)

Ball Sort uses a **dual scoring system**. Every completed puzzle produces two independent numbers that serve completely different purposes:

| | Individual Score | Tournament Weight |
|---|---|---|
| **Function** | `compute_score` | `parimutuel_weight` |
| **Purpose** | Personal performance tracking | Prize pool distribution |
| **Scope** | Absolute — fixed at solve time | Relative — shifts as others finish |
| **Stored in** | `puzzle_stats.final_score` | `tournament_entry.weight` |

---

## 5.1 Individual Score (`compute_score`)

Rewards the player for how well they solved the puzzle. The result is personal and does not change based on what other players do.

```rust
pub fn compute_score(
    difficulty: u8,
    move_count: u32,
    elapsed_secs: u64,
    undo_count: u32,
    num_colors: u8,
    max_capacity: u8,
) -> u64 {
    // Difficulty multiplier: Easy = 1x, Medium = 2x, Hard = 3x
    let base = BASE_POINTS * multiplier;

    // Efficiency: full points if moves <= optimal, scaled down otherwise
    let optimal = (num_colors * max_capacity) / 2;
    let efficiency = if actual <= optimal { base } else { base * optimal / actual };

    // Speed bonus: linear bonus for finishing under 300 seconds
    let speed_bonus = if elapsed_secs >= 300 { 0 } else { base * (300 - elapsed_secs) / 300 };

    // Undo penalty: -50 points per undo used
    let penalty = 50 * undo_count;

    efficiency + speed_bonus - penalty
}
```

### Factors

**Difficulty Multiplier**  
Hard puzzles yield up to 3x the base points of Easy.

**Move Efficiency**  
Full points if you solve within the optimal move count `(num_colors × max_capacity) / 2`. Every move beyond optimal proportionally reduces your score.

**Speed Bonus**  
A linear bonus added for finishing under 300 seconds. No bonus at or beyond 300 seconds.

**Undo Penalty**  
50 points deducted per undo used. Encourages clean solves.

---

## 5.2 Tournament Parimutuel Weight (`parimutuel_weight`)

Determines your **share of the prize pool** relative to all other tournament completers. This is not a personal score — it is a competitive weight. Your share shifts as other players submit results.

```rust
pub fn parimutuel_weight(elapsed_secs: u64, move_count: u32) -> u128 {
    let time  = elapsed_secs.max(1) as u128;
    let moves = (move_count as u128).max(1);

    PARIMUTUEL_SCALAR
        .checked_div(time.saturating_mul(moves))
        .unwrap_or(0)
}
```

### Prize Distribution Formula

```rust
player_reward = (player_weight * net_prize_pool) / cumulative_weight
```

### Factors

**Time Factor**  
Lower `elapsed_secs` produces a higher weight. Speed is the dominant factor.

**Move Factor**  
Lower `move_count` produces a higher weight. Fewer moves means a larger slice.

**Scalar**  
`PARIMUTUEL_SCALAR = 1_000_000_000_000` preserves precision across the integer division, ensuring small performance differences still produce meaningfully distinct weights.

### Key Distinction from Individual Score

A player can have a high `final_score` (they played efficiently against the optimal) but a low `parimutuel_weight` (other tournament players were simply faster). The two systems are entirely independent and do not reference each other.

---

# 6. Finalization & Public Settlement

## 6.1 Close Tournament (Admin)

```rust
close_tournament()
```

Requirements:

- Tournament end_time has passed
- Tournament is not already closed

Actions:

- Deduct the protocol/treasury fee
- Lock the `net_prize_pool`
- Open the window for players to claim

---

# 7. Reward Claiming (Users)

## 7.1 Completers (Claim Prize)

```rust
claim_prize()
```

Permissionless claim for users who successfully finished the puzzle.

Reward formula:

```rust
amount = (player_weight * net_prize_pool) / cumulative_weight
```

---

# 8. Key Guarantees

### Privacy

Individual puzzle moves (`apply_move`) are executed in a secure enclave. Competitors cannot scrape RPC data to copy your solution path.

### Fairness

MagicBlock VRF guarantees every generated board is uniquely and fairly randomized.

### Gasless UX

Session keys remove the need to sign transactions for every move.

### Verifiability

The final `move_count` and `elapsed_secs` are validated on Solana L1 against the TEE’s cryptographically signed state transition.

---

# 9. Summary

Ball Sort introduces a new standard for fully on-chain gaming by combining:

- Provably fair VRF puzzle generation
- Private, gasless ephemeral execution
- Trustless parimutuel tournament settlement

By leveraging MagicBlock Private Ephemeral Rollups and Solana’s PDA model, Ball Sort delivers a real-time and responsive gaming experience without sacrificing decentralization or privacy.