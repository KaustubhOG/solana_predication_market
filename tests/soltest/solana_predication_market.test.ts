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
  let userCollateral: any;
  let userOutcomeA: any;
  let userOutcomeB: any;
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

    // Initialize market
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

    // Setup user token accounts
    userCollateral = await getOrCreateAssociatedTokenAccount(
      provider.connection,
      authority.payer,
      collateralMint,
      authority.publicKey
    );

    userOutcomeA = await getOrCreateAssociatedTokenAccount(
      provider.connection,
      authority.payer,
      outcomeAMintPda,
      authority.publicKey
    );

    userOutcomeB = await getOrCreateAssociatedTokenAccount(
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
      1000000 // 1 token
    );

    console.log("✓ Setup complete");
  });

  it("Split tokens", async () => {
    const amount = 500000; // 0.5 tokens

    // Split: deposit collateral → get outcome tokens
    await program.methods
      .splitTokens(marketId, new anchor.BN(amount))
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

    // Check user received outcome tokens
    const outcomeABalance = await provider.connection.getTokenAccountBalance(userOutcomeA.address);
    const outcomeBBalance = await provider.connection.getTokenAccountBalance(userOutcomeB.address);

    assert.equal(outcomeABalance.value.amount, amount.toString());
    assert.equal(outcomeBBalance.value.amount, amount.toString());

    console.log("✓ Split successful - User received outcome tokens");
  });

  it("Merge tokens", async () => {
    // Merge: burn outcome tokens → get collateral back
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

    // Check outcome tokens were burned
    const outcomeABalance = await provider.connection.getTokenAccountBalance(userOutcomeA.address);
    const outcomeBBalance = await provider.connection.getTokenAccountBalance(userOutcomeB.address);

    assert.equal(outcomeABalance.value.amount, "0");
    assert.equal(outcomeBBalance.value.amount, "0");

    console.log("✓ Merge successful - Outcome tokens burned");
  });
});