import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { Balzort } from "../target/types/balzort";
import {
  Connection,
  PublicKey,
  SystemProgram,
  Transaction,
  sendAndConfirmTransaction,
} from "@solana/web3.js";
import { expect } from "chai";
import { loadOrGenerateKeypair, loadMainWallet, fundWallet } from "./utils";
import * as fs from "fs";
import * as path from "path";

import {
  DELEGATION_PROGRAM_ID,
  PERMISSION_PROGRAM_ID,
  delegationRecordPdaFromDelegatedAccount,
  delegationMetadataPdaFromDelegatedAccount,
  delegateBufferPdaFromDelegatedAccountAndOwnerProgram,
  getAuthToken,
  waitUntilPermissionActive,
  permissionPdaFromAccount,
} from "@magicblock-labs/ephemeral-rollups-sdk";
import * as nacl from "tweetnacl";

const SEED_GAME = "game";
const SEED_PLAYER = "player";
const TEE_URL = "https://tee.magicblock.app";
const TEE_WS_URL = "wss://tee.magicblock.app";
const ephemeralRpcEndpoint = TEE_URL;
const DEVNET_URL = "https://api.devnet.solana.com";

const TEE_VALIDATOR = new PublicKey(
  "MTEWGuqxUpYZGFJQcp8tLN7x5v9BSeoFHYWQQ3n3xzo",
);

const VRF_ORACLE_QUEUE = new PublicKey(
  "Cuj97ggrhhidhbu39TijNVqE74xvKJ69gDervRUXAxGh",
);

const sleep = (ms: number) => new Promise((r) => setTimeout(r, ms));

async function withRetry<T>(
  fn: () => Promise<T>,
  actionName: string,
  retries = 1,
  delayMs = 2000,
): Promise<T> {
  for (let i = 0; i < retries; i++) {
    try {
      return await fn();
    } catch (e) {
      if (i === retries - 1) throw e;
      console.log(
        `      ⚠️  ${actionName} failed (Attempt ${
          i + 1
        }/${retries}). Retrying in ${delayMs / 1000}s...`,
      );
      console.log(`      Error: ${(e as Error).message}`);
      await sleep(delayMs);
    }
  }
  throw new Error("Unreachable");
}

