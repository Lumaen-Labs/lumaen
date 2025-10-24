// import { describe, it, before } from "node:test";
// import * as anchor from "@coral-xyz/anchor";
// import { BN, Program } from "@coral-xyz/anchor";
// import { BankrunProvider } from "anchor-bankrun";
// import { TOKEN_PROGRAM_ID } from "@solana/spl-token";
// import { createAccount, createMint, mintTo } from "spl-token-bankrun";
// import { startAnchor, BanksClient, ProgramTestContext } from "solana-bankrun";
// import { PublicKey, Keypair } from "@solana/web3.js";
// import { readFileSync } from "fs";
// import path from "path";

// // Import the IDL
// import IDL from "../target/idl/core_router.json";
// import { CoreRouter } from "../target/types/core_router";

// describe("Core Protocol Bankrun Tests", async () => {
//   let provider: BankrunProvider;
//   let program: Program<CoreRouter>;
//   let context: ProgramTestContext;
//   let banksClient: BanksClient;
//   let signer: Keypair;
  
//   // Mints
//   let usdcMint: PublicKey;
//   let solMint: PublicKey;
  
//   // Markets
//   let usdcMarket: PublicKey;
//   let solMarket: PublicKey;
  
//   // Supply Vaults
//   let usdcSupplyVault: PublicKey;
//   let solSupplyVault: PublicKey;
  
//   // Protocol State
//   let protocolState: PublicKey;
  
//   // User accounts
//   let userUsdcAccount: PublicKey;
//   let userSolAccount: PublicKey;
//   let userUsdcPosition: PublicKey;
//   let userSolPosition: PublicKey;

//   before(async () => {
//     // Get the program ID from IDL
//     const programId = new PublicKey(IDL.address);
    
//     console.log("Starting Anchor with program:", programId.toString());
    
//     // Initialize test context
//     // Use the project root directory
//     context = await startAnchor(
//       path.join(process.cwd()),
//       [{ name: "core_router", programId }],
//       []
//     );
    
//     provider = new BankrunProvider(context);
//     anchor.setProvider(provider);
    
//     // Create program instance
//     program = new Program(IDL as anchor.Idl, provider) as unknown as Program<CoreRouter>;
    
//     banksClient = context.banksClient;
//     signer = provider.wallet.payer;

//     console.log("✅ Test environment initialized");
//     console.log("Program ID:", program.programId.toString());
//     console.log("Signer:", signer.publicKey.toString());
//   });

//   it("Setup: Create Test Mints", async () => {
//     console.log("Creating test token mints...");
    
//     // Create USDC mint (6 decimals)
//     usdcMint = await createMint(
//       banksClient,
//       signer,
//       signer.publicKey,
//       null,
//       6
//     );
//     console.log("USDC Mint:", usdcMint.toString());

//     // Create SOL mint (9 decimals)
//     solMint = await createMint(
//       banksClient,
//       signer,
//       signer.publicKey,
//       null,
//       9
//     );
//     console.log("SOL Mint:", solMint.toString());

//     // Derive Protocol State PDA
//     [protocolState] = PublicKey.findProgramAddressSync(
//       [Buffer.from("protocol_state")],
//       program.programId
//     );
//     console.log("Protocol State PDA:", protocolState.toString());

//     // Derive Market PDAs
//     [usdcMarket] = PublicKey.findProgramAddressSync(
//       [Buffer.from("market"), usdcMint.toBuffer()],
//       program.programId
//     );
//     console.log("USDC Market PDA:", usdcMarket.toString());

//     [solMarket] = PublicKey.findProgramAddressSync(
//       [Buffer.from("market"), solMint.toBuffer()],
//       program.programId
//     );
//     console.log("SOL Market PDA:", solMarket.toString());

//     console.log("✅ Test mints and PDAs created");
//   });

//   it("Initialize Protocol", async () => {
//     const feeCollector = Keypair.generate();

//     console.log("Initializing protocol...");
//     console.log("Admin:", signer.publicKey.toString());
//     console.log("Fee Collector:", feeCollector.publicKey.toString());

//     try {
//       const tx = await program.methods
//         .initializeProtocol()
//         .accounts({
//           admin: signer.publicKey,
//           feeCollector: feeCollector.publicKey,
//           protocolState: protocolState,
//           systemProgram: anchor.web3.SystemProgram.programId,
//         })
//         .rpc({ commitment: "confirmed" });

//       console.log("✅ Protocol initialized");
//       console.log("Transaction:", tx);
      
//       // Verify protocol state
//       const protocolStateAccount = await program.account.protocolState.fetch(protocolState);
//       console.log("Verified - Admin:", protocolStateAccount.admin.toString());
//       console.log("Verified - Fee Collector:", protocolStateAccount.feeCollector.toString());
//       console.log("Verified - Total Markets:", protocolStateAccount.totalMarkets.toString());
//       console.log("Verified - Protocol Paused:", protocolStateAccount.protocolPaused);
//     } catch (error) {
//       console.error("❌ Error initializing protocol:");
//       console.error(error);
//       throw error;
//     }
//   });

