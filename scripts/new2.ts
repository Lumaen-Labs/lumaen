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
import { PythSolanaReceiver } from "@pythnetwork/pyth-solana-receiver";

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
//   const pythReceiver = new anchor.web3.PublicKey("7UVimffxr9ow1uXYxsr4LHAcV58mLzhmwaeKvJ1pjLiE");
//   const connection: anchor.web3.Connection = provider.connection;
//   const wallet: Wallet = provider.wallet;
  const pythSolanaReceiver = new PythSolanaReceiver({ connection, wallet });
 
  // Pyth feed IDs
  const usdcFeedId = Buffer.from("41f3625971ca2ed2263e78573fe5ce23e13d2558ed3f2e47ab0f84fb9e7ae722", "hex");
  const usdtFeedId = Buffer.from("1fc18861232290221461220bd4e2acd1dCDFbc89c84092c93c18bdc7756c1588", "hex");

  const SOL_PRICE_FEED_ID =
    "0xeaa020c61cc479712813461ce153894a96a6c00b21ed0cfc2798d1f9a9e9c94a";

   const solUsdPriceFeedAccount = pythSolanaReceiver
  .getPriceFeedAccountAddress(0,SOL_PRICE_FEED_ID )
  .toBase58();

  // Calculate PriceUpdateV2 PDAs
//   const [usdcPriceUpdatePda] = anchor.web3.PublicKey.findProgramAddressSync(
//     [Buffer.from("price_account"), usdcFeedId],
//     pythReceiver
//   );

//   const [usdtPriceUpdatePda] = anchor.web3.PublicKey.findProgramAddressSync(
//     [Buffer.from("price_account"), usdtFeedId],
//     pythReceiver
//   );

//   console.log("USDC Price Update PDA:", usdcPriceUpdatePda.toBase58());
//   console.log("USDT Price Update PDA:", usdtPriceUpdatePda.toBase58());

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

  // Fetch protocol state after init
  const protocolData = await program.account.protocolState.fetch(protocolStatePda);
  console.log("Protocol state data:", protocolData);

  // Step 2: Create Mock Mint and Initialize Market
  const mockMint1 = await createMint(
    connection,
    wallet.payer,
    wallet.publicKey,
    null,
    9 // Decimals
  );
  console.log("Mock mint 1 created:", mockMint1.toBase58());

  const mockMint2 = await createMint(
    connection,
    wallet.payer,
    wallet.publicKey,
    null,
    6 // Decimals
  );
  console.log("Mock mint 2 created:", mockMint2.toBase58());  

  const [marketPda1] = anchor.web3.PublicKey.findProgramAddressSync(
    [Buffer.from("market"), mockMint1.toBuffer()],
    program.programId
  );

  const [marketPda2] = anchor.web3.PublicKey.findProgramAddressSync(
    [Buffer.from("market"), mockMint2.toBuffer()],
    program.programId
  );

  const [supplyVaultPda1] = anchor.web3.PublicKey.findProgramAddressSync(
    [Buffer.from("supply_vault"), marketPda1.toBuffer()],
    program.programId
  );

  const [supplyVaultPda2] = anchor.web3.PublicKey.findProgramAddressSync(
    [Buffer.from("supply_vault"), marketPda2.toBuffer()],
    program.programId
  );

  const marketConfig1 = {
    maxLtv: new BN(50000),
    liquidationThreshold: new BN(52500),
    liquidationPenalty: new BN(500),
    reserveFactor: new BN(1000),
    minDepositAmount: new BN(0),
    maxDepositAmount: new BN(1000000),
    minBorrowAmount: new BN(0),
    maxBorrowAmount: new BN(500000),
    depositFee: new BN(0),
    withdrawFee: new BN(0),
    borrowFee: new BN(0),
    repayFee: new BN(0),
    pythFeedId: Array.from(usdcFeedId), // Pass as array
  };
  const marketConfig2 = {
    maxLtv: new BN(60000),
    liquidationThreshold: new BN(62500),
    liquidationPenalty: new BN(600),
    reserveFactor: new BN(1500),
    minDepositAmount: new BN(0),
    maxDepositAmount: new BN(2000000),
    minBorrowAmount: new BN(200),
    maxBorrowAmount: new BN(1000000),
    depositFee: new BN(0),
    withdrawFee: new BN(0),
    borrowFee: new BN(0),
    repayFee: new BN(0),
    pythFeedId: Array.from(usdtFeedId), // Pass as array
  };

  try {
    await program.account.market.fetch(marketPda1);
    console.log("⚠️  Market already initialized, skipping...");
  } catch (err: any) {
    if (err.name === 'AccountDoesNotExistError' || err.toString().includes('Account does not exist')) {
      try {
        await program.methods
          .initializeMarket(marketConfig1)
          .accounts({
            owner: wallet.publicKey,
            protocolState: protocolStatePda,
            mint: mockMint1,
            market: marketPda1,
            supplyVault: supplyVaultPda1,
            tokenProgram: TOKEN_PROGRAM_ID,
            systemProgram: anchor.web3.SystemProgram.programId,
          } as any)
          .rpc();
        console.log("Market initialized for mint:", mockMint1.toBase58());
      } catch (initErr) {
        console.error("Error initializing market:", initErr);
        return;
      }
    } else {
      console.error("Error checking market:", err);
      return;
    }
  }

  // Fetch market1 data
