import * as anchor from "@coral-xyz/anchor";
import { Program, BN } from "@coral-xyz/anchor";
import { createMint, createAssociatedTokenAccount, mintTo } from "@solana/spl-token";
import {
  TOKEN_PROGRAM_ID,
  ASSOCIATED_TOKEN_PROGRAM_ID,
} from "@solana/spl-token";
import * as fs from "fs";
import * as path from "path";
import * as os from "os";
import { CoreRouter } from "../target/types/core_router";
import { PythSolanaReceiver } from "@pythnetwork/pyth-solana-receiver";
import { HermesClient } from "@pythnetwork/hermes-client";
import { Connection, PublicKey } from "@solana/web3.js";

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
  const idl = JSON.parse(fs.readFileSync("./target/idl/core_router.json", "utf8"));
  const program = new Program(idl, provider) as Program<CoreRouter>;

  // Correct feed IDs (remove 0x prefix if present; use plain hex strings)
  const SOL_PRICE_FEED_ID = "ef0d8b6fda2ceba41da15d4095d1da392a0d2f8ed0c6c7bc0f4cfac8c280b56d";  // SOL/USD
  const USDC_PRICE_FEED_ID = "eaa020c61cc479712813461ce153894a96a6c00b21ed0cfc2798d1f9a9e9c94a";  // USDC/USD

  // Pyth feed IDs as Buffers
  const usdcFeedId = Buffer.from(USDC_PRICE_FEED_ID, "hex");
  const solFeedId = Buffer.from(SOL_PRICE_FEED_ID, "hex");

  // PythSolanaReceiver setup
  const pythSolanaReceiver = new PythSolanaReceiver({
    connection: connection,
    wallet: wallet,
  });

  // Derive price feed accounts
  const solUsdPriceFeedAccount = pythSolanaReceiver
    .getPriceFeedAccountAddress(0, SOL_PRICE_FEED_ID)
    .toBase58();

  const usdcUsdPriceFeedAccount = pythSolanaReceiver
    .getPriceFeedAccountAddress(0, USDC_PRICE_FEED_ID)
    .toBase58();

  console.log("SOL Price Feed Account:", solUsdPriceFeedAccount);
  console.log("USDC Price Feed Account:", usdcUsdPriceFeedAccount);

  // Fetch and post price updates
  const hermes = new HermesClient("https://hermes.pyth.network/");
  console.log("Fetching price updates from Hermes for SOL and USDC feeds...");
  const priceUpdatePayload = (
    await hermes.getLatestPriceUpdates([SOL_PRICE_FEED_ID, USDC_PRICE_FEED_ID], { encoding: "base64" })
  ).binary.data;

  const txBuilder = pythSolanaReceiver.newTransactionBuilder({ closeUpdateAccounts: false });
  await txBuilder.addPostPriceUpdates(priceUpdatePayload);

  const versionedTxs = await txBuilder.buildVersionedTransactions({
    computeUnitPriceMicroLamports: 50_000,
  });
  await pythSolanaReceiver.provider.sendAll(versionedTxs, { skipPreflight: true });
  console.log("Posted PriceUpdates to chain via PythSolanaReceiver.");

  // Verify accounts
  const solInfo = await connection.getAccountInfo(new PublicKey(solUsdPriceFeedAccount));
  console.log("SOL PriceFeed account exists:", !!solInfo);
  const usdcInfo = await connection.getAccountInfo(new PublicKey(usdcUsdPriceFeedAccount));
  console.log("USDC PriceFeed account exists:", !!usdcInfo);

  // Step 1: Initialize Protocol
  const [protocolStatePda] = anchor.web3.PublicKey.findProgramAddressSync(
    [Buffer.from("protocol_state")],
    program.programId
  );

  const feeCollector = wallet.publicKey;

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

  const protocolData = await program.account.protocolState.fetch(protocolStatePda);
  console.log("Protocol state data:", protocolData);

  // Step 2: Create Mock Mints and Initialize Markets
  const mockMintSol = await createMint(
    connection,
    wallet.payer,
    wallet.publicKey,
    null,
    9 // SOL decimals
  );
  console.log("Mock SOL mint created:", mockMintSol.toBase58());

  const mockMintUsdc = await createMint(
    connection,
    wallet.payer,
    wallet.publicKey,
    null,
    6 // USDC decimals
  );
  console.log("Mock USDC mint created:", mockMintUsdc.toBase58());

  const [marketPdaSol] = anchor.web3.PublicKey.findProgramAddressSync(
    [Buffer.from("market"), mockMintSol.toBuffer()],
    program.programId
  );

  const [marketPdaUsdc] = anchor.web3.PublicKey.findProgramAddressSync(
    [Buffer.from("market"), mockMintUsdc.toBuffer()],
    program.programId
  );

  const [supplyVaultPdaSol] = anchor.web3.PublicKey.findProgramAddressSync(
    [Buffer.from("supply_vault"), marketPdaSol.toBuffer()],
    program.programId
  );

  const [supplyVaultPdaUsdc] = anchor.web3.PublicKey.findProgramAddressSync(
    [Buffer.from("supply_vault"), marketPdaUsdc.toBuffer()],
    program.programId
  );

  const marketConfigSol = {
    maxLtv: new BN(50000),
    liquidationThreshold: new BN(52500),
    liquidationPenalty: new BN(500),
    reserveFactor: new BN(1000),
    minDepositAmount: new BN(0),
    maxDepositAmount: new BN(10000000).mul(new BN(10).pow(new BN(9))),
    minBorrowAmount: new BN(0),
    maxBorrowAmount: new BN(5000000).mul(new BN(10).pow(new BN(9))),
    depositFee: new BN(0),
    withdrawFee: new BN(0),
    borrowFee: new BN(0),
    repayFee: new BN(0),
    pythFeedId: Array.from(solFeedId),
  };

  const marketConfigUsdc = {
    maxLtv: new BN(60000),
    liquidationThreshold: new BN(62500),
    liquidationPenalty: new BN(600),
    reserveFactor: new BN(1500),
    minDepositAmount: new BN(0),
    maxDepositAmount: new BN(20000000).mul(new BN(10).pow(new BN(6))),
    minBorrowAmount: new BN(0),
    maxBorrowAmount: new BN(10000000).mul(new BN(10).pow(new BN(6))),
    depositFee: new BN(0),
    withdrawFee: new BN(0),
    borrowFee: new BN(0),
    repayFee: new BN(0),
    pythFeedId: Array.from(usdcFeedId),
  };

  // Force initialization for markets to ensure updated config
  try {
    await program.methods
      .initializeMarket(marketConfigSol)
      .accounts({
        owner: wallet.publicKey,
        protocolState: protocolStatePda,
        mint: mockMintSol,
        market: marketPdaSol,
        supplyVault: supplyVaultPdaSol,
        tokenProgram: TOKEN_PROGRAM_ID,
        systemProgram: anchor.web3.SystemProgram.programId,
      } as any)
      .rpc();
    console.log("SOL Market initialized:", mockMintSol.toBase58());
  } catch (initErr) {
    console.error("Error initializing SOL market:", initErr);
    return;
  }

  try {
    await program.methods
      .initializeMarket(marketConfigUsdc)
      .accounts({
        owner: wallet.publicKey,
        protocolState: protocolStatePda,
        mint: mockMintUsdc,
        market: marketPdaUsdc,
        supplyVault: supplyVaultPdaUsdc,
        tokenProgram: TOKEN_PROGRAM_ID,
        systemProgram: anchor.web3.SystemProgram.programId,
      } as any)
      .rpc();
    console.log("USDC Market initialized:", mockMintUsdc.toBase58());
  } catch (initErr) {
    console.error("Error initializing USDC market:", initErr);
    return;
  }

  // Step 3: Prepare for Deposits - Create User ATAs and Mint Tokens
  const userTokenAccountSol = await createAssociatedTokenAccount(
    connection,
    wallet.payer,
    mockMintSol,
    wallet.publicKey
  );
  console.log("User SOL token account:", userTokenAccountSol.toBase58());

  const mintAmountSol = new BN(20).mul(new BN(10).pow(new BN(9)));
  await mintTo(
    connection,
    wallet.payer,
    mockMintSol,
    userTokenAccountSol,
    wallet.publicKey,
    mintAmountSol.toNumber()
  );
  console.log("Minted", mintAmountSol.toString(), "SOL tokens to user");

  const userTokenAccountUsdc = await createAssociatedTokenAccount(
    connection,
    wallet.payer,
    mockMintUsdc,
    wallet.publicKey
  );
  console.log("User USDC token account:", userTokenAccountUsdc.toBase58());

  const mintAmountUsdc = new BN(1000).mul(new BN(10).pow(new BN(6)));
  await mintTo(
    connection,
    wallet.payer,
    mockMintUsdc,
    userTokenAccountUsdc,
    wallet.publicKey,
    mintAmountUsdc.toNumber()
  );
  console.log("Minted", mintAmountUsdc.toString(), "USDC tokens to user");

  // User Position PDAs
  const [userPositionPdaSol] = anchor.web3.PublicKey.findProgramAddressSync(
    [Buffer.from("user_account"), wallet.publicKey.toBuffer(), marketPdaSol.toBuffer()],
    program.programId
  );

  const [userPositionPdaUsdc] = anchor.web3.PublicKey.findProgramAddressSync(
    [Buffer.from("user_account"), wallet.publicKey.toBuffer(), marketPdaUsdc.toBuffer()],
    program.programId
  );

  // Step 4: Deposits
  const depositAmountSol = new BN(20).mul(new BN(10).pow(new BN(9)));

  try {
    await program.methods
      .deposit(depositAmountSol)
      .accounts({
        signer: wallet.publicKey,
        mint: mockMintSol,
        market: marketPdaSol,
        userTokenAccount: userTokenAccountSol,
        supplyVault: supplyVaultPdaSol,
        userAccount: userPositionPdaSol,
        tokenProgram: TOKEN_PROGRAM_ID,
        associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
        systemProgram: anchor.web3.SystemProgram.programId,
      } as any)
      .rpc();
    console.log("SOL Deposit successful! Amount:", depositAmountSol.toString());
  } catch (err) {
    console.error("Error during SOL deposit:", err);
  }

  const depositAmountUsdc = new BN(1000).mul(new BN(10).pow(new BN(6)));

  try {
    await program.methods
      .deposit(depositAmountUsdc)
      .accounts({
        signer: wallet.publicKey,
        mint: mockMintUsdc,
        market: marketPdaUsdc,
        userTokenAccount: userTokenAccountUsdc,
        supplyVault: supplyVaultPdaUsdc,
        userAccount: userPositionPdaUsdc,
        tokenProgram: TOKEN_PROGRAM_ID,
        associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
        systemProgram: anchor.web3.SystemProgram.programId,
      } as any)
      .rpc();
    console.log("USDC Deposit successful! Amount:", depositAmountUsdc.toString());
  } catch (err) {
    console.error("Error during USDC deposit:", err);
  }

  // Fetch after deposits
  const postDepositMarketSol = await program.account.market.fetch(marketPdaSol);
  console.log("Post-deposit SOL Market:", postDepositMarketSol);

  const postDepositUserSol = await program.account.userPosition.fetch(userPositionPdaSol);
  console.log("Post-deposit SOL User Position:", postDepositUserSol);

  const postDepositMarketUsdc = await program.account.market.fetch(marketPdaUsdc);
  console.log("Post-deposit USDC Market:", postDepositMarketUsdc);

  const postDepositUserUsdc = await program.account.userPosition.fetch(userPositionPdaUsdc);
  console.log("Post-deposit USDC User Position:", postDepositUserUsdc);

  // Re-fetch and post fresh price updates
  console.log("Re-fetching and posting fresh price updates...");
  const freshPriceUpdatePayload = (
    await hermes.getLatestPriceUpdates([SOL_PRICE_FEED_ID, USDC_PRICE_FEED_ID], { encoding: "base64" })
  ).binary.data;
  const freshTxBuilder = pythSolanaReceiver.newTransactionBuilder({ closeUpdateAccounts: false });
  await freshTxBuilder.addPostPriceUpdates(freshPriceUpdatePayload);
  const freshVersionedTxs = await freshTxBuilder.buildVersionedTransactions({
    computeUnitPriceMicroLamports: 50_000,
  });
  await pythSolanaReceiver.provider.sendAll(freshVersionedTxs, { skipPreflight: true });
  console.log("Posted fresh PriceUpdates.");

  // Derive Loan PDAs for each combination
  const [loanPdaSolToUsdc] = anchor.web3.PublicKey.findProgramAddressSync(
    [Buffer.from("loan"), marketPdaSol.toBuffer(), marketPdaUsdc.toBuffer(), wallet.publicKey.toBuffer()],
    program.programId
  );

  const [loanPdaSolToSol] = anchor.web3.PublicKey.findProgramAddressSync(
    [Buffer.from("loan"), marketPdaSol.toBuffer(), marketPdaSol.toBuffer(), wallet.publicKey.toBuffer()],
    program.programId
  );

  const [loanPdaUsdcToUsdc] = anchor.web3.PublicKey.findProgramAddressSync(
    [Buffer.from("loan"), marketPdaUsdc.toBuffer(), marketPdaUsdc.toBuffer(), wallet.publicKey.toBuffer()],
    program.programId
  );

  const [loanPdaUsdcToSol] = anchor.web3.PublicKey.findProgramAddressSync(
    [Buffer.from("loan"), marketPdaUsdc.toBuffer(), marketPdaSol.toBuffer(), wallet.publicKey.toBuffer()],
    program.programId
  );

  // Borrow 1: SOL collateral to USDC borrow
  const sharesSolToUsdc = new BN(1).mul(new BN(10).pow(new BN(9)));
  const borrowSolToUsdc = new BN(250).mul(new BN(10).pow(new BN(6)));

  try {
    await program.methods
      .borrow(sharesSolToUsdc, borrowSolToUsdc)
      .accounts({
        borrower: wallet.publicKey,
        collateralMint: mockMintSol,
        borrowMint: mockMintUsdc,
        protocolState: protocolStatePda,
        collateralMarket: marketPdaSol,
        borrowMarket: marketPdaUsdc,
        collateralPosition: userPositionPdaSol,
        loan: loanPdaSolToUsdc,
        tokenProgram: TOKEN_PROGRAM_ID,
        priceUpdateCol: new PublicKey(solUsdPriceFeedAccount),
        priceUpdateBorrow: new PublicKey(usdcUsdPriceFeedAccount),
        systemProgram: anchor.web3.SystemProgram.programId,
      } as any)
      .rpc();
    console.log("Borrow successful (SOL coll to USDC)! Borrowed:", borrowSolToUsdc.toString());
  } catch (err) {
    console.error("Error during borrow (SOL coll to USDC):", err);
  }

  // Borrow 2: SOL collateral to SOL borrow
  const sharesSolToSol = new BN(2).mul(new BN(10).pow(new BN(9)));
  const borrowSolToSol = new BN(4).mul(new BN(10).pow(new BN(9)));

  try {
    await program.methods
      .borrow(sharesSolToSol, borrowSolToSol)
      .accounts({
        borrower: wallet.publicKey,
        collateralMint: mockMintSol,
        borrowMint: mockMintSol,
        protocolState: protocolStatePda,
        collateralMarket: marketPdaSol,
        borrowMarket: marketPdaSol,
        collateralPosition: userPositionPdaSol,
        loan: loanPdaSolToSol,
        tokenProgram: TOKEN_PROGRAM_ID,
        priceUpdateCol: new PublicKey(solUsdPriceFeedAccount),
        priceUpdateBorrow: new PublicKey(solUsdPriceFeedAccount),
        systemProgram: anchor.web3.SystemProgram.programId,
      } as any)
      .rpc();
    console.log("Borrow successful (SOL coll to SOL)! Borrowed:", borrowSolToSol.toString());
  } catch (err) {
    console.error("Error during borrow (SOL coll to SOL):", err);
  }

  // Borrow 3: USDC collateral to USDC borrow
  const sharesUsdcToUsdc = new BN(100).mul(new BN(10).pow(new BN(6)));
  const borrowUsdcToUsdc = new BN(200).mul(new BN(10).pow(new BN(6)));

  try {
    await program.methods
      .borrow(sharesUsdcToUsdc, borrowUsdcToUsdc)
      .accounts({
        borrower: wallet.publicKey,
        collateralMint: mockMintUsdc,
        borrowMint: mockMintUsdc,
        protocolState: protocolStatePda,
        collateralMarket: marketPdaUsdc,
        borrowMarket: marketPdaUsdc,
        collateralPosition: userPositionPdaUsdc,
        loan: loanPdaUsdcToUsdc,
        tokenProgram: TOKEN_PROGRAM_ID,
        priceUpdateCol: new PublicKey(usdcUsdPriceFeedAccount),
        priceUpdateBorrow: new PublicKey(usdcUsdPriceFeedAccount),
        systemProgram: anchor.web3.SystemProgram.programId,
      } as any)
      .rpc();
    console.log("Borrow successful (USDC coll to USDC)! Borrowed:", borrowUsdcToUsdc.toString());
  } catch (err) {
    console.error("Error during borrow (USDC coll to USDC):", err);
  }

  // Borrow 4: USDC collateral to SOL borrow
  const sharesUsdcToSol = new BN(300).mul(new BN(10).pow(new BN(6)));
  const borrowUsdcToSol = new BN(1).mul(new BN(10).pow(new BN(9)));

  try {
    await program.methods
      .borrow(sharesUsdcToSol, borrowUsdcToSol)
      .accounts({
        borrower: wallet.publicKey,
        collateralMint: mockMintUsdc,
        borrowMint: mockMintSol,
        protocolState: protocolStatePda,
        collateralMarket: marketPdaUsdc,
        borrowMarket: marketPdaSol,
        collateralPosition: userPositionPdaUsdc,
        loan: loanPdaUsdcToSol,
        tokenProgram: TOKEN_PROGRAM_ID,
        priceUpdateCol: new PublicKey(usdcUsdPriceFeedAccount),
        priceUpdateBorrow: new PublicKey(solUsdPriceFeedAccount),
        systemProgram: anchor.web3.SystemProgram.programId,
      } as any)
      .rpc();
    console.log("Borrow successful (USDC coll to SOL)! Borrowed:", borrowUsdcToSol.toString());
  } catch (err) {
    console.error("Error during borrow (USDC coll to SOL):", err);
  }

  // Fetch after borrows
  try {
    const postBorrowLoanSolToUsdc = await program.account.loan.fetch(loanPdaSolToUsdc);
    console.log("Post-borrow Loan (SOL to USDC):", postBorrowLoanSolToUsdc);
  } catch (err) {
    console.error("Error fetching Loan (SOL to USDC):", err);
  }

  try {
    const postBorrowLoanSolToSol = await program.account.loan.fetch(loanPdaSolToSol);
    console.log("Post-borrow Loan (SOL to SOL):", postBorrowLoanSolToSol);
  } catch (err) {
    console.error("Error fetching Loan (SOL to SOL):", err);
  }

  try {
    const postBorrowLoanUsdcToUsdc = await program.account.loan.fetch(loanPdaUsdcToUsdc);
    console.log("Post-borrow Loan (USDC to USDC):", postBorrowLoanUsdcToUsdc);
  } catch (err) {
    console.error("Error fetching Loan (USDC to USDC):", err);
  }

  try {
    const postBorrowLoanUsdcToSol = await program.account.loan.fetch(loanPdaUsdcToSol);
    console.log("Post-borrow Loan (USDC to SOL):", postBorrowLoanUsdcToSol);
  } catch (err) {
    console.error("Error fetching Loan (USDC to SOL):", err);
  }

  const postBorrowPositionSol = await program.account.userPosition.fetch(userPositionPdaSol);
  console.log("Post-borrow SOL Position:", postBorrowPositionSol);

  const postBorrowPositionUsdc = await program.account.userPosition.fetch(userPositionPdaUsdc);
  console.log("Post-borrow USDC Position:", postBorrowPositionUsdc);

  const postBorrowMarketSol = await program.account.market.fetch(marketPdaSol);
  console.log("Post-borrow SOL Market:", postBorrowMarketSol);

  const postBorrowMarketUsdc = await program.account.market.fetch(marketPdaUsdc);
  console.log("Post-borrow USDC Market:", postBorrowMarketUsdc);

  // Print all PDAs
  console.log("\nAll PDAs:");
  console.log("Protocol State PDA:", protocolStatePda.toBase58());
  console.log("SOL Market PDA:", marketPdaSol.toBase58());
  console.log("USDC Market PDA:", marketPdaUsdc.toBase58());
  console.log("SOL Supply Vault PDA:", supplyVaultPdaSol.toBase58());
  console.log("USDC Supply Vault PDA:", supplyVaultPdaUsdc.toBase58());
  console.log("SOL User Position PDA:", userPositionPdaSol.toBase58());
  console.log("USDC User Position PDA:", userPositionPdaUsdc.toBase58());
  console.log("Loan SOL to USDC PDA:", loanPdaSolToUsdc.toBase58());
  console.log("Loan SOL to SOL PDA:", loanPdaSolToSol.toBase58());
  console.log("Loan USDC to USDC PDA:", loanPdaUsdcToUsdc.toBase58());
  console.log("Loan USDC to SOL PDA:", loanPdaUsdcToSol.toBase58());
}

main().catch((err) => console.error(err));