//   it("Initialize USDC Market", async () => {
//     // Derive supply vault PDA
//     [usdcSupplyVault] = PublicKey.findProgramAddressSync(
//       [Buffer.from("supply_vault"), usdcMarket.toBuffer()],
//       program.programId
//     );
//     console.log("USDC Supply Vault:", usdcSupplyVault.toString());

//     // Create Pyth feed ID (placeholder)
//     const pythFeedId = new Array(32).fill(0);
//     pythFeedId[0] = 1; // USDC identifier

//     const marketConfig = {
//       maxLtv: 7500, // 75%
//       liquidationThreshold: 8000, // 80%
//       liquidationPenalty: 500, // 5%
//       reserveFactor: 1000, // 10%
//       minDepositAmount: new BN(1_000_000), // 1 USDC
//       maxDepositAmount: new BN(1_000_000_000_000), // 1M USDC
//       minBorrowAmount: new BN(1_000_000), // 1 USDC
//       maxBorrowAmount: new BN(1_000_000_000_000), // 1M USDC
//       depositFee: 0,
//       withdrawFee: 0,
//       borrowFee: 0,
//       repayFee: 0,
//       pythFeedId: Buffer.from(pythFeedId),
//     };

//     console.log("Initializing USDC market with config:");
//     console.log("Max LTV:", marketConfig.maxLtv);
//     console.log("Liquidation Threshold:", marketConfig.liquidationThreshold);

//     try {
//       const tx = await program.methods
//         .initializeMarket(marketConfig)
//         .accounts({
//           owner: signer.publicKey,
//           protocolState: protocolState,
//           underlyingMint: usdcMint,
//           market: usdcMarket,
//           supplyVault: usdcSupplyVault,
//           tokenProgram: TOKEN_PROGRAM_ID,
//           systemProgram: anchor.web3.SystemProgram.programId,
//         })
//         .rpc({ commitment: "confirmed" });

//       console.log("✅ USDC Market initialized");
//       console.log("Transaction:", tx);
      
//       // Verify market state
//       const marketAccount = await program.account.market.fetch(usdcMarket);
//       console.log("Verified - Mint:", marketAccount.mint.toString());
//       console.log("Verified - Supply Vault:", marketAccount.supplyVault.toString());
//       console.log("Verified - Max LTV:", marketAccount.maxLtv.toString());
//       console.log("Verified - Total Deposits:", marketAccount.totalDeposits.toString());
//     } catch (error) {
//       console.error("❌ Error initializing USDC market:");
//       console.error(error);
//       throw error;
//     }
//   });

//   it("Initialize SOL Market", async () => {
//     [solSupplyVault] = PublicKey.findProgramAddressSync(
//       [Buffer.from("supply_vault"), solMarket.toBuffer()],
//       program.programId
//     );
//     console.log("SOL Supply Vault:", solSupplyVault.toString());

//     const pythFeedId = new Array(32).fill(0);
//     pythFeedId[0] = 2; // SOL identifier

//     const marketConfig = {
//       maxLtv: 7000, // 70%
//       liquidationThreshold: 7500, // 75%
//       liquidationPenalty: 500, // 5%
//       reserveFactor: 1000, // 10%
//       minDepositAmount: new BN(100_000_000), // 0.1 SOL
//       maxDepositAmount: new BN(1_000_000_000_000), // 1000 SOL
//       minBorrowAmount: new BN(100_000_000), // 0.1 SOL
//       maxBorrowAmount: new BN(1_000_000_000_000), // 1000 SOL
//       depositFee: 0,
//       withdrawFee: 0,
//       borrowFee: 0,
//       repayFee: 0,
//       pythFeedId: Buffer.from(pythFeedId),
//     };

//     console.log("Initializing SOL market...");

//     try {
//       const tx = await program.methods
//         .initializeMarket(marketConfig)
//         .accounts({
//           owner: signer.publicKey,
//           protocolState: protocolState,
//           underlyingMint: solMint,
//           market: solMarket,
//           supplyVault: solSupplyVault,
//           tokenProgram: TOKEN_PROGRAM_ID,
//           systemProgram: anchor.web3.SystemProgram.programId,
//         })
//         .rpc({ commitment: "confirmed" });

//       console.log("✅ SOL Market initialized");
//       console.log("Transaction:", tx);
//     } catch (error) {
//       console.error("❌ Error initializing SOL market:");
//       console.error(error);
//       throw error;
//     }
//   });

