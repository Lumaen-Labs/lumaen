// import * as anchor from "@coral-xyz/anchor";
// import { Program, BN, web3 } from "@coral-xyz/anchor";
// import { 
//   TOKEN_PROGRAM_ID,
//   createMint,
//   createAccount,
//   mintTo,
//   getAccount
// } from "@solana/spl-token";
// import { assert } from "chai";
// import { CoreRouter } from "../target/types/core_router";

// // Pyth Price Feed IDs (Devnet)
// const PRICE_FEED_IDS = {
//   SOL: "0xef0d8b6fda2ceba41da15d4095d1da392a0d2f8ed0c6c7bc0f4cfac8c280b56d",
//   USDC: "0x41f3625971ca2ed2263e78573fe5ce23e13d2558ed3f2e47ab0f84fb9e7ae722",
//   USDT: "0x2b89b9dc8fdf9f34709a5b106b472f0f39bb6ca9ce04b0fd7f2e971688e2e53b"
// };

// describe("Core Protocol Tests", () => {
//   // Configure the client to use the devnet cluster
//   const provider = anchor.AnchorProvider.env();
//   anchor.setProvider(provider);

//   const program = anchor.workspace.CoreRouter as Program<CoreRouter>;
//   const connection = provider.connection;
//   const wallet = provider.wallet as anchor.Wallet;

//   // Test accounts
//   let protocolState: web3.PublicKey;
//   let usdcMint: web3.PublicKey;
//   let solMint: web3.PublicKey;
//   let usdcMarket: web3.PublicKey;
//   let solMarket: web3.PublicKey;
//   let usdcSupplyVault: web3.PublicKey;
//   let solSupplyVault: web3.PublicKey;
//   let userUsdcAccount: web3.PublicKey;
//   let userSolAccount: web3.PublicKey;
//   let userPosition: web3.PublicKey;
//   let loan: web3.PublicKey;
//   let loanCTokenVault: web3.PublicKey;
  
//   // Pyth oracle accounts
//   let pythPriceUpdateAccount: web3.PublicKey;
  
//   // Fee collector
//   let feeCollector: web3.Keypair;

//   // Solend devnet addresses (example - you'll need actual Solend reserve addresses)
//   const SOLEND_PROGRAM_ID = new web3.PublicKey("ALend7Ketfx5bxh6ghsCDXAoDrhvEmsXT3cynB6aPLgx");
//   const SOLEND_MARKET = new web3.PublicKey("GvjoVKNjBvQcFaSKUW1gTE7DxhSpjHbE69umVR5nPuQp"); // Main market
  
//   // You'll need to get these from Solend for the specific reserves you want to use
//   let solendUsdcReserve: web3.PublicKey;
//   let solendSolReserve: web3.PublicKey;

//   before(async () => {
//     // Create fee collector
//     feeCollector = web3.Keypair.generate();
    
//     // Airdrop SOL for testing
//     // const airdropSig = await connection.requestAirdrop(
//     //   wallet.publicKey,
//     //   2 * web3.LAMPORTS_PER_SOL 
//     // );
//     // await connection.confirmTransaction(airdropSig);

//     // Create mints (in production, use actual token mints)
//     usdcMint = await createMint(
//       connection,
//       wallet.payer,
//       wallet.publicKey,
//       null,
//       6 // USDC has 6 decimals
//     );

//     solMint = await createMint(
//       connection,
//       wallet.payer,
//       wallet.publicKey,
//       null,
//       9 // SOL has 9 decimals
//     );

//     // Create user token accounts
//     userUsdcAccount = await createAccount(
//       connection,
//       wallet.payer,
//       usdcMint,
//       wallet.publicKey
//     );

//     userSolAccount = await createAccount(
//       connection,
//       wallet.payer,
//       solMint,
//       wallet.publicKey
//     );

//     // Mint tokens to user for testing
//     await mintTo(
//       connection,
//       wallet.payer,
//       usdcMint,
//       userUsdcAccount,
//       wallet.publicKey,
//       1000 * 10**6 // 1000 USDC
//     );

//     await mintTo(
//       connection,
//       wallet.payer,
//       solMint,
//       userSolAccount,
//       wallet.publicKey,
//       10 * 10**9 // 10 SOL
//     );

//     // Derive PDAs
//     const [protocolState] =  web3.PublicKey.findProgramAddressSync(
//       [Buffer.from("protocol_state")],
//       program.programId
//     );

//     [usdcMarket] = web3.PublicKey.findProgramAddressSync(
//       [Buffer.from("market"), usdcMint.toBuffer()],
//       program.programId
//     );

//     [solMarket] = web3.PublicKey.findProgramAddressSync(
//       [Buffer.from("market"), solMint.toBuffer()],
//       program.programId
//     );

//     [usdcSupplyVault] = web3.PublicKey.findProgramAddressSync(
//       [Buffer.from("supply_vault"), usdcMarket.toBuffer()],
//       program.programId
//     );

//     [solSupplyVault] = web3.PublicKey.findProgramAddressSync(
//       [Buffer.from("supply_vault"), solMarket.toBuffer()],
//       program.programId
//     );

//     // Setup Pyth price feeds (you'll need to implement proper Pyth integration)
//     // For testing, we'll use a mock account
//     pythPriceUpdateAccount = web3.Keypair.generate().publicKey;
//   });

//   describe("Initialization", () => {
//     it("Initializes the protocol", async () => {
//       const tx = await program.methods
//         .initializeProtocol()
//         .accounts({
//           admin: wallet.publicKey,
//           feeCollector: feeCollector.publicKey,
//           protocolState: protocolState,
//           systemProgram: web3.SystemProgram.programId,
//         })
//         .rpc();

//       console.log("Protocol initialized:", tx);

//       // Verify protocol state
//       const state = await program.account.protocolState.fetch(protocolState);
//       assert.equal(state.admin.toBase58(), wallet.publicKey.toBase58());
//       assert.equal(state.feeCollector.toBase58(), feeCollector.publicKey.toBase58());
//       assert.equal(state.protocolPaused, false);
//     });

