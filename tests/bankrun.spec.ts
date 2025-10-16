import { describe, it } from "node:test";
import * as anchor from "@coral-xyz/anchor";
import { BN, Program } from "@coral-xyz/anchor";
import { BankrunProvider } from "anchor-bankrun";
import { TOKEN_PROGRAM_ID, ASSOCIATED_TOKEN_PROGRAM_ID } from "@solana/spl-token";
import { createAccount, createMint, mintTo } from "spl-token-bankrun";
import { startAnchor, BanksClient, ProgramTestContext } from "solana-bankrun";
import { PublicKey, Keypair, SystemProgram } from "@solana/web3.js";

// @ts-ignore
import IDL from "../target/idl/core_protocol.json";
import { CoreRouter } from "../target/types/core_router";

describe("Core Protocol Tests", async () => {
  let provider: BankrunProvider;
  let program: Program<CoreRouter>;
  let context: ProgramTestContext;
  let banksClient: BanksClient;
  let signer: Keypair;
  let mint: PublicKey;
  let market: PublicKey;
  let supplyVault: PublicKey;
  let userPosition: PublicKey;
  let userTokenAccount: PublicKey;
  let protocolState: PublicKey;


  // Initialize test context
  context = await startAnchor(
    __dirname + "/..",
    [{ name: "core_protocol", programId: new PublicKey(IDL.address) }],
    []
  );
  
  provider = new BankrunProvider(context);
  program = new Program<CoreRouter>(IDL as CoreRouter, provider);
  banksClient = context.banksClient;
  signer = provider.wallet.payer;

  console.log("set up completed");

  it("Initialize Protocol", async () => {
    const feeCollector = Keypair.generate();
    [protocolState] = PublicKey.findProgramAddressSync(
      [Buffer.from("protocol_state")],
      program.programId
    );

    console.log("I'm here");

    const initTx = await program.methods
      .initializeProtocol()
      .accounts({
        admin: signer.publicKey,
        feeCollector: feeCollector.publicKey,
        protocolState: protocolState,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .rpc({ commitment: "confirmed" });

    console.log("Protocol initialized:", initTx);
  });

  // it("Initialize Market", async () => {
  //   // Create test token mint
  //   mint = await createMint(
  //     banksClient,
  //     signer,
  //     signer.publicKey,
  //     null,
  //     6
  //   );

  //   [market] = PublicKey.findProgramAddressSync(
  //     [Buffer.from("market"), mint.toBuffer()],
  //     program.programId
  //   );

  //   [supplyVault] = PublicKey.findProgramAddressSync(
  //     [Buffer.from("supply_vault"), market.toBuffer()],
  //     program.programId
  //   );

  //   const marketConfig = {
  //     maxLtv: 7500,
  //     liquidationThreshold: 8000,
  //     liquidationPenalty: 500,
  //     reserveFactor: 1000,
  //     minDepositAmount: new BN(1000000),
  //     maxDepositAmount: new BN(1000000000000),
  //     minBorrowAmount: new BN(1000000),
  //     maxBorrowAmount: new BN(1000000000000),
  //     depositFee: 0,
  //     withdrawFee: 0,
  //     borrowFee: 0,
  //     repayFee: 0,
  //   };

  //   const initMarketTx = await program.methods
  //     .initializeMarket(marketConfig)
  //     .accounts({
  //       owner: signer.publicKey,
  //       underlyingMint: mint,
  //       market,
  //       supplyVault,
  //       tokenProgram: TOKEN_PROGRAM_ID,
  //       systemProgram: anchor.web3.SystemProgram.programId,
  //     })
  //     .rpc({ commitment: "confirmed" });

  //   console.log("Market initialized:", initMarketTx);
  // });

  // it("Initialize User Position", async () => {
  //   [userPosition] = PublicKey.findProgramAddressSync(
  //     [Buffer.from("user_account"), signer.publicKey.toBuffer(), mint.toBuffer()],
  //     program.programId
  //   );

  //   const initUserTx = await program.methods
  //     .initializeUserPosition(mint)
  //     .accounts({
  //       signer: signer.publicKey,
  //       userAccount: userPosition,
  //       systemProgram: anchor.web3.SystemProgram.programId,
  //     })
  //     .rpc({ commitment: "confirmed" });

  //   console.log("User position initialized:", initUserTx);
  // });

  // it("Create and Fund User Token Account", async () => {
  //   userTokenAccount = await createAccount(
  //     banksClient,
  //     signer,
  //     mint,
  //     signer.publicKey
  //   );

  //   const amount = new BN(1000000000);
  //   await mintTo(
  //     banksClient,
  //     signer,
  //     mint,
  //     userTokenAccount,
  //     signer,
  //     amount.toNumber()
  //   );

  //   console.log("User token account created and funded:", userTokenAccount.toBase58());
  // });

  // it("Test Deposit", async () => {
  //   const depositAmount = new BN(100000000);

  //   const depositTx = await program.methods
  //     .deposit(depositAmount)
  //     .accounts({
  //       signer: signer.publicKey,
  //       mint,
  //       market,
  //       supplyVault,
  //       userTokenAccount,
  //       userPosition,
  //       tokenProgram: TOKEN_PROGRAM_ID,
  //       associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
  //       systemProgram: anchor.web3.SystemProgram.programId,
  //     })
  //     .rpc({ commitment: "confirmed" });

  //   console.log("Deposit successful:", depositTx);
  // });

  // it("Test Withdraw", async () => {
  //   const withdrawAmount = new BN(50000000);

  //   const withdrawTx = await program.methods
  //     .withdraw(withdrawAmount)
  //     .accounts({
  //       signer: signer.publicKey,
  //       mint,
  //       market,
  //       supplyVault,
  //       userPosition,
  //       userTokenAccount,
  //       tokenProgram: TOKEN_PROGRAM_ID,
  //       associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
  //       systemProgram: anchor.web3.SystemProgram.programId,
  //     })
  //     .rpc({ commitment: "confirmed" });

  //   console.log("Withdraw successful:", withdrawTx);
  // });
});