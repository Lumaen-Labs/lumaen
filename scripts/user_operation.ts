import * as anchor from "@coral-xyz/anchor";
import { Program, BN, web3 } from "@coral-xyz/anchor";
import {
  TOKEN_PROGRAM_ID,
  ASSOCIATED_TOKEN_PROGRAM_ID,
  getOrCreateAssociatedTokenAccount,
} from "@solana/spl-token";
import { CoreRouter } from "../target/types/core_router";
import * as fs from "fs";
import * as path from "path";
import * as readline from "readline";

/* ──────────────────────────────────────────────
   Types
────────────────────────────────────────────── */
interface DeploymentConfig {
  protocolState: web3.PublicKey;
  usdcMint: web3.PublicKey;
  solMint: web3.PublicKey;
  usdcMarket: web3.PublicKey;
  solMarket: web3.PublicKey;
  programId: web3.PublicKey;
}
/* ──────────────────────────────────────────────
   Helpers
────────────────────────────────────────────── */
function loadDeploymentConfig(): DeploymentConfig {
  const configPath = path.join(__dirname, "..", "deployment-config.json");
  
  try {
    const data = JSON.parse(fs.readFileSync(configPath, "utf-8"));
    
    console.log("Raw config data:", JSON.stringify(data, null, 2));
    
    // Validate each field before creating PublicKey
    const requiredFields = [
      'protocolState',
      'usdcMint',
      'solMint',
      'usdcMarket',
      'solMarket',
      'programId'
    ];
    
    for (const field of requiredFields) {
      if (!data[field]) {
        throw new Error(`Missing or invalid field in deployment-config.json: ${field}`);
      }
    }
    
    return {
      protocolState: new web3.PublicKey(data.protocolState),
      usdcMint: new web3.PublicKey(data.usdcMint),
      solMint: new web3.PublicKey(data.solMint),
      usdcMarket: new web3.PublicKey(data.usdcMarket),
      solMarket: new web3.PublicKey(data.solMarket),
      programId: new web3.PublicKey(data.programId),
    };
  } catch (error: any) {
    console.error("❌ Error loading deployment config:");
    console.error(error.message);
    throw error;
  }
}

/* ──────────────────────────────────────────────
   Manual Provider (no env vars)
────────────────────────────────────────────── */
function getManualProvider(): anchor.AnchorProvider {
  const connection = new web3.Connection("https://api.devnet.solana.com", "confirmed");
  const walletPath = path.join(process.env.HOME || ".", ".config", "solana", "id.json");
  const secret = JSON.parse(fs.readFileSync(walletPath, "utf-8"));
  const keypair = web3.Keypair.fromSecretKey(Uint8Array.from(secret));
  const wallet = new anchor.Wallet(keypair);
  const provider = new anchor.AnchorProvider(connection, wallet, { commitment: "confirmed" });
  anchor.setProvider(provider);
  return provider;
}

/* ──────────────────────────────────────────────
   CLI Setup
────────────────────────────────────────────── */
const rl = readline.createInterface({
  input: process.stdin,
  output: process.stdout,
});

function prompt(question: string): Promise<string> {
  return new Promise((resolve) => rl.question(question, resolve));
}