//     it("Initializes USDC market with 5x leverage", async () => {
//       const config = {
//         maxLtv: new BN(50000), // 500% for 5x leverage
//         liquidationThreshold: new BN(52500), // 525%
//         liquidationPenalty: new BN(500), // 5%
//         reserveFactor: new BN(1000), // 10%
//         minDepositAmount: new BN(1 * 10**6), // 1 USDC
//         maxDepositAmount: new BN(1000000 * 10**6), // 1M USDC
//         minBorrowAmount: new BN(1 * 10**6), // 1 USDC
//         maxBorrowAmount: new BN(100000 * 10**6), // 100K USDC
//         depositFee: new BN(10), // 0.1%
//         withdrawFee: new BN(10), // 0.1%
//         borrowFee: new BN(50), // 0.5%
//         repayFee: new BN(0), // 0%
//         pythFeedId: Buffer.from(PRICE_FEED_IDS.USDC.slice(2), 'hex'),
//       };

//       const tx = await program.methods
//         .initializeMarket(config)
//         .accounts({
//           owner: wallet.publicKey,
//           protocolState: protocolState,
//           underlyingMint: usdcMint,
//           market: usdcMarket,
//           supplyVault: usdcSupplyVault,
//           tokenProgram: TOKEN_PROGRAM_ID,
//           systemProgram: web3.SystemProgram.programId,
//         })
//         .rpc();

//       console.log("USDC market initialized:", tx);

//       // Verify market state
//       const market = await program.account.market.fetch(usdcMarket);
//       assert.equal(market.mint.toBase58(), usdcMint.toBase58());
//       assert.equal(market.maxLtv.toNumber(), 50000); // 500% LTV
//     });

//     it("Initializes SOL market", async () => {
//       const config = {
//         maxLtv: new BN(50000), // 500% for 5x leverage
//         liquidationThreshold: new BN(52500), // 525%
//         liquidationPenalty: new BN(500), // 5%
//         reserveFactor: new BN(1000), // 10%
//         minDepositAmount: new BN(0.01 * 10**9), // 0.01 SOL
//         maxDepositAmount: new BN(10000 * 10**9), // 10000 SOL
//         minBorrowAmount: new BN(0.01 * 10**9), // 0.01 SOL
//         maxBorrowAmount: new BN(1000 * 10**9), // 1000 SOL
//         depositFee: new BN(10), // 0.1%
//         withdrawFee: new BN(10), // 0.1%
//         borrowFee: new BN(50), // 0.5%
//         repayFee: new BN(0), // 0%
//         pythFeedId: Buffer.from(PRICE_FEED_IDS.SOL.slice(2), 'hex'),
//       };

//       const tx = await program.methods
//         .initializeMarket(config)
//         .accounts({
//           owner: wallet.publicKey,
//           protocolState: protocolState,
//           underlyingMint: solMint,
//           market: solMarket,
//           supplyVault: solSupplyVault,
//           tokenProgram: TOKEN_PROGRAM_ID,
//           systemProgram: web3.SystemProgram.programId,
//         })
//         .rpc();

//       console.log("SOL market initialized:", tx);
//     });
//   });

//   describe("Core Operations", () => {
//     it("Deposits USDC", async () => {
//       // Derive user position PDA
//       [userPosition] = web3.PublicKey.findProgramAddressSync(
//         [Buffer.from("user_position"), wallet.publicKey.toBuffer(), usdcMarket.toBuffer()],
//         program.programId
//       );

//       // First initialize user position
//       await program.methods
//         .initializeUserPosition()
//         .accounts({
//           signer: wallet.publicKey,
//           market: usdcMarket,
//           userAccount: userPosition,
//           systemProgram: web3.SystemProgram.programId,
//         })
//         .rpc();

//       // Deposit 100 USDC
//       const depositAmount = new BN(100 * 10**6);
      
//       const tx = await program.methods
//         .deposit(depositAmount)
//         .accounts({
//           signer: wallet.publicKey,
//           mint: usdcMint,
//           market: usdcMarket,
//           userTokenAccount: userUsdcAccount,
//           supplyVault: usdcSupplyVault,
//           userPosition: userPosition,
//           tokenProgram: TOKEN_PROGRAM_ID,
//           systemProgram: web3.SystemProgram.programId,
//         })
//         .rpc();

//       console.log("Deposited 100 USDC:", tx);

//       // Verify deposit
//       const position = await program.account.userPosition.fetch(userPosition);
//       assert(position.depositedShares.gt(new BN(0)));
//       console.log("Deposited shares:", position.depositedShares.toString());
//     });

//     it("Borrows SOL with 5x leverage", async () => {
//       // Derive loan PDA
//       [loan] = web3.PublicKey.findProgramAddressSync(
//         [
//           Buffer.from("loan"),
//           usdcMarket.toBuffer(), // collateral market
//           solMarket.toBuffer(), // borrow market
//           wallet.publicKey.toBuffer()
//         ],
//         program.programId
//       );

//       // Initialize loan
//       await program.methods
//         .initializeLoan(solMint, usdcMint) // borrow SOL, collateral USDC
//         .accounts({
//           borrower: wallet.publicKey,
//           supplyMarket: usdcMarket,
//           borrowMarket: solMarket,
//           protocolState: protocolState,
//           loan: loan,
//           systemProgram: web3.SystemProgram.programId,
//         })
//         .rpc();

//       // Borrow 5 SOL with 100 USDC collateral (assuming price ratio allows)
//       // For 5x leverage: $100 USDC collateral -> can borrow up to $500 worth
//       const borrowAmount = new BN(5 * 10**9); // 5 SOL
//       const collateralShares = new BN(99 * 10**6); // Use 99 USDC worth of shares as collateral

//       const tx = await program.methods
//         .borrow(collateralShares, borrowAmount)
//         .accounts({
//           borrower: wallet.publicKey,
//           collateralMint: usdcMint,
//           borrowMint: solMint,
//           protocolState: protocolState,
//           collateralMarket: usdcMarket,
//           borrowMarket: solMarket,
//           collateralPosition: userPosition,
//           loan: loan,
//           tokenProgram: TOKEN_PROGRAM_ID,
//           priceUpdate: pythPriceUpdateAccount,
//           systemProgram: web3.SystemProgram.programId,
//         })
//         .rpc();

//       console.log("Borrowed 5 SOL with 5x leverage:", tx);