//   it("Initialize User Position for USDC", async () => {
//     // Correct PDA derivation: uses market.key() not market.mint
//     [userUsdcPosition] = PublicKey.findProgramAddressSync(
//       [Buffer.from("user_account"), signer.publicKey.toBuffer(), usdcMarket.toBuffer()],
//       program.programId
//     );
//     console.log("User USDC Position PDA:", userUsdcPosition.toString());

//     try {
//       const tx = await program.methods
//         .initializeUserPosition()
//         .accounts({
//           signer: signer.publicKey,
//           market: usdcMarket,
//           userAccount: userUsdcPosition,
//           systemProgram: anchor.web3.SystemProgram.programId,
//         })
//         .rpc({ commitment: "confirmed" });

//       console.log("✅ User USDC position initialized");
//       console.log("Transaction:", tx);
      
//       // Verify position
//       const positionAccount = await program.account.userPosition.fetch(userUsdcPosition);
//       console.log("Verified - User:", positionAccount.user.toString());
//       console.log("Verified - Market:", positionAccount.market.toString());
//       console.log("Verified - Deposited Shares:", positionAccount.depositedShares.toString());
//     } catch (error) {
//       console.error("❌ Error initializing user position:");
//       console.error(error);
//       throw error;
//     }
//   });

//   it("Create and Fund User USDC Token Account", async () => {
//     console.log("Creating user USDC token account...");

//     userUsdcAccount = await createAccount(
//       banksClient,
//       signer,
//       usdcMint,
//       signer.publicKey
//     );
//     console.log("User USDC Account:", userUsdcAccount.toBase58());

//     const fundAmount = new BN(1_000_000_000); // 1000 USDC
//     await mintTo(
//       banksClient,
//       signer,
//       usdcMint,
//       userUsdcAccount,
//       signer,
//       fundAmount.toNumber()
//     );

//     console.log("✅ User USDC account funded with:", fundAmount.toString(), "µUSDC");
//   });

//   it("Deposit USDC", async () => {
//     const depositAmount = new BN(100_000_000); // 100 USDC

//     console.log("Depositing", depositAmount.toString(), "µUSDC...");

//     try {
//       const tx = await program.methods
//         .deposit(depositAmount)
//         .accounts({
//           signer: signer.publicKey,
//           mint: usdcMint,
//           market: usdcMarket,
//           supplyVault: usdcSupplyVault,
//           userTokenAccount: userUsdcAccount,
//           userPosition: userUsdcPosition,
//           tokenProgram: TOKEN_PROGRAM_ID,
//           associatedTokenProgram: anchor.utils.token.ASSOCIATED_PROGRAM_ID,
//           systemProgram: anchor.web3.SystemProgram.programId,
//         })
//         .rpc({ commitment: "confirmed" });

//       console.log("✅ Deposit successful");
//       console.log("Transaction:", tx);
      
//       // Verify balances
//       const positionAccount = await program.account.userPosition.fetch(userUsdcPosition);
//       console.log("User deposited shares:", positionAccount.depositedShares.toString());
      
//       const marketAccount = await program.account.market.fetch(usdcMarket);
//       console.log("Market total deposits:", marketAccount.totalDeposits.toString());
//       console.log("Market total shares:", marketAccount.totalDepositedShares.toString());
//     } catch (error) {
//       console.error("❌ Error during deposit:");
//       console.error(error);
//       throw error;
//     }
//   });

//   it("Withdraw USDC", async () => {
//     // Get current position
//     const positionAccount = await program.account.userPosition.fetch(userUsdcPosition);
//     const totalShares = positionAccount.depositedShares;
//     const withdrawShares = totalShares.div(new BN(2)); // Withdraw 50%

//     console.log("Total shares:", totalShares.toString());
//     console.log("Withdrawing shares:", withdrawShares.toString());

//     try {
//       const tx = await program.methods
//         .withdraw(withdrawShares)
//         .accounts({
//           signer: signer.publicKey,
//           mint: usdcMint,
//           market: usdcMarket,
//           supplyVault: usdcSupplyVault,
//           userPosition: userUsdcPosition,
//           userTokenAccount: userUsdcAccount,
//           tokenProgram: TOKEN_PROGRAM_ID,
//           associatedTokenProgram: anchor.utils.token.ASSOCIATED_PROGRAM_ID,
//           systemProgram: anchor.web3.SystemProgram.programId,
//         })
//         .rpc({ commitment: "confirmed" });

//       console.log("✅ Withdraw successful");
//       console.log("Transaction:", tx);
      
//       // Verify balances
//       const positionAfter = await program.account.userPosition.fetch(userUsdcPosition);
//       console.log("Remaining shares:", positionAfter.depositedShares.toString());
//     } catch (error) {
//       console.error("❌ Error during withdraw:");
//       console.error(error);
//       throw error;
//     }
//   });
// });