//   const market1Data = await program.account.market.fetch(marketPda1);
//   console.log("Market1 data:", market1Data);

  try {
    await program.account.market.fetch(marketPda2);
    console.log("⚠️  Market already initialized, skipping...");
  } catch (err: any) {
    if (err.name === 'AccountDoesNotExistError' || err.toString().includes('Account does not exist')) {
      try {
        await program.methods
          .initializeMarket(marketConfig2)
          .accounts({
            owner: wallet.publicKey,
            protocolState: protocolStatePda,
            mint: mockMint2,
            market: marketPda2,
            supplyVault: supplyVaultPda2,
            tokenProgram: TOKEN_PROGRAM_ID,
            systemProgram: anchor.web3.SystemProgram.programId,
          } as any)
          .rpc();
        console.log("Market initialized for mint:", mockMint2.toBase58());
      } catch (initErr) {
        console.error("Error initializing market:", initErr);
        return;
      }
    } else {
      console.error("Error checking market:", err);
      return;
    }
  }

  // Fetch market2 data
//   const market2Data = await program.account.market.fetch(marketPda2);
//   console.log("Market2 data:", market2Data);


  // Step 3: Prepare for Deposit - Create User ATA and Mint Tokens
  const userTokenAccount1 = await createAssociatedTokenAccount(
    connection,
    wallet.payer,
    mockMint1,
    wallet.publicKey
  );
  console.log("User token account:", userTokenAccount1.toBase58());

  const mintAmount1 = new BN(100000); // Mint 100000 tokens to user
  await mintTo(
    connection,
    wallet.payer,
    mockMint1,
    userTokenAccount1,
    wallet.publicKey,
    mintAmount1.toNumber() // Use toNumber() if BN is not directly accepted
  );
  console.log("Minted", mintAmount1.toString(), "tokens to user");


  const userTokenAccount2 = await createAssociatedTokenAccount(
    connection,
    wallet.payer,
    mockMint2,
    wallet.publicKey
  );
  console.log("User token account 2:", userTokenAccount2.toBase58());

  const mintAmount2 = new BN(2000000); // Mint 2000000 tokens to user
  await mintTo(
    connection,
    wallet.payer,
    mockMint2,
    userTokenAccount2,
    wallet.publicKey,
    mintAmount2.toNumber() // Use toNumber() if BN is not directly accepted
  );
  console.log("Minted", mintAmount2.toString(), "tokens to user");

  // User Position PDA (will be init_if_needed)
  const [userAccountPda1] = anchor.web3.PublicKey.findProgramAddressSync(
    [Buffer.from("user_account"), wallet.publicKey.toBuffer(), marketPda1.toBuffer()],
    program.programId
  );

  const [userAccountPda2] = anchor.web3.PublicKey.findProgramAddressSync(
    [Buffer.from("user_account"), wallet.publicKey.toBuffer(), marketPda2.toBuffer()],
    program.programId
  );

  // Step 4: Deposit
  const depositAmount1 = new BN(1000);

  try {
    await program.methods
      .deposit(depositAmount1)
      .accounts({
        signer: wallet.publicKey, // user signing the transaction
        mint: mockMint1,           // USDC , USDT
        market: marketPda1,
        userTokenAccount: userTokenAccount1,
        supplyVault: supplyVaultPda1,
        userAccount: userAccountPda1,
        tokenProgram: TOKEN_PROGRAM_ID,
        associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
        systemProgram: anchor.web3.SystemProgram.programId,
      } as any)
      .rpc();
    console.log("Deposit successful! Amount:", depositAmount1.toString());
  } catch (err) {
    console.error("Error during deposit:", err);
  }


  const depositAmount2 = new BN(2000000);

  try {
    await program.methods
      .deposit(depositAmount2)
      .accounts({
        signer: wallet.publicKey,
        mint: mockMint2,
        market: marketPda2,
        userTokenAccount: userTokenAccount2,
        supplyVault: supplyVaultPda2,
        userAccount: userAccountPda2,
        tokenProgram: TOKEN_PROGRAM_ID,
        associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
        systemProgram: anchor.web3.SystemProgram.programId,
      } as any)
      .rpc();
    console.log("Deposit successful! Amount:", depositAmount2.toString());
  } catch (err) {
    console.error("Error during deposit:", err);
  }

  // Fetch after deposits
  const postDepositMarket1 = await program.account.market.fetch(marketPda1);
  console.log("Post-deposit Market1:", postDepositMarket1);

  const postDepositUser1 = await program.account.userPosition.fetch(userAccountPda1);
  console.log("Post-deposit User Position1:", postDepositUser1);

  const postDepositMarket2 = await program.account.market.fetch(marketPda2);
  console.log("Post-deposit Market2:", postDepositMarket2);

  const postDepositUser2 = await program.account.userPosition.fetch(userAccountPda2);
  console.log("Post-deposit User Position2:", postDepositUser2);