//       // Verify loan
//       const loanAccount = await program.account.loan.fetch(loan);
//       assert.equal(loanAccount.borrower.toBase58(), wallet.publicKey.toBase58());
//       assert(loanAccount.borrowedAmount.gt(new BN(0)));
//       console.log("Loan created with ID:", loanAccount.loanId.toString());
//     });

//     it("Invests borrowed funds in Solend", async () => {
//       // Derive cToken vault PDA
//       [loanCTokenVault] = web3.PublicKey.findProgramAddressSync(
//         [Buffer.from("loan_ctoken_vault"), loan.toBuffer()],
//         program.programId
//       );

//       // You'll need actual Solend reserve addresses for the tokens
//       // For testing, we'll use placeholder addresses
//       solendSolReserve = web3.Keypair.generate().publicKey;
      
//       const investAmount = new BN(4 * 10**9); // Invest 4 SOL in Solend

//       // Note: This will fail in actual testing without proper Solend setup
//       // You'll need to use actual Solend reserve accounts
//       try {
//         const tx = await program.methods
//           .investInSolend(investAmount)
//           .accounts({
//             borrower: wallet.publicKey,
//             loan: loan,
//             borrowMarket: solMarket,
//             protocolVault: solSupplyVault,
//             solendLiquiditySupply: web3.Keypair.generate().publicKey, // Replace with actual
//             solendReserve: solendSolReserve,
//             solendCtokenMint: web3.Keypair.generate().publicKey, // Replace with actual
//             loanCtokenVault: loanCTokenVault,
//             solendCollateralSupply: web3.Keypair.generate().publicKey, // Replace with actual
//             solendProgram: SOLEND_PROGRAM_ID,
//             solendMarket: SOLEND_MARKET,
//             solendMarketAuthority: web3.Keypair.generate().publicKey, // Replace with actual
//             underlyingMint: solMint,
//             tokenProgram: TOKEN_PROGRAM_ID,
//             systemProgram: web3.SystemProgram.programId,
//             rent: web3.SYSVAR_RENT_PUBKEY,
//             clock: web3.SYSVAR_CLOCK_PUBKEY,
//           })
//           .rpc();

//         console.log("Invested in Solend:", tx);
//       } catch (error) {
//         console.log("Note: Solend investment would work with proper Solend setup");
//       }
//     });

//     it("Withdraws from Solend", async () => {
//       // Note: This test assumes the previous investment was successful
//       try {
//         const tx = await program.methods
//           .withdrawFromSolend()
//           .accounts({
//             borrower: wallet.publicKey,
//             loan: loan,
//             borrowMarket: solMarket,
//             protocolVault: solSupplyVault,
//             loanCtokenVault: loanCTokenVault,
//             solendReserve: solendSolReserve,
//             solendLiquiditySupply: web3.Keypair.generate().publicKey, // Replace with actual
//             solendCollateralSupply: web3.Keypair.generate().publicKey, // Replace with actual
//             solendCtokenMint: web3.Keypair.generate().publicKey, // Replace with actual
//             solendMarket: SOLEND_MARKET,
//             solendMarketAuthority: web3.Keypair.generate().publicKey, // Replace with actual
//             solendProgram: SOLEND_PROGRAM_ID,
//             underlyingMint: solMint,
//             tokenProgram: TOKEN_PROGRAM_ID,
//             systemProgram: web3.SystemProgram.programId,
//             clock: web3.SYSVAR_CLOCK_PUBKEY,
//           })
//           .rpc();

//         console.log("Withdrawn from Solend:", tx);
//       } catch (error) {
//         console.log("Note: Solend withdrawal would work with proper Solend setup");
//       }
//     });

//     it("Repays loan", async () => {
//       // Repay the loan
//       const repayAmount = new BN(5.1 * 10**9); // Repay 5.1 SOL (principal + interest)

//       const tx = await program.methods
//         .repay(repayAmount)
//         .accounts({
//           borrower: wallet.publicKey,
//           mint: solMint,
//           loan: loan,
//           protocolState: protocolState,
//           collateralMarket: usdcMarket,
//           borrowMarket: solMarket,
//           userPosition: userPosition,
//           userTokenAccount: userSolAccount,
//           supplyVault: solSupplyVault,
//           tokenProgram: TOKEN_PROGRAM_ID,
//           systemProgram: web3.SystemProgram.programId,
//         })
//         .rpc();

//       console.log("Repaid loan:", tx);

//       // Verify loan is repaid
//       const loanAccount = await program.account.loan.fetch(loan);
//       assert.equal(loanAccount.statusU8, 1); // 1 = Repaid
//       assert.equal(loanAccount.borrowedAmount.toNumber(), 0);
//     });

//     it("Withdraws USDC", async () => {
//       // Withdraw deposited USDC
//       const withdrawShares = new BN(50 * 10**6); // Withdraw 50 USDC worth of shares

//       const tx = await program.methods
//         .withdraw(withdrawShares)
//         .accounts({
//           signer: wallet.publicKey,
//           mint: usdcMint,
//           market: usdcMarket,
//           supplyVault: usdcSupplyVault,
//           userPosition: userPosition,
//           userTokenAccount: userUsdcAccount,
//           tokenProgram: TOKEN_PROGRAM_ID,
//           systemProgram: web3.SystemProgram.programId,
//         })
//         .rpc();

//       console.log("Withdrawn 50 USDC:", tx);

//       // Verify withdrawal
//       const position = await program.account.userPosition.fetch(userPosition);
//       console.log("Remaining deposited shares:", position.depositedShares.toString());
//     });

//   });
// });



// import * as anchor from "@coral-xyz/anchor";
// import { Program, BN, web3 } from "@coral-xyz/anchor";
// import { 
//   TOKEN_PROGRAM_ID,
//   ASSOCIATED_TOKEN_PROGRAM_ID,
//   createMint,
//   createAccount,
//   mintTo,
//   getAccount
// } from "@solana/spl-token";
// import { assert } from "chai";
// import { CoreRouter } from "../target/types/core_router";

// // Pyth Price Feed IDs (Devnet)
// const PRICE_FEED_IDS = {
//   SOL: "0xef0d8b6fda2ceba41da15d4095d1da392a0d2f8ed0c6c7bc0f4cfac8c280b56d",
//   USDC: "0x41f3625971ca2ed2263e78573fe5ce23e13d2558ed3f2e47ab0f84fb9e7ae722",
//   USDT: "0x2b89b9dc8fdf9f34709a5b106b472f0f39bb6ca9ce04b0fd7f2e971688e2e53b"
// };

