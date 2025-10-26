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

interface DeploymentConfig {
  protocolState: web3.PublicKey;
  usdcMint: web3.PublicKey;
  solMint: web3.PublicKey;
  usdcMarket: web3.PublicKey;
  solMarket: web3.PublicKey;
  programId: web3.PublicKey;
}

function loadDeploymentConfig(): DeploymentConfig {
  const configPath = path.join(__dirname, "..", "deployment-config.json");
  const data = JSON.parse(fs.readFileSync(configPath, "utf-8"));
  return {
    protocolState: new web3.PublicKey(data.protocolState),
    usdcMint: new web3.PublicKey(data.usdcMint),
    solMint: new web3.PublicKey(data.solMint),
    usdcMarket: new web3.PublicKey(data.usdcMarket),
    solMarket: new web3.PublicKey(data.solMarket),
    programId: new web3.PublicKey(data.programId),
  };
}

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
    
    const userTokenAccount = await getOrCreateAssociatedTokenAccount(
      connection,
      wallet.payer,
      mint,
      wallet.publicKey
    );

    // CRITICAL: Derive market from MINT, not from config
    // This ensures consistency with how Anchor derives it
    const [marketPda] = web3.PublicKey.findProgramAddressSync(
      [Buffer.from("market"), mint.toBuffer()],
      program.programId
    );
    console.log("Market PDA (derived from mint):", marketPda.toString());
    console.log("Market PDA (from config):", (isUSDC ? config.usdcMarket : config.solMarket).toString());

    // Derive user position using the mint-derived market
    const [userPosition] = web3.PublicKey.findProgramAddressSync(
      [Buffer.from("user_account"), wallet.publicKey.toBuffer(), marketPda.toBuffer()],
      program.programId
    );
    console.log("User Position PDA:", userPosition.toString());

    try {
      await program.account.userPosition.fetch(userPosition);
      console.log("✅ User position exists");
    } catch (err) {
      console.log("Initializing user position...");
      console.log("\n🔍 DEBUG - Account Types:");
console.log("━━━━━━━━━━━━━━━━━━━━━━━━━━━");

// Check what mint actually is
const mintInfo = await connection.getAccountInfo(mint);
console.log("Mint account owner:", mintInfo?.owner.toString());
console.log("Expected: TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA");

// Check what market actually is  
const marketInfo = await connection.getAccountInfo(marketPda);
console.log("\nMarket account owner:", marketInfo?.owner.toString());
console.log("Expected:", program.programId.toString());

console.log("\nAccounts being passed:");
console.log("  mint:", mint.toString());
console.log("  market:", marketPda.toString());
console.log("━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");
      const initTx = await program.methods
        .initializeUserPosition(marketPda)
        .accounts({
          signer: wallet.publicKey,
          userAccount:userPosition,
          systemProgram: web3.SystemProgram.programId,
        }as any)
        .rpc();
      console.log("✅ User position initialized:", initTx);
      await connection.confirmTransaction(initTx, "confirmed");
    }

    const depositAmount = new BN(amount * 10 ** decimals);
    
    const [supplyVault] = web3.PublicKey.findProgramAddressSync(
      [Buffer.from("supply_vault"), marketPda.toBuffer()],
      program.programId
    );
    
    console.log("Submitting deposit with:");
    console.log("  - Market:", marketPda.toString());
    console.log("  - User Position:", userPosition.toString());
    console.log("  - Supply Vault:", supplyVault.toString());
    
    const tx = await program.methods
      .deposit(depositAmount)
      .accountsPartial({
        signer: wallet.publicKey,
        mint: mint,
        market: marketPda,
        userTokenAccount: userTokenAccount.address,
        supplyVault: supplyVault,
        userAccount: userPosition,
        tokenProgram: TOKEN_PROGRAM_ID,
        associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
        systemProgram: web3.SystemProgram.programId,
      })
      .rpc();

    console.log("✅ Deposit successful!");
    console.log(`   → https://explorer.solana.com/tx/${tx}?cluster=devnet`);
  } catch (error: any) {
    console.error("❌ Deposit failed:", error.message);
    if (error.logs) console.error("Logs:", error.logs);
    throw error;
  }
}

async function main() {
  try {
    const baseConfig = loadDeploymentConfig();
    const provider = getManualProvider();
    const idlPath = path.join(__dirname, "../target/idl/core_router.json");
    const idl = JSON.parse(fs.readFileSync(idlPath, "utf-8"));
    const program = new anchor.Program(idl, provider) as Program<CoreRouter>;
    const wallet = provider.wallet as anchor.Wallet;

    console.log("\n🏦 Connected to Core Protocol");
    console.log("Wallet:", wallet.publicKey.toString());
    console.log("Program:", program.programId.toString());

    while (true) {
      await showMenu();
      const choice = (await prompt("Select an option: ")).trim();

      switch (choice) {
        case "1": {
          const amount = parseFloat(await prompt("Enter USDC amount: "));
          if (isNaN(amount) || amount <= 0) {
            console.log("❌ Invalid amount");
            break;
          }
          await deposit(program, wallet, baseConfig, baseConfig.usdcMint, amount, 6);
          break;
        }
        case "2": {
          const amount = parseFloat(await prompt("Enter SOL amount: "));
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
          console.log("Invalid option");
      }
    }
  } catch (err: any) {
    console.error("\n❌ Error:", err.message);
    rl.close();
    process.exit(1);
  }
}

main();