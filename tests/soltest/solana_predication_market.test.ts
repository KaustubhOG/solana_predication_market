import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { SolanaPredicationMarket } from "../target/types/solana_predication_market";
import { PublicKey, Keypair, SystemProgram } from "@solana/web3.js";
import { TOKEN_PROGRAM_ID, createMint } from "@solana/spl-token";
import { assert } from "chai";

describe("Prediction Market Tests", () => {
  // Setup the provider and program
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace.SolanaPredicationMarket as Program<SolanaPredicationMarket>;
  
  // The wallet that will create the market
  const authority = provider.wallet as anchor.Wallet;
  
  // This will store our collateral token mint address
  let collateralMint: PublicKey;

  // Run this before all tests
  before(async () => {
    console.log("Setting up test environment...");
    
    // Create a token that users will use as collateral
    collateralMint = await createMint(
      provider.connection,
      authority.payer,
      authority.publicKey,
      null,
      6 // 6 decimals (like USDC)
    );

    console.log("✓ Collateral token created:", collateralMint.toBase58());
  });

  it("Can initialize a prediction market", async () => {
    console.log("\n--- Test: Initialize Market ---");
    
    // Step 1: Set up market parameters
    const marketId = 1;
    const currentTime = Math.floor(Date.now() / 1000);
    const settlementDeadline = currentTime + 86400; // Settles in 24 hours
    
    console.log("Market ID:", marketId);
    console.log("Settlement Deadline:", new Date(settlementDeadline * 1000));

    // Step 2: Find the addresses (PDAs) for our market accounts
    // These are deterministic addresses derived from seeds
    const [marketPda] = PublicKey.findProgramAddressSync(
      [
        Buffer.from("market"),
        new anchor.BN(marketId).toArrayLike(Buffer, "le", 4)
      ],
      program.programId
    );

    const [vaultPda] = PublicKey.findProgramAddressSync(
      [
        Buffer.from("vault"),
        new anchor.BN(marketId).toArrayLike(Buffer, "le", 4)
      ],
      program.programId
    );

    const [outcomeAMintPda] = PublicKey.findProgramAddressSync(
      [
        Buffer.from("outcome_a"),
        new anchor.BN(marketId).toArrayLike(Buffer, "le", 4)
      ],
      program.programId
    );

    const [outcomeBMintPda] = PublicKey.findProgramAddressSync(
      [
        Buffer.from("outcome_b"),
        new anchor.BN(marketId).toArrayLike(Buffer, "le", 4)
      ],
      program.programId
    );

    console.log("Market Address:", marketPda.toBase58());

    // Step 3: Call the initialize_market instruction
    const tx = await program.methods
      .initializeMarket(
        marketId,
        new anchor.BN(settlementDeadline)
      )
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

    console.log("✓ Transaction successful:", tx);

    // Step 4: Verify the market was created correctly
    const market = await program.account.market.fetch(marketPda);

    // Check that all values are correct
    assert.equal(
      market.authority.toBase58(),
      authority.publicKey.toBase58(),
      "Authority should match"
    );
    
    assert.equal(
      market.marketId,
      marketId,
      "Market ID should match"
    );
    
    assert.equal(
      market.settlementDeadline.toNumber(),
      settlementDeadline,
      "Settlement deadline should match"
    );
    
    assert.equal(
      market.isSettled,
      false,
      "Market should not be settled yet"
    );
    
    assert.equal(
      market.totalCollateralLocked.toNumber(),
      0,
      "Should have no collateral locked initially"
    );

    console.log("✓ All market fields verified correctly!");
    console.log("\nMarket Details:");
    console.log("- Market ID:", market.marketId);
    console.log("- Authority:", market.authority.toBase58());
    console.log("- Settlement Deadline:", new Date(market.settlementDeadline.toNumber() * 1000));
    console.log("- Is Settled:", market.isSettled);
    console.log("- Total Collateral Locked:", market.totalCollateralLocked.toNumber());
  });
});