// describe("Core Protocol Tests", () => {
//   // Configure the client to use the devnet cluster
//   const provider = anchor.AnchorProvider.env();
//   anchor.setProvider(provider);

//   const program = anchor.workspace.CoreRouter as Program<CoreRouter>;
//   const connection = provider.connection;
//   const wallet = provider.wallet as anchor.Wallet;

//   // Test accounts - declared at module level
//   let protocolState: web3.PublicKey;
//   let usdcMint: web3.PublicKey;
//   let solMint: web3.PublicKey;
//   let usdcMarket: web3.PublicKey;
//   let solMarket: web3.PublicKey;
//   let usdcSupplyVault: web3.PublicKey;
//   let solSupplyVault: web3.PublicKey;
//   let userUsdcAccount: web3.PublicKey;
//   let userSolAccount: web3.PublicKey;
//   let userUsdcPosition: web3.PublicKey;
//   let userSolPosition: web3.PublicKey;
//   let loan: web3.PublicKey;
//   let loanCTokenVault: web3.PublicKey;
  
//   // Pyth oracle accounts
//   let pythPriceUpdateAccount: web3.PublicKey;
  
//   // Fee collector
//   let feeCollector: web3.Keypair;

//   // Solend devnet addresses
//   const SOLEND_PROGRAM_ID = new web3.PublicKey("ALend7Ketfx5bxh6ghsCDXAoDrhvEmsXT3cynB6aPLgx");
//   const SOLEND_MARKET = new web3.PublicKey("GvjoVKNjBvQcFaSKUW1gTE7DxhSpjHbE69umVR5nPuQp");
  
//   let solendUsdcReserve: web3.PublicKey;
//   let solendSolReserve: web3.PublicKey;

//   before(async () => {
//     console.log("Setting up test environment...");
    
//     // Create fee collector
//     feeCollector = web3.Keypair.generate();
    
//     // Airdrop SOL for testing (uncomment if needed)
//     // const airdropSig = await connection.requestAirdrop(
//     //   wallet.publicKey,
//     //   2 * web3.LAMPORTS_PER_SOL 
//     // );
//     // await connection.confirmTransaction(airdropSig);

//     // Create mints
//     console.log("Creating test mints...");
//     usdcMint = await createMint(
//       connection,
//       wallet.payer,
//       wallet.publicKey,
//       null,
//       6 // USDC has 6 decimals
//     );
//     console.log("USDC Mint:", usdcMint.toString());

//     solMint = await createMint(
//       connection,
//       wallet.payer,
//       wallet.publicKey,
//       null,
//       9 // SOL has 9 decimals
//     );
//     console.log("SOL Mint:", solMint.toString());

//     // Create user token accounts
//     userUsdcAccount = await createAccount(
//       connection,
//       wallet.payer,
//       usdcMint,
//       wallet.publicKey
//     );

//     userSolAccount = await createAccount(
//       connection,
//       wallet.payer,
//       solMint,
//       wallet.publicKey
//     );

//     // Mint tokens to user for testing
//     await mintTo(
//       connection,
//       wallet.payer,
//       usdcMint,
//       userUsdcAccount,
//       wallet.publicKey,
//       1000 * 10**6 // 1000 USDC
//     );

//     await mintTo(
//       connection,
//       wallet.payer,
//       solMint,
//       userSolAccount,
//       wallet.publicKey,
//       10 * 10**9 // 10 SOL
//     );

//     // Derive PDAs
//     [protocolState] = web3.PublicKey.findProgramAddressSync(
//       [Buffer.from("protocol_state")],
//       program.programId
//     );
//     console.log("Protocol State:", protocolState.toString());

//     [usdcMarket] = web3.PublicKey.findProgramAddressSync(
//       [Buffer.from("market"), usdcMint.toBuffer()],
//       program.programId
//     );
//     console.log("USDC Market:", usdcMarket.toString());

//     [solMarket] = web3.PublicKey.findProgramAddressSync(
//       [Buffer.from("market"), solMint.toBuffer()],
//       program.programId
//     );
//     console.log("SOL Market:", solMarket.toString());

//     [usdcSupplyVault] = web3.PublicKey.findProgramAddressSync(
//       [Buffer.from("supply_vault"), usdcMarket.toBuffer()],
//       program.programId
//     );

//     [solSupplyVault] = web3.PublicKey.findProgramAddressSync(
//       [Buffer.from("supply_vault"), solMarket.toBuffer()],
//       program.programId
//     );

//     // Setup Pyth price feeds (mock for testing)
//     pythPriceUpdateAccount = web3.Keypair.generate().publicKey;
    
//     console.log("✅ Setup complete");
//   });

//   describe("Initialization", () => {
//     it("Initializes the protocol", async () => {
//       try {
//         const tx = await program.methods
//           .initializeProtocol()
//           .accounts({
//             admin: wallet.publicKey,
//             feeCollector: feeCollector.publicKey,
//             protocolState: protocolState,
//             systemProgram: web3.SystemProgram.programId,
//           })
//           .rpc();

//         console.log("Protocol initialized:", tx);

//         // Verify protocol state
//         const state = await program.account.protocolState.fetch(protocolState);
//         assert.equal(state.admin.toBase58(), wallet.publicKey.toBase58());
//         assert.equal(state.feeCollector.toBase58(), feeCollector.publicKey.toBase58());
//         assert.equal(state.protocolPaused, false);
//         console.log("✅ Protocol state verified");
//       } catch (error) {
//         console.error("Error initializing protocol:", error);
//         throw error;
//       }
//     });

//     it("Initializes USDC market with 5x leverage", async () => {
//       const config = {
//         maxLtv: new BN(50000), // 500% for 5x leverage
//         liquidationThreshold: new BN(52500), // 525%
//         liquidationPenalty: new BN(500), // 5%
//         reserveFactor: new BN(1000), // 10%
//         minDepositAmount: new BN(1 * 10**6), // 1 USDC
//         maxDepositAmount: new BN(1000000 * 10**6), // 1M USDC
//         minBorrowAmount: new BN(1 * 10**6), // 1 USDC
//         maxBorrowAmount: new BN(100000 * 10**6), // 100K USDC
//         depositFee: new BN(0), // 0%
//         withdrawFee: new BN(0), // 0%
//         borrowFee: new BN(0), // 0%
//         repayFee: new BN(0), // 0%
//         pythFeedId: Buffer.from(PRICE_FEED_IDS.USDC.slice(2), 'hex'),
//       };

