import * as anchor from "@coral-xyz/anchor";
import { Program, BN, web3 } from "@coral-xyz/anchor";
import { createMint } from "@solana/spl-token";
import { CoreRouter } from "../target/types/core_router";
import * as fs from "fs";
import * as path from "path";

const PRICE_FEED_IDS = {
  SOL: "0xef0d8b6fda2ceba41da15d4095d1da392a0d2f8ed0c6c7bc0f4cfac8c280b56d",
  USDC: "0x41f3625971ca2ed2263e78573fe5ce23e13d2558ed3f2e47ab0f84fb9e7ae722",
};

interface DeploymentConfig {
  protocolState: string;
  usdcMint: string;
  solMint: string;
  usdcMarket: string;
  solMarket: string;
  programId: string;
}

async function saveConfig(config: DeploymentConfig) {
  const configPath = path.join(__dirname, "..", "deployment-config.json");
  fs.writeFileSync(configPath, JSON.stringify(config, null, 2));
  console.log(`\n📄 Config saved to: deployment-config.json`);
}

async function loadConfig(): Promise<DeploymentConfig | null> {
  try {
    const configPath = path.join(__dirname, "..", "deployment-config.json");
    return JSON.parse(fs.readFileSync(configPath, "utf-8"));
  } catch {
    return null;
  }
}