//   const withdrawAmount = new BN(100);
//   try {
//     await program.methods
//       .withdraw(withdrawAmount)
//       .accounts({
//         signer: wallet.publicKey,
//         mint: mockMint,
//         market: marketPda1,
//         supplyVault: supplyVaultPda1,
//         userTokenAccount: userTokenAccount,
//         userAccount: userAccountPda,
//         tokenProgram: TOKEN_PROGRAM_ID,
//         associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
//         systemProgram: anchor.web3.SystemProgram.programId,
//       } as any)
//       .rpc();
//     console.log("Withdraw successful! Amount:", withdrawAmount.toString());
//   } catch (err) {
//     console.error("Error during withdraw:", err);
//   }
// }

  // Borrow: Collateral from market1 (USDC), borrow from market2 (USDT)
  const sharesAmount = new BN(500); // Collateral shares from market1
  const borrowAmount = new BN(2500); // Borrow 5x leverage example

  const [loanPda] = anchor.web3.PublicKey.findProgramAddressSync(
    [Buffer.from("loan"), marketPda1.toBuffer(), marketPda2.toBuffer(), wallet.publicKey.toBuffer()],
    program.programId
  );

  try {
    await program.methods
      .borrow(sharesAmount, borrowAmount)
      .accounts({
        borrower: wallet.publicKey,
        collateralMint: mockMint1,
        borrowMint: mockMint2,
        protocolState: protocolStatePda,
        collateralMarket: marketPda1,
        borrowMarket: marketPda2,
        collateralPosition: userAccountPda1,
        loan: loanPda,
        tokenProgram: TOKEN_PROGRAM_ID,
        priceUpdate: solUsdPriceFeedAccount,  // For collateral (USDC)
        // Add if program expects separate: borrowPriceUpdate: usdtPriceUpdatePda,
        systemProgram: anchor.web3.SystemProgram.programId,
      } as any)
      .rpc();
    console.log("Borrow successful! Borrowed:", borrowAmount.toString());
  } catch (err) {
    console.error("Error during borrow:", err);
  }

  // Fetch after borrow
  const postBorrowLoan = await program.account.loan.fetch(loanPda);
  console.log("Post-borrow Loan:", postBorrowLoan);

  const postBorrowCollateralPosition = await program.account.userPosition.fetch(userAccountPda1);
  console.log("Post-borrow Collateral Position:", postBorrowCollateralPosition);

  const postBorrowMarket1 = await program.account.market.fetch(marketPda1);
  console.log("Post-borrow Market1:", postBorrowMarket1);

  const postBorrowMarket2 = await program.account.market.fetch(marketPda2);
  console.log("Post-borrow Market2:", postBorrowMarket2);

  // Repay
  const repayAmount = new BN(2500 + 100); // Assume with interest/fee

  try {
    await program.methods
      .repay(repayAmount)
      .accounts({
        borrower: wallet.publicKey,
        mint: mockMint2,
        loan: loanPda,
        protocolState: protocolStatePda,
        collateralMarket: marketPda1,
        borrowMarket: marketPda2,
        userPosition: userAccountPda1,
        userTokenAccount: userTokenAccount2,
        supplyVault: supplyVaultPda2,
        tokenProgram: TOKEN_PROGRAM_ID,
        associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
        systemProgram: anchor.web3.SystemProgram.programId,
      } as any)
      .rpc();
    console.log("Repay successful! Repaid:", repayAmount.toString());
  } catch (err) {
    console.error("Error during repay:", err);
  }

 // Fetch after repay (loan should be closed)
  try {
    const postRepayLoan = await program.account.loan.fetch(loanPda);
    console.log("Post-repay Loan:", postRepayLoan);
  } catch (err) {
    console.log("Loan account closed as expected:", err);
  }

  const postRepayCollateralPosition = await program.account.userPosition.fetch(userAccountPda1);
  console.log("Post-repay Collateral Position:", postRepayCollateralPosition);

  const postRepayMarket1 = await program.account.market.fetch(marketPda1);
  console.log("Post-repay Market1:", postRepayMarket1);

  const postRepayMarket2 = await program.account.market.fetch(marketPda2);
  console.log("Post-repay Market2:", postRepayMarket2);

}

main().catch((err) => console.error(err));