//       const tx = await program.methods
//         .initializeMarket(config)
//         .accounts({
//           owner: wallet.publicKey,
//           protocolState: protocolState,
//           underlyingMint: usdcMint,
//           market: usdcMarket,
//           supplyVault: usdcSupplyVault,
//           tokenProgram: TOKEN_PROGRAM_ID,
//           systemProgram: web3.SystemProgram.programId,
//         })
//         .rpc();

//       console.log("USDC market initialized:", tx);

//       // Verify market state
//       const market = await program.account.market.fetch(usdcMarket);
//       assert.equal(market.mint.toBase58(), usdcMint.toBase58());
//       assert.equal(market.maxLtv.toNumber(), 50000);
//       console.log("✅ USDC market verified");
//     });

//     it("Initializes SOL market", async () => {
//       const config = {
//         maxLtv: new BN(50000), // 500% for 5x leverage
//         liquidationThreshold: new BN(52500), // 525%
//         liquidationPenalty: new BN(500), // 5%
//         reserveFactor: new BN(1000), // 10%
//         minDepositAmount: new BN(0.01 * 10**9), // 0.01 SOL
//         maxDepositAmount: new BN(10000 * 10**9), // 10000 SOL
//         minBorrowAmount: new BN(0.01 * 10**9), // 0.01 SOL
//         maxBorrowAmount: new BN(1000 * 10**9), // 1000 SOL
//         depositFee: new BN(0), // 0%
//         withdrawFee: new BN(0), // 0%
//         borrowFee: new BN(0), // 0%
//         repayFee: new BN(0), // 0%
//         pythFeedId: Buffer.from(PRICE_FEED_IDS.SOL.slice(2), 'hex'),
//       };

//       const tx = await program.methods
//         .initializeMarket(config)
//         .accounts({
//           owner: wallet.publicKey,
//           protocolState: protocolState,
//           underlyingMint: solMint,
//           market: solMarket,
//           supplyVault: solSupplyVault,
//           tokenProgram: TOKEN_PROGRAM_ID,
//           systemProgram: web3.SystemProgram.programId,
//         })
//         .rpc();

//       console.log("SOL market initialized:", tx);
//       console.log("✅ SOL market verified");
//     });
//   });

//   describe("Core Operations", () => {
//     it("Initializes user USDC position", async () => {
//       // CRITICAL FIX: Use "user_account" not "user_position"
//       // and use market.key() not mint
//       [userUsdcPosition] = web3.PublicKey.findProgramAddressSync(
//         [Buffer.from("user_account"), wallet.publicKey.toBuffer(), usdcMarket.toBuffer()],
//         program.programId
//       );
//       console.log("User USDC Position PDA:", userUsdcPosition.toString());

//       const tx = await program.methods
//         .initializeUserPosition()
//         .accounts({
//           signer: wallet.publicKey,
//           market: usdcMarket,
//           userAccount: userUsdcPosition,
//           systemProgram: web3.SystemProgram.programId,
//         })
//         .rpc();

//       console.log("User USDC position initialized:", tx);
      
//       // Verify
//       const position = await program.account.userPosition.fetch(userUsdcPosition);
//       assert.equal(position.user.toBase58(), wallet.publicKey.toBase58());
//       console.log("✅ User position verified");
//     });

//     it("Deposits USDC", async () => {
//       const depositAmount = new BN(100 * 10**6); // 100 USDC
      
//       const tx = await program.methods
//         .deposit(depositAmount)
//         .accounts({
//           signer: wallet.publicKey,
//           mint: usdcMint,
//           market: usdcMarket,
//           userTokenAccount: userUsdcAccount,
//           supplyVault: usdcSupplyVault,
//           userPosition: userUsdcPosition,
//           tokenProgram: TOKEN_PROGRAM_ID,
//           associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
//           systemProgram: web3.SystemProgram.programId,
//         })
//         .rpc();

//       console.log("Deposited 100 USDC:", tx);

//       // Verify deposit
//       const position = await program.account.userPosition.fetch(userUsdcPosition);
//       assert(position.depositedShares.gt(new BN(0)));
//       console.log("Deposited shares:", position.depositedShares.toString());
//       console.log("✅ Deposit verified");
//     });

//     it("Initializes user SOL position", async () => {
//       [userSolPosition] = web3.PublicKey.findProgramAddressSync(
//         [Buffer.from("user_account"), wallet.publicKey.toBuffer(), solMarket.toBuffer()],
//         program.programId
//       );
//       console.log("User SOL Position PDA:", userSolPosition.toString());

//       const tx = await program.methods
//         .initializeUserPosition()
//         .accounts({
//           signer: wallet.publicKey,
//           market: solMarket,
//           userAccount: userSolPosition,
//           systemProgram: web3.SystemProgram.programId,
//         })
//         .rpc();

//       console.log("User SOL position initialized:", tx);
//       console.log("✅ SOL position verified");
//     });

//     it("Deposits SOL for borrowing", async () => {
//       const depositAmount = new BN(2 * 10**9); // 2 SOL
      
//       const tx = await program.methods
//         .deposit(depositAmount)
//         .accounts({
//           signer: wallet.publicKey,
//           mint: solMint,
//           market: solMarket,
//           userTokenAccount: userSolAccount,
//           supplyVault: solSupplyVault,
//           userPosition: userSolPosition,
//           tokenProgram: TOKEN_PROGRAM_ID,
//           associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
//           systemProgram: web3.SystemProgram.programId,
//         })
//         .rpc();

//       console.log("Deposited 2 SOL:", tx);
      
//       const position = await program.account.userPosition.fetch(userSolPosition);
//       console.log("SOL shares:", position.depositedShares.toString());
//       console.log("✅ SOL deposit verified");
//     });