async function deploy() {
  console.log("🚀 Deploying Protocol to Devnet...\n");

  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace.CoreRouter as Program<CoreRouter>;
  const wallet = provider.wallet as anchor.Wallet;
  const connection = provider.connection;

  console.log("📍 Deployer:", wallet.publicKey.toString());
  console.log("📍 Program:", program.programId.toString());

  const balance = await connection.getBalance(wallet.publicKey);
  console.log("💰 Balance:", balance / web3.LAMPORTS_PER_SOL, "SOL\n");

  if (balance < 0.5 * web3.LAMPORTS_PER_SOL) {
    console.log("⚠️  Low balance! Run:");
    console.log(`solana airdrop 2 ${wallet.publicKey} --url devnet\n`);
    process.exit(1);
  }

  const existing = await loadConfig();
  if (existing) {
    console.log("⚠️  Deployment exists! Delete deployment-config.json to redeploy.\n");
    return existing;
  }

  console.log("━━━━━━━━━━━━━━━━━━━━━━━━━━━");
  console.log("📦 Creating Token Mints");
  console.log("━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

  const usdcMint = await createMint(connection, wallet.payer, wallet.publicKey, null, 6);
  console.log("✅ USDC Mint:", usdcMint.toString());

  const solMint = await createMint(connection, wallet.payer, wallet.publicKey, null, 9);
  console.log("✅ SOL Mint:", solMint.toString());

  console.log("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━");
  console.log("🔑 Deriving PDAs");
  console.log("━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

  const [protocolState] = web3.PublicKey.findProgramAddressSync(
    [Buffer.from("protocol_state")],
    program.programId
  );
  console.log("✅ Protocol State:", protocolState.toString());

  const [usdcMarket] = web3.PublicKey.findProgramAddressSync(
    [Buffer.from("market"), usdcMint.toBuffer()],
    program.programId
  );
  console.log("✅ USDC Market:", usdcMarket.toString());

  const [usdcSupplyVault] = web3.PublicKey.findProgramAddressSync(
    [Buffer.from("supply_vault"), usdcMarket.toBuffer()],
    program.programId
  );
  console.log("✅ USDC Supply Vault:", usdcSupplyVault.toString());

  const [solMarket] = web3.PublicKey.findProgramAddressSync(
    [Buffer.from("market"), solMint.toBuffer()],
    program.programId
  );
  console.log("✅ SOL Market:", solMarket.toString());

  const [solSupplyVault] = web3.PublicKey.findProgramAddressSync(
    [Buffer.from("supply_vault"), solMarket.toBuffer()],
    program.programId
  );
  console.log("✅ SOL Supply Vault:", solSupplyVault.toString());

  console.log("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━");
  console.log("🏛️  Initializing Protocol");
  console.log("━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

  const feeCollector = web3.Keypair.generate();

  try {
    await program.account.protocolState.fetch(protocolState);
    console.log("⚠️  Already initialized, skipping...");
  } catch {
    const tx = await program.methods
      .initializeProtocol()
      .accounts({ admin: wallet.publicKey, feeCollector: feeCollector.publicKey })
      .rpc();
    console.log("✅ Initialized Protocol! TX:", tx);
  }

  // ────────────────────────────────────────────────
  // USDC Market Initialization
  // ────────────────────────────────────────────────
  console.log("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━");
  console.log("💵 Initializing USDC Market");
  console.log("━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

  try {
    await program.account.market.fetch(usdcMarket);
    console.log("⚠️  Already initialized, skipping...");
  } catch {
    const config = {
      maxLtv: new BN(7500),
      liquidationThreshold: new BN(8000),
      liquidationPenalty: new BN(500),
      reserveFactor: new BN(1000),
      minDepositAmount: new BN(1_000_000),
      maxDepositAmount: new BN(1_000_000_000_000),
      minBorrowAmount: new BN(1_000_000),
      maxBorrowAmount: new BN(100_000_000_000),
      depositFee: new BN(0),
      withdrawFee: new BN(0),
      borrowFee: new BN(0),
      repayFee: new BN(0),
      pythFeedId: Array.from(Buffer.from(PRICE_FEED_IDS.USDC.slice(2), "hex")),
    };

    const tx = await program.methods
      .initializeMarket(config)
      .accounts({
        owner: wallet.publicKey,
        protocolState,
        underlyingMint: usdcMint,
        market: usdcMarket,
        supplyVault: usdcSupplyVault,
        tokenProgram: anchor.utils.token.TOKEN_PROGRAM_ID,
        systemProgram: web3.SystemProgram.programId,
      } as any)
      .rpc();
    console.log("✅ Initialized USDC Market! TX:", tx);
  }

  // ────────────────────────────────────────────────
  // SOL Market Initialization
  // ────────────────────────────────────────────────
  console.log("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━");
  console.log("☀️  Initializing SOL Market");
  console.log("━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

  try {
    await program.account.market.fetch(solMarket);
    console.log("⚠️  Already initialized, skipping...");
  } catch {
    const config = {
      maxLtv: new BN(7000),
      liquidationThreshold: new BN(7500),
      liquidationPenalty: new BN(500),
      reserveFactor: new BN(1000),
      minDepositAmount: new BN(100_000_000),
      maxDepositAmount: new BN(10_000_000_000_000),
      minBorrowAmount: new BN(100_000_000),
      maxBorrowAmount: new BN(1_000_000_000_000),
      depositFee: new BN(0),
      withdrawFee: new BN(0),
      borrowFee: new BN(0),
      repayFee: new BN(0),
      pythFeedId: Array.from(Buffer.from(PRICE_FEED_IDS.SOL.slice(2), "hex")),
    };

    const tx = await program.methods
      .initializeMarket(config)
      .accounts({
        owner: wallet.publicKey,
        protocolState,
        underlyingMint: solMint,
        market: solMarket,
        supplyVault: solSupplyVault,
        tokenProgram: anchor.utils.token.TOKEN_PROGRAM_ID,
        systemProgram: web3.SystemProgram.programId,
      } as any)
      .rpc();
    console.log("✅ Initialized SOL Market! TX:", tx);
  }

  // ────────────────────────────────────────────────
  // Save deployment info
  // ────────────────────────────────────────────────
  const deploymentConfig: DeploymentConfig = {
    protocolState: protocolState.toString(),
    usdcMint: usdcMint.toString(),
    solMint: solMint.toString(),
    usdcMarket: usdcMarket.toString(),
    solMarket: solMarket.toString(),
    programId: program.programId.toString(),
  };

  await saveConfig(deploymentConfig);

  console.log("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━");
  console.log("✅ DEPLOYMENT COMPLETE!");
  console.log("━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");
  console.log("Run: ts-node scripts/user-operations.ts\n");

  return deploymentConfig;
}

deploy()
  .then(() => process.exit(0))
  .catch((err) => {
    console.error("❌ Error:", err);
    process.exit(1);
  });