describe("Balzort devnet E2E test", () => {
  const admin = loadMainWallet();
  let player1 = loadOrGenerateKeypair("player1");
  let sessionKeypair = loadOrGenerateKeypair("session_key");

  const l1Connection = new Connection(DEVNET_URL, "confirmed");

  const l1Provider = new anchor.AnchorProvider(
    l1Connection,
    new anchor.Wallet(admin),
    { preflightCommitment: "confirmed" },
  );
  anchor.setProvider(l1Provider);
  const l1Program = anchor.workspace.Balzort as Program<Balzort>;

  const [gamePda] = PublicKey.findProgramAddressSync(
    [Buffer.from("game")],
    l1Program.programId,
  );
  let [playerPda] = PublicKey.findProgramAddressSync(
    [Buffer.from("player"), player1.publicKey.toBuffer()],
    l1Program.programId,
  );

  let puzzleBoardPda: PublicKey;
  let puzzleStatsPda: PublicKey;
  let puzzleNonce = BigInt(0);

  before(async () => {
    try {
      const authInfo = await l1Connection.getAccountInfo(playerPda);
      if (authInfo) {
        const auth = await l1Program.account.player.fetch(playerPda);

        if (auth.sessionKey !== null) {
          console.log("⚠️ Active puzzle detected — clearing old keys...");

          const walletDir = path.join(process.cwd(), "tests", "wallets");
          ["player1.json", "session_key.json"].forEach((file) => {
            const fullPath = path.join(walletDir, file);
            if (fs.existsSync(fullPath)) fs.unlinkSync(fullPath);
          });

          player1 = loadOrGenerateKeypair("player1");
          sessionKeypair = loadOrGenerateKeypair("session_key");

          [playerPda] = PublicKey.findProgramAddressSync(
            [Buffer.from("player"), player1.publicKey.toBuffer()],
            l1Program.programId,
          );
        }
      }
    } catch (e: any) {
      console.warn(
        "Could not check player auth state (likely layout mismatch). Deleting old wallets and generating new ones. Error:",
        e.message || e,
      );
      const walletDir = path.join(process.cwd(), "tests", "wallets");
      ["player1.json", "session_key.json"].forEach((file) => {
        const fullPath = path.join(walletDir, file);
        if (fs.existsSync(fullPath)) fs.unlinkSync(fullPath);
      });

      player1 = loadOrGenerateKeypair("player1");
      sessionKeypair = loadOrGenerateKeypair("session_key");

      [playerPda] = PublicKey.findProgramAddressSync(
        [Buffer.from("player"), player1.publicKey.toBuffer()],
        l1Program.programId,
      );
    }

    await fundWallet(l1Connection, admin, player1, 0.2);
  });

  it("Initialize GameConfig (Admin)", async () => {
    const configAcc = await l1Connection.getAccountInfo(gamePda);
    if (!configAcc) {
      console.log("Initializing GameConfig...");
    const tx = await l1Program.methods
      .initializeGame({
        treasury: admin.publicKey,
        treasuryFeeBps: 200,
      })
      .accountsPartial({
        authority: admin.publicKey,
        game: gamePda,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .signers([admin])
      .rpc();
    } else {
      console.log("GameConfig already initialized.");
    }
  });

  it("Mega-Tx Onboarding (Player1)", async () => {
    console.log("Mega-Tx Onboarding...");
    const authAcc = await l1Connection.getAccountInfo(playerPda);

    const tx = new Transaction();
    tx.add(
      SystemProgram.transfer({
        fromPubkey: player1.publicKey,
        toPubkey: sessionKeypair.publicKey,
        lamports: 0.05 * 1000000000,
      }),
    );

    if (!authAcc) {
      console.log("  Creating PlayerAuth within Mega-Tx...");
     const tx = await l1Program.methods
      .createPlayer()
      .accountsPartial({
        signer: player1.publicKey,
        player: playerPda,
        game: gamePda,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .signers([player1])
      .rpc();
    } else {
      console.log("  PlayerAuth already created.");
    }

    tx.add(
      await l1Program.methods
        .openSession(sessionKeypair.publicKey, 3600)
        .accountsPartial({
          signer: player1.publicKey,
          player: playerPda,
        })
        .instruction(),
    );

    tx.feePayer = player1.publicKey;
    const { blockhash } = await l1Connection.getLatestBlockhash();
    tx.recentBlockhash = blockhash;
    tx.sign(player1);

    const sig = await sendAndConfirmTransaction(l1Connection, tx, [player1], {
      skipPreflight: true,
      commitment: "confirmed",
    });
    console.log("  ✅ Mega-Tx Onboarding signature:", sig);
  });

  it("Initialize Puzzle (L1 - Signed by Session)", async () => {
    console.log("Requesting puzzle start & VRF randomness...");

    let authAcc = await l1Program.account.player.fetch(playerPda);
    puzzleNonce = BigInt(authAcc.puzzlesStartedNonce.toString());

    const nonceBuf = Buffer.alloc(8);
    nonceBuf.writeBigUInt64LE(puzzleNonce);

    [puzzleBoardPda] = PublicKey.findProgramAddressSync(
      [Buffer.from("puzzle_board"), playerPda.toBuffer(), nonceBuf],
      l1Program.programId,
    );

    [puzzleStatsPda] = PublicKey.findProgramAddressSync(
      [Buffer.from("puzzle_stats"), playerPda.toBuffer(), nonceBuf],
      l1Program.programId,
    );

    console.log("  Calling initPuzzle...");
    const tx = new Transaction();
    tx.add(
      await l1Program.methods
        .initPuzzle(4, 4, 1)
        .accountsPartial({
          payer: sessionKeypair.publicKey,
          signer: sessionKeypair.publicKey,
          player: playerPda,
          game: gamePda,
          puzzleBoard: puzzleBoardPda,
          puzzleStats: puzzleStatsPda,
          oracleQueue: VRF_ORACLE_QUEUE,
          systemProgram: SystemProgram.programId,
        })
        .instruction(),
    );

    tx.feePayer = sessionKeypair.publicKey;
    const { blockhash } = await l1Connection.getLatestBlockhash();
    tx.recentBlockhash = blockhash;
    tx.sign(sessionKeypair);

    const sig = await sendAndConfirmTransaction(
      l1Connection,
      tx,
      [sessionKeypair],
      {
        skipPreflight: true,
        commitment: "confirmed",
      },
    );
    console.log("  InitPuzzle Mega-Tx signature:", sig);
  });

  it("Wait for VRF auto-callback", async function () {
    this.timeout(10000);
    console.log("Waiting for VRF devnet auto-callback...");

    let isReady = false;
    let statsData: any = null;
    let authData: any = null;
    const TIMEOUT_MS = 5000;
    const start = Date.now();

    while (Date.now() - start < TIMEOUT_MS) {
      try {
        statsData = await l1Program.account.puzzleStats.fetch(puzzleStatsPda);
        if (statsData.status === 1) {
          isReady = true;
          console.log(
            `VRF fulfilled in ${((Date.now() - start) / 1000).toFixed(1)}s!`,
          );
          break;
        }
      } catch (e) {
        console.log("Polling for VRF callback...");
      }
      await sleep(3000);
    }

    expect(isReady, "VRF callback timed out after 70s").to.be.true;
    expect(statsData).to.not.be.null;

    authData = await l1Program.account.player.fetch(playerPda);

    console.log("\n=== VRF Randomness Verification ===");
    console.log(
      "  - puzzle_stats.status (expect 1/BoardReady):",
      statsData.status,
    );
    console.log(
      "  - vrf_randomness (hex):",
      Buffer.from(authData.vrfRandomness).toString("hex"),
    );
    console.log("  - puzzle_num_tubes:", statsData.numTubes);
    console.log("  - puzzle_balls_per_tube:", statsData.ballsPerTube);
    console.log("  - puzzle_difficulty:", statsData.difficulty);

    const isAllZeros = authData.vrfRandomness.every(
      (byte: number) => byte === 0,
    );
    expect(isAllZeros, "VRF randomness should not be all zeros").to.be.false;
    console.log("✓ Randomness is non-zero and valid!");
    console.log("===================================\n");
  });

  it("Start Puzzle (L1 - Signed by Session)", async () => {
    console.log("Starting puzzle on L1 (shuffling board)...");

    const l1Tx = new Transaction().add(
      await l1Program.methods
        .startPuzzle()
        .accountsPartial({
          signer: sessionKeypair.publicKey,
          player: playerPda,
          game: gamePda,
          puzzleBoard: puzzleBoardPda,
          puzzleStats: puzzleStatsPda,
        })
        .instruction(),
    );
    l1Tx.feePayer = sessionKeypair.publicKey;
    const { blockhash: l1Blockhash } = await l1Connection.getLatestBlockhash();
    l1Tx.recentBlockhash = l1Blockhash;
    l1Tx.sign(sessionKeypair);

    const sig = await sendAndConfirmTransaction(
      l1Connection,
      l1Tx,
      [sessionKeypair],
      {
        skipPreflight: true,
        commitment: "confirmed",
      },
    );
    console.log("  ✅ L1 Start Puzzle updated. Signature:", sig);
  });

  it("Delegate permissions and puzzle to TEE (L1)", async () => {
    console.log(
      "  Securing Game State with MagicBlock Permissions and Delegating to TEE...",
    );

    const puzzleBoardPermission = permissionPdaFromAccount(puzzleBoardPda);
    const puzzleStatsPermission = permissionPdaFromAccount(puzzleStatsPda);

    const tx = new Transaction();

    // 1. Create Permissions
    tx.add(
      await l1Program.methods
        .createPuzzlePermissions()
        .accountsPartial({
          payer: sessionKeypair.publicKey,
          player: playerPda,
          game: gamePda,
          puzzleBoard: puzzleBoardPda,
          puzzleStats: puzzleStatsPda,
          puzzleBoardPermission,
          puzzleStatsPermission,
          permissionProgram: PERMISSION_PROGRAM_ID,
          systemProgram: SystemProgram.programId,
        })
        .instruction(),
    );

    // 2. Delegate Permissions
    tx.add(
      await l1Program.methods
        .delegatePuzzlePermissions()
        .accountsPartial({
          payer: sessionKeypair.publicKey,
          player: playerPda,
          game: gamePda,
          puzzleBoard: puzzleBoardPda,
          puzzleStats: puzzleStatsPda,
          puzzleBoardPermission,
          puzzleStatsPermission,
          permissionProgram: PERMISSION_PROGRAM_ID,
          delegationProgram: DELEGATION_PROGRAM_ID,
          boardDelegationBuffer:
            delegateBufferPdaFromDelegatedAccountAndOwnerProgram(
              puzzleBoardPermission,
              PERMISSION_PROGRAM_ID,
            ),
          boardDelegationRecord: delegationRecordPdaFromDelegatedAccount(
            puzzleBoardPermission,
          ),
          boardDelegationMetadata: delegationMetadataPdaFromDelegatedAccount(
            puzzleBoardPermission,
          ),
          statsDelegationBuffer:
            delegateBufferPdaFromDelegatedAccountAndOwnerProgram(
              puzzleStatsPermission,
              PERMISSION_PROGRAM_ID,
            ),
          statsDelegationRecord: delegationRecordPdaFromDelegatedAccount(
            puzzleStatsPermission,
          ),
          statsDelegationMetadata: delegationMetadataPdaFromDelegatedAccount(
            puzzleStatsPermission,
          ),
          validator: TEE_VALIDATOR,
          systemProgram: SystemProgram.programId,
        })
        .instruction(),
    );

    // 3. Delegate Puzzle Components
    tx.add(
      await l1Program.methods
        .delegatePuzzle()
        .accountsPartial({
          payer: sessionKeypair.publicKey,
          player: playerPda,
          game: gamePda,
          puzzleBoard: puzzleBoardPda,
          puzzleStats: puzzleStatsPda,
          validator: TEE_VALIDATOR,
        })
        .instruction(),
    );

    tx.feePayer = sessionKeypair.publicKey;
    const { blockhash } = await l1Connection.getLatestBlockhash();
    tx.recentBlockhash = blockhash;
    tx.sign(sessionKeypair);

    const sig = await sendAndConfirmTransaction(
      l1Connection,
      tx,
      [sessionKeypair],
      {
        skipPreflight: true,
        commitment: "confirmed",
      },
    );

    console.log("    ✔ Secured & Delegated to TEE (Sig):", sig);

    console.log(`    ⏳ Waiting for TEE to index delegation...`);
    await waitUntilPermissionActive(ephemeralRpcEndpoint, puzzleBoardPda);
    console.log(`    ✨ TEE Synchronization Complete!`);
  });

  it("Apply Move 1 (TEE - Signed by Session)", async () => {
    console.log("Applying Move 1 using Session Key on TEE...");

    const authToken = await getAuthToken(
      ephemeralRpcEndpoint,
      sessionKeypair.publicKey,
      async (msg) => nacl.sign.detached(msg, sessionKeypair.secretKey),
    );

    const teeConnection = new anchor.web3.Connection(
      `${TEE_URL}?token=${authToken.token}`,
      {
        commitment: "confirmed",
        wsEndpoint: `${TEE_WS_URL}?token=${authToken.token}`,
      },
    );

    const teeProvider = new anchor.AnchorProvider(
      teeConnection,
      new anchor.Wallet(sessionKeypair),
      { commitment: "confirmed", preflightCommitment: "confirmed" },
    );
    const teeProgram = new anchor.Program(l1Program.idl, teeProvider);

    await withRetry(async () => {
      const applyMoveIx = await teeProgram.methods
        .applyMove(0, 3)
        .accountsPartial({
          signer: sessionKeypair.publicKey,
          player: playerPda,
          game: gamePda,
          puzzleBoard: puzzleBoardPda,
          puzzleStats: puzzleStatsPda,
        })
        .instruction();

      const tx = new Transaction().add(applyMoveIx);
      tx.feePayer = sessionKeypair.publicKey;
      tx.recentBlockhash = (await teeConnection.getLatestBlockhash()).blockhash;

      const sig = await sendAndConfirmTransaction(
        teeConnection,
        tx,
        [sessionKeypair],
        {
          skipPreflight: true,
        },
      );
      console.log("  ✅ Apply Move 1 (TEE) signature:", sig);
    }, "Apply Move 1 (TEE)");
  });

  it("Apply Undo (TEE - Signed by Session)", async () => {
    console.log("Applying Undo using Session Key on TEE...");

    const authToken = await getAuthToken(
      ephemeralRpcEndpoint,
      sessionKeypair.publicKey,
      async (msg) => nacl.sign.detached(msg, sessionKeypair.secretKey),
    );

    const teeConnection = new anchor.web3.Connection(
      `${TEE_URL}?token=${authToken.token}`,
      {
        commitment: "confirmed",
        wsEndpoint: `${TEE_WS_URL}?token=${authToken.token}`,
      },
    );

    const teeProvider = new anchor.AnchorProvider(
      teeConnection,
      new anchor.Wallet(sessionKeypair),
      { commitment: "confirmed", preflightCommitment: "confirmed" },
    );
    const teeProgram = new anchor.Program(l1Program.idl, teeProvider);

    await withRetry(async () => {
      const applyUndoIx = await teeProgram.methods
        .applyUndo()
        .accountsPartial({
          signer: sessionKeypair.publicKey,
          player: playerPda,
          game: gamePda,
          puzzleBoard: puzzleBoardPda,
          puzzleStats: puzzleStatsPda,
        })
        .instruction();

      const tx = new Transaction().add(applyUndoIx);
      tx.feePayer = sessionKeypair.publicKey;
      tx.recentBlockhash = (await teeConnection.getLatestBlockhash()).blockhash;

      const sig = await sendAndConfirmTransaction(
        teeConnection,
        tx,
        [sessionKeypair],
        {
          skipPreflight: true,
        },
      );
      console.log("  ✅ Apply Undo (TEE) signature:", sig);
    }, "Apply Undo (TEE)");
  });

  it("Abandon Puzzle & Undelegate (TEE -> L1)", async () => {
    console.log("  🚀 Flushing State back to L1 (Undelegating)...");

    const authToken = await getAuthToken(
      ephemeralRpcEndpoint,
      sessionKeypair.publicKey,
      async (msg) => nacl.sign.detached(msg, sessionKeypair.secretKey),
    );

    const teeConnection = new anchor.web3.Connection(
      `${TEE_URL}?token=${authToken.token}`,
      {
        commitment: "confirmed",
        wsEndpoint: `${TEE_WS_URL}?token=${authToken.token}`,
      },
    );

    const teeProvider = new anchor.AnchorProvider(
      teeConnection,
      new anchor.Wallet(sessionKeypair),
      { commitment: "confirmed" },
    );
    const teeProgram = new anchor.Program(l1Program.idl, teeProvider);

    await withRetry(async () => {
      const undelegateIx = await teeProgram.methods
        .undelegatePuzzle()
        .accountsPartial({
          payer: sessionKeypair.publicKey,
          player: playerPda,
          game: gamePda,
          puzzleBoard: puzzleBoardPda,
          puzzleStats: puzzleStatsPda,
        })
        .instruction();

      const tx = new Transaction().add(undelegateIx);
      tx.feePayer = sessionKeypair.publicKey;
      tx.recentBlockhash = (await teeConnection.getLatestBlockhash()).blockhash;

      let sig: string;
      try {
        sig = await sendAndConfirmTransaction(
          teeConnection,
          tx,
          [sessionKeypair],
          { skipPreflight: true },
        );
      } catch (e: any) {
        if (e.logs) {
          console.error("Undelegate TEE Transaction Logs:");
          console.error(e.logs.join("\n"));
        } else {
          console.error(e);
        }
        throw e;
      }
      console.log("    ✅ State flushed to L1 (Undelegate Sig):", sig);
    }, "Undelegate Puzzle (TEE)");

    // Give L1 a moment to sync
    console.log("    ⏳ Waiting for L1 synchronization...");
    await sleep(2000);

    console.log("  🚀 Sending 'abandonPuzzle' to L1 to update status...");
    const l1Tx = new Transaction().add(
      await l1Program.methods
        .abandonPuzzle()
        .accountsPartial({
          signer: sessionKeypair.publicKey,
          player: playerPda,
          game: gamePda,
          puzzleStats: puzzleStatsPda,
        })
        .instruction(),
    );
    l1Tx.feePayer = sessionKeypair.publicKey;
    const { blockhash: l1Blockhash } = await l1Connection.getLatestBlockhash();
    l1Tx.recentBlockhash = l1Blockhash;
    l1Tx.sign(sessionKeypair);

    const l1Sig = await sendAndConfirmTransaction(
      l1Connection,
      l1Tx,
      [sessionKeypair],
      {
        skipPreflight: true,
        commitment: "confirmed",
      },
    );
    console.log("  ✅ L1 Abandon signature:", l1Sig);

    const statsInfo = await l1Program.account.puzzleStats.fetch(puzzleStatsPda);
    expect(statsInfo.status).to.equal(5); // 5 = Abandoned
  });

  it("Close Session (L1 - Player1)", async () => {
    console.log("Closing session on L1...");
    try {
      const sig = await l1Program.methods
        .closeSession()
        .accountsPartial({
          signer: player1.publicKey,
          player: playerPda,
        })
        .signers([player1])
        .rpc({ skipPreflight: true });

      console.log("  ✅ Session closed. Signature:", sig);

      const authData = await l1Program.account.player.fetch(playerPda);
      expect(authData.sessionKey).to.be.null;
    } catch (e: any) {
      console.error("Close Session error:", e.message || e);
      throw e;
    }
  });
});