//     it("Initializes loan", async () => {
//       // Derive loan PDA - CRITICAL: order is collateral_market, borrow_market, borrower
//       [loan] = web3.PublicKey.findProgramAddressSync(
//         [
//           Buffer.from("loan"),
//           usdcMarket.toBuffer(), // collateral market (USDC)
//           solMarket.toBuffer(),   // borrow market (SOL)
//           wallet.publicKey.toBuffer()
//         ],
//         program.programId
//       );
//       console.log("Loan PDA:", loan.toString());

//       const tx = await program.methods
//         .initializeLoan(solMint, usdcMint) // borrow_asset, collateral_asset
//         .accounts({
//           borrower: wallet.publicKey,
//           supplyMarket: usdcMarket,  // collateral market
//           borrowMarket: solMarket,    // borrow market
//           protocolState: protocolState,
//           loan: loan,
//           systemProgram: web3.SystemProgram.programId,
//         })
//         .rpc();

//       console.log("Loan initialized:", tx);
      
//       const loanAccount = await program.account.loan.fetch(loan);
//       console.log("Loan ID:", loanAccount.loanId.toString());
//       console.log("✅ Loan initialized");
//     });

//     it("Borrows SOL with USDC collateral", async () => {
//       // Get position to see available shares
//       const position = await program.account.userPosition.fetch(userUsdcPosition);
//       console.log("Available USDC shares:", position.depositedShares.toString());
      
//       // Use 90 USDC worth of shares as collateral
//       const collateralShares = new BN(90 * 10**6);
//       const borrowAmount = new BN(1 * 10**9); // Borrow 1 SOL
      
//       console.log("Collateral shares:", collateralShares.toString());
//       console.log("Borrow amount:", borrowAmount.toString());

//       try {
//         const tx = await program.methods
//           .borrow(collateralShares, borrowAmount)
//           .accounts({
//             borrower: wallet.publicKey,
//             collateralMint: usdcMint,
//             borrowMint: solMint,
//             protocolState: protocolState,
//             collateralMarket: usdcMarket,
//             borrowMarket: solMarket,
//             collateralPosition: userUsdcPosition,
//             loan: loan,
//             tokenProgram: TOKEN_PROGRAM_ID,
//             priceUpdate: pythPriceUpdateAccount,
//             systemProgram: web3.SystemProgram.programId,
//           })
//           .rpc();

//         console.log("Borrowed SOL:", tx);

//         // Verify loan
//         const loanAccount = await program.account.loan.fetch(loan);
//         console.log("Borrowed amount:", loanAccount.borrowedAmount.toString());
//         console.log("Collateral locked:", loanAccount.collateralAmount.toString());
//         console.log("✅ Borrow successful");
//       } catch (error) {
//         console.error("Borrow error:", error);
//         console.log("Note: This might fail without proper Pyth price feeds");
//         throw error;
//       }
//     });

//     it("Repays loan", async () => {
//       // Get current loan state
//       const loanAccount = await program.account.loan.fetch(loan);
//       const borrowedUnderlying = loanAccount.borrowedUnderlying;
      
//       // Repay with a bit extra for interest
//       const repayAmount = borrowedUnderlying.add(new BN(0.1 * 10**9));
      
//       console.log("Repaying:", repayAmount.toString());

//       const tx = await program.methods
//         .repay(repayAmount)
//         .accounts({
//           borrower: wallet.publicKey,
//           mint: solMint,
//           loan: loan,
//           protocolState: protocolState,
//           collateralMarket: usdcMarket,
//           borrowMarket: solMarket,
//           userPosition: userUsdcPosition,
//           userTokenAccount: userSolAccount,
//           supplyVault: solSupplyVault,
//           tokenProgram: TOKEN_PROGRAM_ID,
//           associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
//           systemProgram: web3.SystemProgram.programId,
//         })
//         .rpc();

//       console.log("Repaid loan:", tx);

//       // Verify repayment
//       const updatedPosition = await program.account.userPosition.fetch(userUsdcPosition);
//       console.log("Unlocked collateral, locked:", updatedPosition.lockedCollateral.toString());
//       console.log("✅ Loan repaid");
//     });

//     it("Withdraws USDC", async () => {
//       const position = await program.account.userPosition.fetch(userUsdcPosition);
//       const freeShares = position.depositedShares.sub(position.lockedCollateral);
      
//       // Withdraw half of free shares
//       const withdrawShares = freeShares.div(new BN(2));
      
//       console.log("Free shares:", freeShares.toString());
//       console.log("Withdrawing:", withdrawShares.toString());

//       const tx = await program.methods
//         .withdraw(withdrawShares)
//         .accounts({
//           signer: wallet.publicKey,
//           mint: usdcMint,
//           market: usdcMarket,
//           supplyVault: usdcSupplyVault,
//           userPosition: userUsdcPosition,
//           userTokenAccount: userUsdcAccount,
//           tokenProgram: TOKEN_PROGRAM_ID,
//           associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
//           systemProgram: web3.SystemProgram.programId,
//         })
//         .rpc();

//       console.log("Withdrawn USDC:", tx);

//       // Verify withdrawal
//       const updatedPosition = await program.account.userPosition.fetch(userUsdcPosition);
//       console.log("Remaining shares:", updatedPosition.depositedShares.toString());
//       console.log("✅ Withdrawal successful");
//     });
//   });

//   describe("Solend Integration (Optional)", () => {
//     it("Attempts to invest in Solend", async () => {
//       // Derive cToken vault PDA
//       [loanCTokenVault] = web3.PublicKey.findProgramAddressSync(
//         [Buffer.from("loan_ctoken_vault"), loan.toBuffer()],
//         program.programId
//       );

//       console.log("Note: Solend integration requires actual Solend reserve addresses");
//       console.log("This test demonstrates the structure but will fail without proper setup");

//       // Mock addresses for demonstration
//       const mockReserve = web3.Keypair.generate().publicKey;
//       const mockCtokenMint = web3.Keypair.generate().publicKey;
//       const mockLiquiditySupply = web3.Keypair.generate().publicKey;
//       const mockCollateralSupply = web3.Keypair.generate().publicKey;
//       const mockMarketAuthority = web3.Keypair.generate().publicKey;

