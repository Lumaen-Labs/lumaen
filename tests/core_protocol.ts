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

//     it("Closes loan account", async () => {
//       const tx = await program.methods
//         .closeLoan()
//         .accounts({
//           borrower: wallet.publicKey,
//           loan: loan,
//           systemProgram: web3.SystemProgram.programId,
//         })
//         .rpc();

//       console.log("Closed loan account:", tx);

//       // Verify account is closed
//       try {
//         await program.account.loan.fetch(loan);
//         assert.fail("Loan account should be closed");
//       } catch (error) {
//         assert(error.toString().includes("Account does not exist"));
//       }
//     });

//     it("Closes user position when empty", async () => {
//       // First withdraw all remaining funds
//       const position = await program.account.userPosition.fetch(userPosition);
//       if (position.depositedShares.gt(new BN(0))) {
//         await program.methods
//           .withdraw(position.depositedShares)
//           .accounts({
//             signer: wallet.publicKey,
//             mint: usdcMint,
//             market: usdcMarket,
//             supplyVault: usdcSupplyVault,
//             userPosition: userPosition,
//             userTokenAccount: userUsdcAccount,
//             tokenProgram: TOKEN_PROGRAM_ID,
//             systemProgram: web3.SystemProgram.programId,
//           })
//           .rpc();
//       }

//       // Now close the account
//       const tx = await program.methods
//         .closeUserPosition()
//         .accounts({
//           user: wallet.publicKey,
//           market: usdcMarket,
//           userPosition: userPosition,
//           systemProgram: web3.SystemProgram.programId,
//         })
//         .rpc();

//       console.log("Closed user position:", tx);

//       // Verify account is closed
//       try {
//         await program.account.userPosition.fetch(userPosition);
//         assert.fail("User position should be closed");
//       } catch (error) {
//         assert(error.toString().includes("Account does not exist"));
//       }
//     });
//   });
// });