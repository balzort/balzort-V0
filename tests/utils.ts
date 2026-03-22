import { Keypair, Connection, LAMPORTS_PER_SOL } from "@solana/web3.js";
import * as fs from "fs";
import * as path from "path";
import * as os from "os";

const WALLETS_DIR = path.join(process.cwd(), "tests", "wallets");

export function loadOrGenerateKeypair(filename: string): Keypair {
  if (!fs.existsSync(WALLETS_DIR)) {
    fs.mkdirSync(WALLETS_DIR, { recursive: true });
  }

  const filePath = path.join(WALLETS_DIR, `${filename}.json`);
  if (fs.existsSync(filePath)) {
    const secretKeyString = fs.readFileSync(filePath, { encoding: "utf8" });
    const secretKey = Uint8Array.from(JSON.parse(secretKeyString));
    return Keypair.fromSecretKey(secretKey);
  } else {
    const keypair = Keypair.generate();
    fs.writeFileSync(filePath, JSON.stringify(Array.from(keypair.secretKey)));
    console.log(
      `Generated new keypair for ${filename}: ${keypair.publicKey.toBase58()}`,
    );
    return keypair;
  }
}

export function loadMainWallet(): Keypair {
  const defaultPath = path.join(os.homedir(), ".config", "solana", "id.json");
  if (!fs.existsSync(defaultPath)) {
    throw new Error(`Default Solana wallet not found at ${defaultPath}`);
  }
  const secretKeyString = fs.readFileSync(defaultPath, { encoding: "utf8" });
  const secretKey = Uint8Array.from(JSON.parse(secretKeyString));
  return Keypair.fromSecretKey(secretKey);
}

export async function fundWallet(
  connection: Connection,
  funder: Keypair,
  receiver: Keypair,
  amountSol: number,
) {
  const balance = await connection.getBalance(receiver.publicKey);
  if (balance < amountSol * LAMPORTS_PER_SOL) {
    console.log(
      `Funding ${receiver.publicKey.toBase58()} with ${amountSol} SOL from main wallet...`,
    );
    const { SystemProgram, Transaction } = await import("@solana/web3.js");
    const tx = new Transaction().add(
      SystemProgram.transfer({
        fromPubkey: funder.publicKey,
        toPubkey: receiver.publicKey,
        lamports: amountSol * LAMPORTS_PER_SOL,
      }),
    );
    const signature = await connection.sendTransaction(tx, [funder]);
    await connection.confirmTransaction(signature, "confirmed");
    console.log(`Funding complete. Signature: ${signature}`);
  }
}
