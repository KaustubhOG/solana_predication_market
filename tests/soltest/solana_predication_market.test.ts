import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { SolanaPredicationMarket } from "../target/types/solana_predication_market";
import { PublicKey, SystemProgram } from "@solana/web3.js";
import { TOKEN_PROGRAM_ID, createMint, getOrCreateAssociatedTokenAccount, mintTo } from "@solana/spl-token";
import { assert } from "chai";

describe("Prediction Market Tests", () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace.SolanaPredicationMarket as Program<SolanaPredicationMarket>;
  const authority = provider.wallet as anchor.Wallet;
  
  let collateralMint: PublicKey;
  let marketPda: PublicKey;
  let vaultPda: PublicKey;
  let outcomeAMintPda: PublicKey;
  let outcomeBMintPda: PublicKey;
  const marketId = 1;

  before(async () => {
    // Create collateral token
    collateralMint = await createMint(
      provider.connection,
      authority.payer,
      authority.publicKey,
      null,
      6
    );

    // Derive PDAs
    [marketPda] = PublicKey.findProgramAddressSync(
      [Buffer.from("market"), new anchor.BN(marketId).toArrayLike(Buffer, "le", 4)],
      program.programId
    );

    [vaultPda] = PublicKey.findProgramAddressSync(
      [Buffer.from("vault"), new anchor.BN(marketId).toArrayLike(Buffer, "le", 4)],
      program.programId
    );

    [outcomeAMintPda] = PublicKey.findProgramAddressSync(
      [Buffer.from("outcome_a"), new anchor.BN(marketId).toArrayLike(Buffer, "le", 4)],
      program.programId
    );

    [outcomeBMintPda] = PublicKey.findProgramAddressSync(
      [Buffer.from("outcome_b"), new anchor.BN(marketId).toArrayLike(Buffer, "le", 4)],
      program.programId
    );

    console.log("✓ Setup complete");
  });

  it("Initialize market", async () => {
    const currentTime = Math.floor(Date.now() / 1000);
    const settlementDeadline = currentTime + 86400;

    await program.methods
      .initializeMarket(marketId, new anchor.BN(settlementDeadline))
      .accounts({
        market: marketPda,
        authority: authority.publicKey,
        collateralMint: collateralMint,
        collateralVault: vaultPda,
        outcomeAMint: outcomeAMintPda,
        outcomeBMint: outcomeBMintPda,
        tokenProgram: TOKEN_PROGRAM_ID,
        systemProgram: SystemProgram.programId,
        rent: anchor.web3.SYSVAR_RENT_PUBKEY,
      })
      .rpc();

    const market = await program.account.market.fetch(marketPda);
    assert.equal(market.marketId, marketId);
    assert.equal(market.isSettled, false);
    console.log("✓ Market initialized");
  });

  it("Split and merge tokens", async () => {
    // Setup user token accounts
    const userCollateral = await getOrCreateAssociatedTokenAccount(
      provider.connection,
      authority.payer,
      collateralMint,
      authority.publicKey
    );

    const userOutcomeA = await getOrCreateAssociatedTokenAccount(
      provider.connection,
      authority.payer,
      outcomeAMintPda,
      authority.publicKey
    );

    const userOutcomeB = await getOrCreateAssociatedTokenAccount(
      provider.connection,
      authority.payer,
      outcomeBMintPda,
      authority.publicKey
    );

    // Mint collateral to user
    await mintTo(
      provider.connection,
      authority.payer,
      collateralMint,
      userCollateral.address,
      authority.publicKey,
      1000000 // 1 token (6 decimals)
    );

    // Split tokens: deposit collateral, get outcome tokens
    const splitAmount = 500000; // 0.5 tokens
    await program.methods
      .splitTokens(marketId, new anchor.BN(splitAmount))
      .accounts({
        market: marketPda,
        user: authority.publicKey,
        userCollateral: userCollateral.address,
        collateralVault: vaultPda,
        outcomeAMint: outcomeAMintPda,
        outcomeBMint: outcomeBMintPda,
        userOutcomeA: userOutcomeA.address,
        userOutcomeB: userOutcomeB.address,
        tokenProgram: TOKEN_PROGRAM_ID,
      })
      .rpc();

    console.log("✓ Tokens split - User received outcome tokens");

    // Check balances after split
    let outcomeABalance = (await provider.connection.getTokenAccountBalance(userOutcomeA.address)).value.amount;
    let outcomeBBalance = (await provider.connection.getTokenAccountBalance(userOutcomeB.address)).value.amount;
    assert.equal(outcomeABalance, splitAmount.toString());
    assert.equal(outcomeBBalance, splitAmount.toString());

    // Merge tokens: burn outcome tokens, get collateral back
    const mergeAmount = 300000; // 0.3 tokens
    await program.methods
      .mergeTokens(marketId)
      .accounts({
        market: marketPda,
        user: authority.publicKey,
        userCollateral: userCollateral.address,
        collateralVault: vaultPda,
        outcomeAMint: outcomeAMintPda,
        outcomeBMint: outcomeBMintPda,
        userOutcomeA: userOutcomeA.address,
        userOutcomeB: userOutcomeB.address,
        tokenProgram: TOKEN_PROGRAM_ID,
      })
      .rpc();

    console.log("✓ Tokens merged - User got collateral back");

    // Check balances after merge
    outcomeABalance = (await provider.connection.getTokenAccountBalance(userOutcomeA.address)).value.amount;
    outcomeBBalance = (await provider.connection.getTokenAccountBalance(userOutcomeB.address)).value.amount;
    
    const expectedRemaining = splitAmount - mergeAmount;
    assert.equal(outcomeABalance, expectedRemaining.toString());
    assert.equal(outcomeBBalance, expectedRemaining.toString());

    console.log("✓ Balances verified correctly");
  });
});