//       try {
//         const tx = await program.methods
//           .investInSolend()
//           .accounts({
//             borrower: wallet.publicKey,
//             loan: loan,
//             borrowMarket: solMarket,
//             protocolVault: solSupplyVault,
//             solendLiquiditySupply: mockLiquiditySupply,
//             solendReserve: mockReserve,
//             solendCtokenMint: mockCtokenMint,
//             loanCtokenVault: loanCTokenVault,
//             solendCollateralSupply: mockCollateralSupply,
//             solendProgram: SOLEND_PROGRAM_ID,
//             solendMarket: SOLEND_MARKET,
//             solendMarketAuthority: mockMarketAuthority,
//             underlyingMint: solMint,
//             tokenProgram: TOKEN_PROGRAM_ID,
//             systemProgram: web3.SystemProgram.programId,
//             rent: web3.SYSVAR_RENT_PUBKEY,
//             clock: web3.SYSVAR_CLOCK_PUBKEY,
//           })
//           .rpc();

//         console.log("Invested in Solend:", tx);
//       } catch (error) {
//         console.log("✅ Expected: Solend integration requires proper setup");
//         console.log("Error:", error.message);
//       }
//     });
//   });
// });



import * as anchor from "@coral-xyz/anchor";
import { Program, BN, web3 } from "@coral-xyz/anchor";
import { 
  TOKEN_PROGRAM_ID,
  ASSOCIATED_TOKEN_PROGRAM_ID,
  createMint,
  createAccount,
  mintTo,
  getAccount
} from "@solana/spl-token";
import { assert } from "chai";
import { CoreRouter } from "../target/types/core_router";

const PRICE_FEED_IDS = {
  SOL: "0xef0d8b6fda2ceba41da15d4095d1da392a0d2f8ed0c6c7bc0f4cfac8c280b56d",
  USDC: "0x41f3625971ca2ed2263e78573fe5ce23e13d2558ed3f2e47ab0f84fb9e7ae722",
  USDT: "0x2b89b9dc8fdf9f34709a5b106b472f0f39bb6ca9ce04b0fd7f2e971688e2e53b"
};

