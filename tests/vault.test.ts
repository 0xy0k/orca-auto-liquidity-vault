import * as anchor from "@coral-xyz/anchor";
import { BN, Program } from "@coral-xyz/anchor";
import {
  PublicKey,
  Keypair,
  SystemProgram,
  SYSVAR_RENT_PUBKEY,
  LAMPORTS_PER_SOL,
} from "@solana/web3.js";
import { expect } from "chai";
import { Vault } from "../target/types/vault"; // adjust the import path
import { TOKEN_PROGRAM_ID } from "@coral-xyz/anchor/dist/cjs/utils/token";
import {
  createAccount,
  createMint,
  createSyncNativeInstruction,
  getAssociatedTokenAddress,
  mintTo,
} from "@solana/spl-token";
import {
  buildWhirlpoolClient,
  ORCA_WHIRLPOOL_PROGRAM_ID,
  PDAUtil,
  PriceMath,
  WhirlpoolContext,
} from "@orca-so/whirlpools-sdk";
import { Percentage } from "@orca-so/common-sdk";
import { Decimal } from "decimal.js";

describe("Vault Tests", () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace.Vault as Program<Vault>;
  const wallet = provider.wallet;

  let admin = Keypair.generate();
  let vaultPda: PublicKey;
  let vaultBump: number;
  let whirlpoolConfig = new PublicKey(
    "2LecshUwdy9xi7meFgHtFJQNSKk4KdTrcpvaB56dP2NQ"
  );

  let tokenSolMint: PublicKey = new PublicKey(
    "So11111111111111111111111111111111111111112" // WSOL mint
  );

  let whirlpool: PublicKey;
  let tokenUSDCMint: PublicKey;
  let tokenAMint: PublicKey;
  let tokenBMint: PublicKey;
  let tokenAVault = Keypair.generate();
  let tokenBVault = Keypair.generate();
  let shareMint = Keypair.generate();
  let adminSolAccount: PublicKey;
  let adminUSDCAccount: PublicKey;
  let adminShareAccount: PublicKey;
  let adminTokenAAccount: PublicKey;
  let adminTokenBAccount: PublicKey;
  // Add other necessary variables

  before(async () => {
    // Airdrop 10000 SOL to admin
    const { blockhash, lastValidBlockHeight } =
      await provider.connection.getLatestBlockhash();
    const signature = await provider.connection.requestAirdrop(
      admin.publicKey,
      10000 * LAMPORTS_PER_SOL
    );
    await provider.connection.confirmTransaction(
      {
        signature,
        blockhash,
        lastValidBlockHeight,
      },
      "finalized"
    );

    adminSolAccount = await createAccount(
      provider.connection,
      admin,
      tokenSolMint,
      admin.publicKey
    );

    // Wrap SOL and send to admin's token account
    const wrapSolIx = SystemProgram.transfer({
      fromPubkey: admin.publicKey,
      toPubkey: adminSolAccount,
      lamports: 9000 * LAMPORTS_PER_SOL, // Wrapping 9000 SOL for testing
    });
    // Add synchronous token account creation and initialization
    const syncNativeIx = createSyncNativeInstruction(adminSolAccount);
    const tx = new anchor.web3.Transaction().add(wrapSolIx).add(syncNativeIx);

    await provider.sendAndConfirm(tx, [admin]);

    // Create USDC mint
    tokenUSDCMint = await createMint(
      provider.connection,
      admin, // payer
      admin.publicKey, // mint authority
      admin.publicKey, // freeze authority
      6
    );
    // Create admin's USDC token account
    adminUSDCAccount = await createAccount(
      provider.connection,
      admin,
      tokenUSDCMint,
      admin.publicKey
    );
    // Mint 10000 USDC to admin
    await mintTo(
      provider.connection,
      admin,
      tokenUSDCMint,
      adminUSDCAccount,
      admin.publicKey,
      10000000000 // 10000 USDC (with 6 decimals)
    );

    // Initialize Whirlpool client
    let whirlpoolContext = WhirlpoolContext.withProvider(
      provider,
      ORCA_WHIRLPOOL_PROGRAM_ID
    );
    let whirlpoolClient = buildWhirlpoolClient(whirlpoolContext);

    // Create Whirlpool
    const tickSpacing = 64; // Standard tick spacing
    const initialPrice = 10; // Initial price of 1 TokenA per TokenB
    let tokenADecimal: number;
    let tokenBDecimal: number;

    if (tokenSolMint.toBuffer().compare(tokenUSDCMint.toBuffer()) < 0) {
      tokenAMint = tokenSolMint;
      tokenBMint = tokenUSDCMint;
      adminTokenAAccount = adminSolAccount;
      adminTokenBAccount = adminUSDCAccount;
      tokenADecimal = 9;
      tokenBDecimal = 6;
    } else {
      tokenAMint = tokenUSDCMint;
      tokenBMint = tokenSolMint;
      adminTokenAAccount = adminUSDCAccount;
      adminTokenBAccount = adminSolAccount;
      tokenADecimal = 6;
      tokenBDecimal = 9;
    }

    // Get the Whirlpool PDA
    whirlpool = PDAUtil.getWhirlpool(
      ORCA_WHIRLPOOL_PROGRAM_ID,
      whirlpoolConfig,
      tokenAMint,
      tokenBMint,
      tickSpacing
    ).publicKey;

    // Calculate initial sqrt price
    const initialTickIndex = PriceMath.priceToInitializableTickIndex(
      new Decimal(initialPrice),
      tokenADecimal,
      tokenBDecimal,
      tickSpacing
    );

    // Create the Whirlpool
    const createPoolTx = await whirlpoolClient.createPool(
      whirlpoolConfig,
      tokenAMint,
      tokenBMint,
      tickSpacing,
      initialTickIndex,
      wallet.publicKey
    );

    // Wait for the transaction to be confirmed
    await createPoolTx.tx.buildAndExecute();

    // Setup: Create token mint, get PDAs, etc.
    [vaultPda, vaultBump] = PublicKey.findProgramAddressSync(
      [Buffer.from("vault"), whirlpool.toBuffer()],
      program.programId
    );

    // Initialize other test setup
  });

  it("Initialize vault", async () => {
    try {
      await program.methods
        .initializeVault(new BN(100), new BN(1000), -1000, 1000)
        .accounts({
          admin: admin.publicKey,
          vault: vaultPda,
          whirlpool: whirlpool,
          tokenAMint: tokenAMint,
          tokenBMint: tokenBMint,
          tokenAVault: tokenAVault.publicKey,
          tokenBVault: tokenBVault.publicKey,
          shareMint: shareMint.publicKey,
          tokenProgram: TOKEN_PROGRAM_ID,
          systemProgram: SystemProgram.programId,
          rent: SYSVAR_RENT_PUBKEY,
        })
        .signers([admin, tokenAVault, tokenBVault, shareMint])
        .rpc();

      const vaultAccount = await program.account.vault.fetch(vaultPda);
      expect(vaultAccount.admin).to.eql(admin.publicKey);
      expect(vaultAccount.tokenAMint).to.eql(tokenAMint);
      expect(vaultAccount.tokenBMint).to.eql(tokenBMint);
      expect(vaultAccount.whirlpool).to.eql(whirlpool);
      expect(vaultAccount.tokenAVault).to.eql(tokenAVault.publicKey);
      expect(vaultAccount.tokenBVault).to.eql(tokenBVault.publicKey);
      expect(vaultAccount.shareMint).to.eql(shareMint.publicKey);
      expect(vaultAccount.tokenAPrice.toString()).to.eql("100");
      expect(vaultAccount.tokenBPrice.toString()).to.eql("1000");
      expect(vaultAccount.tokenADecimal).to.eql(9);
      expect(vaultAccount.tokenBDecimal).to.eql(6);
      expect(vaultAccount.lowerTick).to.eql(-1000);
      expect(vaultAccount.upperTick).to.eql(1000);
    } catch (error) {
      console.error("Error:", error);
      throw error;
    }
  });

  // it("Update prices", async () => {
  //   const newPriceA = new BN(2000); // Example price
  //   const newPriceB = new BN(200); // Example price
  //   try {
  //     await program.methods
  //       .updatePrices(newPriceA, newPriceB)
  //       .accounts({
  //         vault: vaultPda,
  //         admin: wallet.publicKey,
  //         whirlpool: whirlpool,
  //       })
  //       .rpc();

  //     const vaultAccount = await program.account.vault.fetch(vaultPda);
  //     expect(vaultAccount.tokenAPrice.toString()).to.equal(
  //       newPriceA.toString()
  //     );
  //     expect(vaultAccount.tokenBPrice.toString()).to.equal(
  //       newPriceB.toString()
  //     );
  //   } catch (error) {
  //     console.error("Error:", error);
  //     throw error;
  //   }
  // });

  // it("Update ticks", async () => {
  //   const newLowerTick = -2000;
  //   const newUpperTick = 2000;

  //   try {
  //     await program.methods
  //       .updateTicks(newLowerTick, newUpperTick)
  //       .accounts({
  //         vault: vaultPda,
  //         admin: wallet.publicKey,
  //         whirlpool: whirlpool,
  //       })
  //       .rpc();

  //     const vaultAccount = await program.account.vault.fetch(vaultPda);
  //     expect(vaultAccount.lowerTick).to.equal(newLowerTick);
  //     expect(vaultAccount.upperTick).to.equal(newUpperTick);
  //   } catch (error) {
  //     console.error("Error:", error);
  //     throw error;
  //   }
  // });

  it("Deposits into vault", async () => {
    const depositSolAmount = 1 * LAMPORTS_PER_SOL; // 1 SOL
    const depositUSDCAmount = 10000000; // 10 USDC
    let depositTokenAAmount = 0;
    let depositTokenBAmount = 0;
    if (tokenSolMint.toBuffer().compare(tokenUSDCMint.toBuffer()) < 0) {
      depositTokenAAmount = depositSolAmount;
      depositTokenBAmount = depositUSDCAmount;
    } else {
      depositTokenAAmount = depositUSDCAmount;
      depositTokenBAmount = depositSolAmount;
    }

    try {
      adminShareAccount = await createAccount(
        provider.connection,
        admin,
        shareMint.publicKey,
        admin.publicKey
      );

      await program.methods
        .deposit(new BN(depositTokenAAmount), new BN(depositTokenBAmount))
        .accounts({
          vault: vaultPda,
          user: admin.publicKey,
          whirlpool: whirlpool,
          position: null,
          userTokenA: adminTokenAAccount,
          userTokenB: adminTokenBAccount,
          tokenAVault: tokenAVault.publicKey,
          tokenBVault: tokenBVault.publicKey,
          shareMint: shareMint.publicKey,
          userShare: adminShareAccount,
          tokenProgram: TOKEN_PROGRAM_ID,
        })
        .signers([admin])
        .rpc();

      // Verify the deposit
      const tokenAVaultBalance =
        await provider.connection.getTokenAccountBalance(tokenAVault.publicKey);
      const tokenBVaultBalance =
        await provider.connection.getTokenAccountBalance(tokenBVault.publicKey);

      expect(tokenAVaultBalance.value.amount).to.equal(
        depositTokenAAmount.toString()
      );
      expect(tokenBVaultBalance.value.amount).to.equal(
        depositTokenBAmount.toString()
      );
    } catch (err) {
      console.error("Error:", err);
      throw err;
    }
  });

  it("Withdraws from vault", async () => {
    try {
      const shareTokenBalanceBefore =
        await provider.connection.getTokenAccountBalance(adminShareAccount);

      await program.methods
        .withdraw(
          new BN(shareTokenBalanceBefore.value.amount),
          new BN(0),
          new BN(0)
        )
        .accounts({
          vault: vaultPda,
          user: admin.publicKey,
          whirlpool: whirlpool,
          position: null,
          userTokenA: adminTokenAAccount,
          userTokenB: adminTokenBAccount,
          tokenAVault: tokenAVault.publicKey,
          tokenBVault: tokenBVault.publicKey,
          shareMint: shareMint.publicKey,
          userShare: adminShareAccount,
          tokenProgram: TOKEN_PROGRAM_ID,
        })
        .signers([admin])
        .rpc();

      const tokenAVaultBalance =
        await provider.connection.getTokenAccountBalance(tokenAVault.publicKey);
      const tokenBVaultBalance =
        await provider.connection.getTokenAccountBalance(tokenBVault.publicKey);
      const shareTokenBalanceAfter =
        await provider.connection.getTokenAccountBalance(adminShareAccount);

      expect(tokenAVaultBalance.value.amount).to.equal("0");
      expect(tokenBVaultBalance.value.amount).to.equal("0");
      expect(shareTokenBalanceAfter.value.amount).to.equal("0");
    } catch (err) {
      console.error("Error:", err);
      throw err;
    }
  });

  // it("Fails to withdraw more than available balance", async () => {
  //   const withdrawAmount = 2 * LAMPORTS_PER_SOL; // 2 SOL (more than deposited)

  //   try {
  //     await program.methods
  //       .withdraw(new anchor.BN(withdrawAmount))
  //       .accounts({
  //         vault: vaultPDA,
  //         authority: provider.wallet.publicKey,
  //         systemProgram: SystemProgram.programId,
  //       })
  //       .rpc();

  //     // Should not reach here
  //     expect.fail("Expected withdrawal to fail");
  //   } catch (err) {
  //     expect(err.toString()).to.include("insufficient funds");
  //   }
  // });

  // it("Fails to withdraw with wrong authority", async () => {
  //   const wrongKeypair = anchor.web3.Keypair.generate();
  //   const withdrawAmount = 0.1 * LAMPORTS_PER_SOL;

  //   try {
  //     await program.methods
  //       .withdraw(new anchor.BN(withdrawAmount))
  //       .accounts({
  //         vault: vaultPDA,
  //         authority: wrongKeypair.publicKey,
  //         systemProgram: SystemProgram.programId,
  //       })
  //       .signers([wrongKeypair])
  //       .rpc();

  //     // Should not reach here
  //     expect.fail("Expected withdrawal to fail");
  //   } catch (err) {
  //     expect(err.toString()).to.include("unauthorized");
  //   }
  // });

  // it("Closes vault and returns funds", async () => {
  //   try {
  //     const balanceBefore = await provider.connection.getBalance(
  //       provider.wallet.publicKey
  //     );

  //     const tx = await program.methods
  //       .close()
  //       .accounts({
  //         vault: vaultPDA,
  //         authority: provider.wallet.publicKey,
  //         systemProgram: SystemProgram.programId,
  //       })
  //       .rpc();

  //     // Verify vault is closed
  //     const vaultAccount = await provider.connection.getAccountInfo(vaultPDA);
  //     expect(vaultAccount).to.be.null;

  //     const balanceAfter = await provider.connection.getBalance(
  //       provider.wallet.publicKey
  //     );

  //     // Account for transaction fees
  //     expect(balanceAfter).to.be.above(balanceBefore);
  //   } catch (err) {
  //     console.error("Error:", err);
  //     throw err;
  //   }
  // });
});