async function showMenu() {
  console.log("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
  console.log("🏦 CORE PROTOCOL - USER OPERATIONS");
  console.log("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
  console.log("1. 💰 Deposit USDC");
  console.log("2. ☀️  Deposit SOL");
  console.log("3. 📊 View My Positions");
  console.log("9. ❌ Exit");
  console.log("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");
}

/* ──────────────────────────────────────────────
   USER OPERATIONS
────────────────────────────────────────────── */


async function deposit(
  program: Program<CoreRouter>,
  wallet: anchor.Wallet,
  config: DeploymentConfig,
  mint: web3.PublicKey,
  amount: number,
  decimals: number
) {
  const isUSDC = mint.equals(config.usdcMint);
  const marketName = isUSDC ? "USDC" : "SOL";
  console.log(`\n💰 Depositing ${amount} ${marketName}...`);

  try {
    const connection = program.provider.connection;
    
    console.log("Fetching user token account...");
    const userTokenAccount = await getOrCreateAssociatedTokenAccount(
      connection,
      wallet.payer,
      mint,
      wallet.publicKey
    );
    console.log("User token account:", userTokenAccount.address.toString());

    // Use the market PDA from config
    const marketPda = isUSDC ? config.usdcMarket : config.solMarket;
    console.log("Market PDA:", marketPda.toString());

    // Derive user position PDA - FIXED: use "user_account" not "user_position"
    const [userPosition] = web3.PublicKey.findProgramAddressSync(
      [Buffer.from("user_account"), wallet.publicKey.toBuffer(), marketPda.toBuffer()],
      program.programId
    );
    console.log("User Position PDA:", userPosition.toString());

    // Initialize user position if not exists
    try {
      await program.account.userPosition.fetch(userPosition);
      console.log("User position already exists");
    } catch (err) {
      console.log("Initializing user position...");
      const initTx = await program.methods
        .initializeUserPosition()
        .accounts({
          signer: wallet.publicKey,
          market: marketPda,
          userAccount: userPosition,
          systemProgram: web3.SystemProgram.programId,
        }as any)
        .rpc();
      console.log("User position initialized:", initTx);
      
      // Wait for confirmation
      await connection.confirmTransaction(initTx, "confirmed");
      console.log("User position initialization confirmed");
    }

    const depositAmount = new BN(amount * 10 ** decimals);
    console.log("Deposit amount (raw):", depositAmount.toString());
    
    // Derive supply vault PDA
    const [supplyVault] = web3.PublicKey.findProgramAddressSync(
      [Buffer.from("supply_vault"), marketPda.toBuffer()],
      program.programId
    );
    console.log("Supply Vault PDA:", supplyVault.toString());
    
    console.log("Submitting deposit transaction...");
    const tx = await program.methods
      .deposit(depositAmount)
      .accounts({
        signer: wallet.publicKey,
        mint: mint,
        market: marketPda,
        userTokenAccount: userTokenAccount.address,
        supplyVault: supplyVault,
        userPosition: userPosition,
        tokenProgram: TOKEN_PROGRAM_ID,
        associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
        systemProgram: web3.SystemProgram.programId,
      }as any)
      .rpc();

    console.log("✅ Deposit successful!");
    console.log(`   → https://explorer.solana.com/tx/${tx}?cluster=devnet`);
  } catch (error: any) {
    console.error("❌ Deposit failed:");
    console.error("Error message:", error.message);
    if (error.logs) {
      console.error("Error logs:", error.logs);
    }
    if (error.stack) {
      console.error("Stack trace:", error.stack);
    }
    throw error;
  }
}
/* ──────────────────────────────────────────────
   MAIN LOOP
────────────────────────────────────────────── */
async function main() {
  try {
    console.log("Loading deployment config...");
    const baseConfig = loadDeploymentConfig();
    console.log("Config loaded successfully");

    console.log("Setting up provider...");
    const provider = getManualProvider();
    console.log("Provider setup complete");

    console.log("Loading IDL...");
    const idlPath = path.join(__dirname, "../target/idl/core_router.json");
    const idl = JSON.parse(fs.readFileSync(idlPath, "utf-8"));
    console.log("IDL loaded");

    console.log("Initializing program...");
    const program = new anchor.Program(idl, provider) as Program<CoreRouter>;
    const wallet = provider.wallet as anchor.Wallet;

    console.log("\n🏦 Connected to Core Protocol");
    console.log("Wallet:", wallet.publicKey.toString());
    console.log("Program:", program.programId.toString());
    console.log("Cluster: Devnet\n");

    while (true) {
      await showMenu();
      const choice = (await prompt("Select an option: ")).trim();

      switch (choice) {
        case "1": {
          const amountStr = await prompt("Enter USDC amount: ");
          const amount = parseFloat(amountStr);
          if (isNaN(amount) || amount <= 0) {
            console.log("❌ Invalid amount");
            break;
          }
          await deposit(program, wallet, baseConfig, baseConfig.usdcMint, amount, 6);
          break;
        }
        case "2": {
          const amountStr = await prompt("Enter SOL amount: ");
          const amount = parseFloat(amountStr);
          if (isNaN(amount) || amount <= 0) {
            console.log("❌ Invalid amount");
            break;
          }
          await deposit(program, wallet, baseConfig, baseConfig.solMint, amount, 9);
          break;
        }
        case "3":
          console.log("📊 View Positions - Coming soon!");
          break;
        case "9":
          console.log("\n👋 Goodbye!");
          rl.close();
          process.exit(0);
        default:
          console.log("Invalid option, please try again.");
      }
    }
  } catch (err: any) {
    console.error("\n❌ Error:", err.message || err);
    if (err.stack) {
      console.error("Stack trace:", err.stack);
    }
    rl.close();
    process.exit(1);
  }
}

main();