import * as anchor from "@coral-xyz/anchor";
import { Program, BN } from "@coral-xyz/anchor";
import { createMint, createAssociatedTokenAccount, mintTo } from "@solana/spl-token";
import {
  TOKEN_PROGRAM_ID,
  ASSOCIATED_TOKEN_PROGRAM_ID,
  getOrCreateAssociatedTokenAccount,
} from "@solana/spl-token";
import * as fs from "fs";
import * as path from "path";
import * as os from "os";
import { CoreRouter } from "../target/types/core_router"; // Replace 'core_protocol' with your actual program name from Anchor.toml

async function main() {
  // Set up provider for Devnet
  const connection = new anchor.web3.Connection(anchor.web3.clusterApiUrl("devnet"), "confirmed");
  console.log("Connected to Devnet");

  // Load wallet (assuming ~/.config/solana/id.json)
  const walletPath = path.join(os.homedir(), ".config", "solana", "id.json");
  const walletKeypair = anchor.web3.Keypair.fromSecretKey(
    Uint8Array.from(JSON.parse(fs.readFileSync(walletPath, "utf-8")))
  );
  const wallet = new anchor.Wallet(walletKeypair);

  const provider = new anchor.AnchorProvider(connection, wallet, {
    commitment: "confirmed",
  });
  anchor.setProvider(provider);

  // Load program (replace with your actual IDL path and program name)
  const idl = JSON.parse(fs.readFileSync("./target/idl/core_router.json", "utf8")); // Replace with your IDL file name
  const program = new Program(idl, provider) as Program<CoreRouter>;

  // Step 1: Initialize Protocol
  const [protocolStatePda] = anchor.web3.PublicKey.findProgramAddressSync(
    [Buffer.from("protocol_state")],
    program.programId
  );

  const feeCollector = wallet.publicKey; // Use wallet as fee collector for simplicity

  try {
    await program.account.protocolState.fetch(protocolStatePda);
    console.log("⚠️  Protocol already initialized, skipping...");
  } catch (err: any) {
    if (err.name === 'AccountDoesNotExistError' || err.toString().includes('Account does not exist')) {
      try {
        await program.methods
          .initializeProtocol()
          .accounts({
            admin: wallet.publicKey,
            feeCollector: feeCollector,
            protocolState: protocolStatePda,
            systemProgram: anchor.web3.SystemProgram.programId,
          } as any)
          .rpc();
        console.log("Protocol initialized! PDA:", protocolStatePda.toBase58());
      } catch (initErr) {
        console.error("Error initializing protocol:", initErr);
        return;
      }
    } else {
      console.error("Error checking protocol state:", err);
      return;
    }
  }

  // Step 2: Create Mock Mint and Initialize Market
  const mockMint = await createMint(
    connection,
    wallet.payer,
    wallet.publicKey,
    null,
    9 // Decimals
  );
  console.log("Mock mint created:", mockMint.toBase58());

  const [marketPda] = anchor.web3.PublicKey.findProgramAddressSync(
    [Buffer.from("market"), mockMint.toBuffer()],
    program.programId
  );

  const [supplyVaultPda] = anchor.web3.PublicKey.findProgramAddressSync(
    [Buffer.from("supply_vault"), marketPda.toBuffer()],
    program.programId
  );

  const marketConfig = {
    maxLtv: new BN(50000),
    liquidationThreshold: new BN(52500),
    liquidationPenalty: new BN(500),
    reserveFactor: new BN(1000),
    minDepositAmount: new BN(100),
    maxDepositAmount: new BN(1000000),
    minBorrowAmount: new BN(100),
    maxBorrowAmount: new BN(500000),
    depositFee: new BN(10),
    withdrawFee: new BN(10),
    borrowFee: new BN(10),
    repayFee: new BN(10),
    pythFeedId: Array(32).fill(0), // Mock Pyth feed ID
  };

  try {
    await program.account.market.fetch(marketPda);
    console.log("⚠️  Market already initialized, skipping...");
  } catch (err: any) {
    if (err.name === 'AccountDoesNotExistError' || err.toString().includes('Account does not exist')) {
      try {
        await program.methods
          .initializeMarket(marketConfig)
          .accounts({
            owner: wallet.publicKey,
            protocolState: protocolStatePda,
            mint: mockMint,
            market: marketPda,
            supplyVault: supplyVaultPda,
            tokenProgram: TOKEN_PROGRAM_ID,
            systemProgram: anchor.web3.SystemProgram.programId,
          } as any)
          .rpc();
        console.log("Market initialized for mint:", mockMint.toBase58());
      } catch (initErr) {
        console.error("Error initializing market:", initErr);
        return;
      }
    } else {
      console.error("Error checking market:", err);
      return;
    }
  }

  // Step 3: Prepare for Deposit - Create User ATA and Mint Tokens
  const userTokenAccount = await createAssociatedTokenAccount(
    connection,
    wallet.payer,
    mockMint,
    wallet.publicKey
  );
  console.log("User token account:", userTokenAccount.toBase58());

  const mintAmount = new BN(10000); // Mint 10000 tokens to user
  await mintTo(
    connection,
    wallet.payer,
    mockMint,
    userTokenAccount,
    wallet.publicKey,
    mintAmount.toNumber() // Use toNumber() if BN is not directly accepted
  );
  console.log("Minted", mintAmount.toString(), "tokens to user");

  // User Position PDA (will be init_if_needed)
  const [userAccountPda] = anchor.web3.PublicKey.findProgramAddressSync(
    [Buffer.from("user_account"), wallet.publicKey.toBuffer(), marketPda.toBuffer()],
    program.programId
  );

  // Step 4: Deposit
  const depositAmount = new BN(1000);

  try {
    await program.methods
      .deposit(depositAmount)
      .accounts({
        signer: wallet.publicKey,
        mint: mockMint,
        market: marketPda,
        userTokenAccount: userTokenAccount,
        supplyVault: supplyVaultPda,
        userAccount: userAccountPda,
        tokenProgram: TOKEN_PROGRAM_ID,
        associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
        systemProgram: anchor.web3.SystemProgram.programId,
      } as any)
      .rpc();
    console.log("Deposit successful! Amount:", depositAmount.toString());
  } catch (err) {
    console.error("Error during deposit:", err);
  }
}

main().catch((err) => console.error(err));