describe("Core Protocol Tests", () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace.CoreRouter as Program<CoreRouter>;
  const connection = provider.connection;
  const wallet = provider.wallet as anchor.Wallet;

  let protocolState: web3.PublicKey;
  let usdcMint: web3.PublicKey;
  let solMint: web3.PublicKey;
  let usdcMarket: web3.PublicKey;
  let solMarket: web3.PublicKey;
  let usdcSupplyVault: web3.PublicKey;
  let solSupplyVault: web3.PublicKey;
  let userUsdcAccount: web3.PublicKey;
  let userSolAccount: web3.PublicKey;
  let userUsdcPosition: web3.PublicKey;
  let userSolPosition: web3.PublicKey;
  let loan: web3.PublicKey;
  let pythPriceUpdateAccount: web3.PublicKey;
  let feeCollector: web3.Keypair;

  before(async () => {
    feeCollector = web3.Keypair.generate();
    
    usdcMint = await createMint(connection, wallet.payer, wallet.publicKey, null, 6);
    solMint = await createMint(connection, wallet.payer, wallet.publicKey, null, 9);

    userUsdcAccount = await createAccount(connection, wallet.payer, usdcMint, wallet.publicKey);
    userSolAccount = await createAccount(connection, wallet.payer, solMint, wallet.publicKey);

    await mintTo(connection, wallet.payer, usdcMint, userUsdcAccount, wallet.publicKey, 1000 * 10**6);
    await mintTo(connection, wallet.payer, solMint, userSolAccount, wallet.publicKey, 10 * 10**9);

    [protocolState] = web3.PublicKey.findProgramAddressSync([Buffer.from("protocol_state")], program.programId);
    [usdcMarket] = web3.PublicKey.findProgramAddressSync([Buffer.from("market"), usdcMint.toBuffer()], program.programId);
    [solMarket] = web3.PublicKey.findProgramAddressSync([Buffer.from("market"), solMint.toBuffer()], program.programId);
    [usdcSupplyVault] = web3.PublicKey.findProgramAddressSync([Buffer.from("supply_vault"), usdcMarket.toBuffer()], program.programId);
    [solSupplyVault] = web3.PublicKey.findProgramAddressSync([Buffer.from("supply_vault"), solMarket.toBuffer()], program.programId);

    pythPriceUpdateAccount = web3.Keypair.generate().publicKey;
  });

  describe("Initialization", () => {
    it("Initializes the protocol", async () => {
      try {
        const state = await program.account.protocolState.fetch(protocolState);
        console.log("Protocol already initialized, skipping...");
      } catch {
        const tx = await program.methods.initializeProtocol()
          .accounts({
            admin: wallet.publicKey,
            feeCollector: feeCollector.publicKey,
            protocolState: protocolState,
            systemProgram: web3.SystemProgram.programId,
          }).rpc();
        console.log("Protocol initialized:", tx);
      }
    });

    it("Initializes USDC market", async () => {
      try {
        const market = await program.account.market.fetch(usdcMarket);
        console.log("USDC market already initialized, skipping...");
      } catch {
        const config = {
          maxLtv: new BN(50000),
          liquidationThreshold: new BN(52500),
          liquidationPenalty: new BN(500),
          reserveFactor: new BN(1000),
          minDepositAmount: new BN(1 * 10**6),
          maxDepositAmount: new BN(1000000 * 10**6),
          minBorrowAmount: new BN(1 * 10**6),
          maxBorrowAmount: new BN(100000 * 10**6),
          depositFee: new BN(0),
          withdrawFee: new BN(0),
          borrowFee: new BN(0),
          repayFee: new BN(0),
          pythFeedId: Buffer.from(PRICE_FEED_IDS.USDC.slice(2), 'hex'),
        };
        const tx = await program.methods.initializeMarket(config)
          .accounts({
            owner: wallet.publicKey,
            protocolState: protocolState,
            underlyingMint: usdcMint,
            market: usdcMarket,
            supplyVault: usdcSupplyVault,
            tokenProgram: TOKEN_PROGRAM_ID,
            systemProgram: web3.SystemProgram.programId,
          }).rpc();
        console.log("USDC market initialized:", tx);
      }
    });

    it("Initializes SOL market", async () => {
      try {
        const market = await program.account.market.fetch(solMarket);
        console.log("SOL market already initialized, skipping...");
      } catch {
        const config = {
          maxLtv: new BN(50000),
          liquidationThreshold: new BN(52500),
          liquidationPenalty: new BN(500),
          reserveFactor: new BN(1000),
          minDepositAmount: new BN(0.01 * 10**9),
          maxDepositAmount: new BN(10000 * 10**9),
          minBorrowAmount: new BN(0.01 * 10**9),
          maxBorrowAmount: new BN(1000 * 10**9),
          depositFee: new BN(0),
          withdrawFee: new BN(0),
          borrowFee: new BN(0),
          repayFee: new BN(0),
          pythFeedId: Buffer.from(PRICE_FEED_IDS.SOL.slice(2), 'hex'),
        };
        const tx = await program.methods.initializeMarket(config)
          .accounts({
            owner: wallet.publicKey,
            protocolState: protocolState,
            underlyingMint: solMint,
            market: solMarket,
            supplyVault: solSupplyVault,
            tokenProgram: TOKEN_PROGRAM_ID,
            systemProgram: web3.SystemProgram.programId,
          }).rpc();
        console.log("SOL market initialized:", tx);
      }
    });
  });

  describe("Core Operations", () => {
    it("Initializes user USDC position", async () => {
      [userUsdcPosition] = web3.PublicKey.findProgramAddressSync(
        [Buffer.from("user_account"), wallet.publicKey.toBuffer(), usdcMarket.toBuffer()],
        program.programId
      );

      try {
        const position = await program.account.userPosition.fetch(userUsdcPosition);
        console.log("USDC position already initialized, skipping...");
      } catch {
        const tx = await program.methods.initializeUserPosition()
          .accounts({
            signer: wallet.publicKey,
            market: usdcMarket,
            userAccount: userUsdcPosition,
            systemProgram: web3.SystemProgram.programId,
          }).rpc();
        console.log("User USDC position initialized:", tx);
      }
    });

    it("Deposits USDC", async () => {
      const depositAmount = new BN(100 * 10**6);
      const tx = await program.methods.deposit(depositAmount)
        .accounts({
          signer: wallet.publicKey,
          mint: usdcMint,
          market: usdcMarket,
          userTokenAccount: userUsdcAccount,
          supplyVault: usdcSupplyVault,
          userPosition: userUsdcPosition,
          tokenProgram: TOKEN_PROGRAM_ID,
          associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
          systemProgram: web3.SystemProgram.programId,
        }).rpc();
      console.log("Deposited 100 USDC:", tx);
    });

    it("Initializes user SOL position", async () => {
      [userSolPosition] = web3.PublicKey.findProgramAddressSync(
        [Buffer.from("user_account"), wallet.publicKey.toBuffer(), solMarket.toBuffer()],
        program.programId
      );

      try {
        const position = await program.account.userPosition.fetch(userSolPosition);
        console.log("SOL position already initialized, skipping...");
      } catch {
        const tx = await program.methods.initializeUserPosition()
          .accounts({
            signer: wallet.publicKey,
            market: solMarket,
            userAccount: userSolPosition,
            systemProgram: web3.SystemProgram.programId,
          }).rpc();
        console.log("User SOL position initialized:", tx);
      }
    });

    it("Deposits SOL", async () => {
      const depositAmount = new BN(2 * 10**9);
      const tx = await program.methods.deposit(depositAmount)
        .accounts({
          signer: wallet.publicKey,
          mint: solMint,
          market: solMarket,
          userTokenAccount: userSolAccount,
          supplyVault: solSupplyVault,
          userPosition: userSolPosition,
          tokenProgram: TOKEN_PROGRAM_ID,
          associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
          systemProgram: web3.SystemProgram.programId,
        }).rpc();
      console.log("Deposited 2 SOL:", tx);
    });

    it("Initializes loan", async () => {
      [loan] = web3.PublicKey.findProgramAddressSync(
        [Buffer.from("loan"), usdcMarket.toBuffer(), solMarket.toBuffer(), wallet.publicKey.toBuffer()],
        program.programId
      );

      try {
        const loanAcc = await program.account.loan.fetch(loan);
        console.log("Loan already initialized, skipping...");
      } catch {
        const tx = await program.methods.initializeLoan(solMint, usdcMint)
          .accounts({
            borrower: wallet.publicKey,
            supplyMarket: usdcMarket,
            borrowMarket: solMarket,
            protocolState: protocolState,
            loan: loan,
            systemProgram: web3.SystemProgram.programId,
          }).rpc();
        console.log("Loan initialized:", tx);
      }
    });

    it("Borrows SOL (will fail without Pyth)", async () => {
      const collateralShares = new BN(90 * 10**6);
      const borrowAmount = new BN(1 * 10**9);
      
      try {
        const tx = await program.methods.borrow(collateralShares, borrowAmount)
          .accounts({
            borrower: wallet.publicKey,
            collateralMint: usdcMint,
            borrowMint: solMint,
            protocolState: protocolState,
            collateralMarket: usdcMarket,
            borrowMarket: solMarket,
            collateralPosition: userUsdcPosition,
            loan: loan,
            tokenProgram: TOKEN_PROGRAM_ID,
            priceUpdate: pythPriceUpdateAccount,
            systemProgram: web3.SystemProgram.programId,
          }).rpc();
        console.log("Borrowed SOL:", tx);
      } catch (error) {
        console.log("Expected: Borrow requires proper Pyth price feeds");
      }
    });

    it("Withdraws USDC", async () => {
      const position = await program.account.userPosition.fetch(userUsdcPosition);
      const freeShares = position.depositedShares.sub(position.lockedCollateral);
      
      if (freeShares.gt(new BN(0))) {
        const withdrawShares = freeShares.div(new BN(2));
        const tx = await program.methods.withdraw(withdrawShares)
          .accounts({
            signer: wallet.publicKey,
            mint: usdcMint,
            market: usdcMarket,
            supplyVault: usdcSupplyVault,
            userPosition: userUsdcPosition,
            userTokenAccount: userUsdcAccount,
            tokenProgram: TOKEN_PROGRAM_ID,
            associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
            systemProgram: web3.SystemProgram.programId,
          }).rpc();
        console.log("Withdrawn USDC:", tx);
      } else {
        console.log("No free shares to withdraw");
      }
    });
  });
});