import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { SanctionedTransferHook } from "../target/types/sanctioned_transfer_hook";
import {
    PublicKey,
    SystemProgram,
    Transaction,
    sendAndConfirmTransaction,
    Keypair,
    SendTransactionError,
} from "@solana/web3.js";
import {
    ExtensionType,
    TOKEN_2022_PROGRAM_ID,
    getMintLen,
    createInitializeMintInstruction,
    createInitializeTransferHookInstruction,
    ASSOCIATED_TOKEN_PROGRAM_ID,
    createAssociatedTokenAccountInstruction,
    createMintToInstruction,
    createTransferCheckedInstruction,
    getAssociatedTokenAddressSync,
    createApproveInstruction,
    createSyncNativeInstruction,
    TOKEN_PROGRAM_ID,
    getOrCreateAssociatedTokenAccount,
    addExtraAccountsToInstruction,
    createTransferCheckedWithTransferHookInstruction,
    getTransferHook,
    getMint,
    getExtraAccountMetaAddress,
} from "@solana/spl-token";
import { assert, expect } from "chai";

describe("transfer-hook", () => {
    // Configure the client to use the local cluster.
    const provider = anchor.AnchorProvider.env();
    anchor.setProvider(provider);

    const program = anchor.workspace.SanctionedTransferHook as Program<SanctionedTransferHook>;
    const wallet = provider.wallet as anchor.Wallet;
    const connection = provider.connection;

    // Generate keypair to use as address for the transfer-hook enabled mint
    const mint = new Keypair();
    const decimals = 9;


    // Sender token account address
    const sourceTokenAccount = getAssociatedTokenAddressSync(
        mint.publicKey,
        wallet.publicKey,
        false,
        TOKEN_2022_PROGRAM_ID,
        ASSOCIATED_TOKEN_PROGRAM_ID
    );

    // Recipient token account address
    const recipient = Keypair.generate();
    const recipient2 = Keypair.generate();
    const destinationTokenAccount = getAssociatedTokenAddressSync(
        mint.publicKey,
        recipient.publicKey,
        false,
        TOKEN_2022_PROGRAM_ID,
        ASSOCIATED_TOKEN_PROGRAM_ID
    );
    const destination2TokenAccount = getAssociatedTokenAddressSync(
        mint.publicKey,
        recipient2.publicKey,
        false,
        TOKEN_2022_PROGRAM_ID,
        ASSOCIATED_TOKEN_PROGRAM_ID
    );

    // ExtraAccountMetaList address
    // Store extra accounts required by the custom transfer hook instruction
    const [extraAccountMetaListPDA] = PublicKey.findProgramAddressSync(
        [Buffer.from("extra-account-metas"), mint.publicKey.toBuffer()],
        program.programId
    );

    // PDA delegate to transfer wSOL tokens from sender
    const [delegatePDA] = PublicKey.findProgramAddressSync(
        [Buffer.from("delegate")],
        program.programId
    );


    const [blockListPda] = PublicKey.findProgramAddressSync(
        [Buffer.from("block_list")],
        program.programId
    );

    it("Create Mint Account with Transfer Hook Extension", async () => {
        const extensions = [ExtensionType.TransferHook];
        const mintLen = getMintLen(extensions);
        const lamports =
            await provider.connection.getMinimumBalanceForRentExemption(mintLen);

        const transaction = new Transaction().add(
            SystemProgram.createAccount({
                fromPubkey: wallet.publicKey,
                newAccountPubkey: mint.publicKey,
                space: mintLen,
                lamports: lamports,
                programId: TOKEN_2022_PROGRAM_ID,
            }),
            createInitializeTransferHookInstruction(
                mint.publicKey,
                wallet.publicKey,
                program.programId, // Transfer Hook Program ID
                TOKEN_2022_PROGRAM_ID,
            ),
            createInitializeMintInstruction(
                mint.publicKey,
                decimals,
                wallet.publicKey,
                null,
                TOKEN_2022_PROGRAM_ID,
            ),
        );

        const txSig = await sendAndConfirmTransaction(
            provider.connection,
            transaction,
            [wallet.payer, mint],
        );
        console.log(`Transaction Signature: ${txSig}`);
        await new Promise((resolve) => setTimeout(resolve, 1000));

    });
    it("Creates Block List", async () => {
        const ix = await program.methods
            .initialize()
            .accounts({
                authority: wallet.publicKey,
                blockList: blockListPda,
                systemProgram: SystemProgram.programId
            })
            .instruction();
        const transaction = new Transaction().add(ix);
        const txSig = await sendAndConfirmTransaction(
            provider.connection,
            transaction,
            [wallet.payer],
        );
        console.log("Transaction Signature:", txSig);
        await new Promise((resolve) => setTimeout(resolve, 1000));

    })


    it("Create Token Accounts and Mint Tokens", async () => {
        // 100 tokens
        const amount = 100 * 10 ** decimals;

        const transaction = new Transaction().add(
            createAssociatedTokenAccountInstruction(
                wallet.publicKey,
                sourceTokenAccount,
                wallet.publicKey,
                mint.publicKey,
                TOKEN_2022_PROGRAM_ID,
                ASSOCIATED_TOKEN_PROGRAM_ID,
            ),
            createAssociatedTokenAccountInstruction(
                wallet.publicKey,
                destinationTokenAccount,
                recipient.publicKey,
                mint.publicKey,
                TOKEN_2022_PROGRAM_ID,
                ASSOCIATED_TOKEN_PROGRAM_ID,
            ),
            createAssociatedTokenAccountInstruction(
                wallet.publicKey,
                destination2TokenAccount,
                recipient2.publicKey,
                mint.publicKey,
                TOKEN_2022_PROGRAM_ID,
                ASSOCIATED_TOKEN_PROGRAM_ID,
            ),
            createMintToInstruction(
                mint.publicKey,
                sourceTokenAccount,
                wallet.publicKey,
                amount,
                [],
                TOKEN_2022_PROGRAM_ID,
            ),
        );

        const txSig = await sendAndConfirmTransaction(
            connection,
            transaction,
            [wallet.payer],
            { skipPreflight: false },
        );

        console.log(`Transaction Signature: ${txSig}`);
        await new Promise((resolve) => setTimeout(resolve, 1000));

    });

    // Account to store extra accounts required by the transfer hook instruction
    it("Create ExtraAccountMetaList Account", async () => {
        const initializeExtraAccountMetaListInstruction = await program.methods
            .initializeExtraAccountMetaList()
            .accounts({
                payer: wallet.publicKey,
                extraAccountMetaList: extraAccountMetaListPDA,
                mint: mint.publicKey,
                tokenProgram: TOKEN_2022_PROGRAM_ID,
                associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
                blockList: blockListPda,
            })
            .instruction();

        const transaction = new Transaction().add(
            initializeExtraAccountMetaListInstruction,
        );

        const txSig = await sendAndConfirmTransaction(
            provider.connection,
            transaction,
            [wallet.payer],
        );
        console.log("Transaction Signature:", txSig);
        await new Promise((resolve) => setTimeout(resolve, 1000));

    });

    it("Transfer Hook with Extra Account Meta", async () => {
        // 1 tokens
        const amount = 1 * 10 ** decimals;


        // Standard token transfer instruction
        const transferInstruction = createTransferCheckedInstruction(
            sourceTokenAccount,
            mint.publicKey,
            destinationTokenAccount,
            wallet.publicKey,
            amount,
            decimals,
            [],
            TOKEN_2022_PROGRAM_ID,
        );
        const ix = await addExtraAccountsToInstruction(
            connection,
            transferInstruction,
            mint.publicKey,
            "confirmed",
            TOKEN_2022_PROGRAM_ID,
        );


        const transaction = new Transaction().add(
            ix,
        );
        const txSig = await sendAndConfirmTransaction(
            connection,
            transaction,
            [wallet.payer],
        );
        console.log("Transfer Signature:", txSig);
        await new Promise((resolve) => setTimeout(resolve, 1000));


    });
    it("Adds destination owner to deny list", async () => {
        const ix = await program.methods
            .addToBlockList([
                recipient.publicKey
            ])
            .accounts({
                authority: wallet.publicKey,
                blockList: blockListPda,
            }).instruction();
        const tx = new Transaction().add(
            ix
        );
        const txSig = await sendAndConfirmTransaction(
            provider.connection,
            tx,
            [wallet.payer]
        );
        console.log("Transaction signature:", txSig);
        await new Promise((resolve) => setTimeout(resolve, 1000));

    });
    it("Fails to send to destination owner", async () => {
        // 1 tokens
        const amount = 1 * 10 ** decimals;


        // Standard token transfer instruction
        const transferInstruction = createTransferCheckedInstruction(
            sourceTokenAccount,
            mint.publicKey,
            destinationTokenAccount,
            wallet.publicKey,
            amount,
            decimals,
            [],
            TOKEN_2022_PROGRAM_ID,
        );
        const ix = await addExtraAccountsToInstruction(
            connection,
            transferInstruction,
            mint.publicKey,
            "confirmed",
            TOKEN_2022_PROGRAM_ID,
        );


        const transaction = new Transaction().add(
            ix,
        );
        try {
            const txSig = await sendAndConfirmTransaction(
                connection,
                transaction,
                [wallet.payer],
            );
            // If we get here, the transaction succeeded when it shouldn't have
            assert.fail("Transaction should have failed");
        } catch (error) {
            // Verify it's the right type of error
            expect(error).to.be.instanceOf(SendTransactionError);
        }
        await new Promise((resolve) => setTimeout(resolve, 1000));

    });
    it("Removes destination owner from deny list", async () => {
        const ix = await program.methods
            .removeFromBlockList([
                recipient.publicKey
            ])
            .accounts({
                authority: wallet.publicKey,
                blockList: blockListPda,
            }).instruction();
        const tx = new Transaction().add(
            ix
        );
        const txSig = await sendAndConfirmTransaction(
            provider.connection,
            tx,
            [wallet.payer]
        );
        console.log("Transaction signature:", txSig);
        await new Promise((resolve) => setTimeout(resolve, 1000));
    });
    it("Can send to destination owner", async () => {
        // 1 tokens
        const amount = 1 * 10 ** decimals;


        // Standard token transfer instruction
        const transferInstruction = createTransferCheckedInstruction(
            sourceTokenAccount,
            mint.publicKey,
            destinationTokenAccount,
            wallet.publicKey,
            amount,
            decimals,
            [],
            TOKEN_2022_PROGRAM_ID,
        );
        const ix = await addExtraAccountsToInstruction(
            connection,
            transferInstruction,
            mint.publicKey,
            "confirmed",
            TOKEN_2022_PROGRAM_ID,
        );


        const transaction = new Transaction().add(
            ix,
        );
        const txSig = await sendAndConfirmTransaction(
            connection,
            transaction,
            [wallet.payer],
        );
        console.log("Transaction signature:", txSig);

        await new Promise((resolve) => setTimeout(resolve, 1000));
    });
    it("Adds source owner to deny list", async () => {
        const ix = await program.methods
            .addToBlockList([
                wallet.publicKey
            ])
            .accounts({
                authority: wallet.publicKey,
                blockList: blockListPda,
            }).instruction();
        const tx = new Transaction().add(
            ix
        );
        const txSig = await sendAndConfirmTransaction(
            provider.connection,
            tx,
            [wallet.payer]
        );
        console.log("Transaction signature:", txSig);
        await new Promise((resolve) => setTimeout(resolve, 1000));
    });
    it("Fails to send from test wallet", async () => {
        // 1 tokens
        const amount = 1 * 10 ** decimals;


        // Standard token transfer instruction
        const transferInstruction = createTransferCheckedInstruction(
            sourceTokenAccount,
            mint.publicKey,
            destinationTokenAccount,
            wallet.publicKey,
            amount,
            decimals,
            [],
            TOKEN_2022_PROGRAM_ID,
        );
        const ix = await addExtraAccountsToInstruction(
            connection,
            transferInstruction,
            mint.publicKey,
            "confirmed",
            TOKEN_2022_PROGRAM_ID,
        );


        const transaction = new Transaction().add(
            ix,
        );
        try {
            const txSig = await sendAndConfirmTransaction(
                connection,
                transaction,
                [wallet.payer],
            );
            // If we get here, the transaction succeeded when it shouldn't have
            assert.fail("Transaction should have failed");
        } catch (error) {
            // Verify it's the right type of error
            expect(error).to.be.instanceOf(SendTransactionError);
        }
        await new Promise((resolve) => setTimeout(resolve, 1000));
    });
    it("Adds destination to deny list", async () => {
        const ix = await program.methods
            .addToBlockList([
                recipient.publicKey
            ])
            .accounts({
                authority: wallet.publicKey,
                blockList: blockListPda,
            }).instruction();
        const tx = new Transaction().add(
            ix
        );
        const txSig = await sendAndConfirmTransaction(
            provider.connection,
            tx,
            [wallet.payer]
        );
        console.log("Transaction signature:", txSig);
        await new Promise((resolve) => setTimeout(resolve, 1000));
    });
    it("Removes source from deny list", async () => {
        const ix = await program.methods
            .removeFromBlockList([
                wallet.publicKey
            ])
            .accounts({
                authority: wallet.publicKey,
                blockList: blockListPda,
            }).instruction();
        const tx = new Transaction().add(
            ix
        );
        const txSig = await sendAndConfirmTransaction(
            provider.connection,
            tx,
            [wallet.payer]
        );
        console.log("Transaction signature:", txSig);
        await new Promise((resolve) => setTimeout(resolve, 1000));
    });
    it("Fails to send to destination", async () => {
        // 1 tokens
        const amount = 1 * 10 ** decimals;


        // Standard token transfer instruction
        const transferInstruction = createTransferCheckedInstruction(
            sourceTokenAccount,
            mint.publicKey,
            destinationTokenAccount,
            wallet.publicKey,
            amount,
            decimals,
            [],
            TOKEN_2022_PROGRAM_ID,
        );
        const ix = await addExtraAccountsToInstruction(
            connection,
            transferInstruction,
            mint.publicKey,
            "confirmed",
            TOKEN_2022_PROGRAM_ID,
        );


        const transaction = new Transaction().add(
            ix,
        );
        try {
            const txSig = await sendAndConfirmTransaction(
                connection,
                transaction,
                [wallet.payer],
            );
            // If we get here, the transaction succeeded when it shouldn't have
            assert.fail("Transaction should have failed");
        } catch (error) {
            // Verify it's the right type of error
            expect(error).to.be.instanceOf(SendTransactionError);
        }
        await new Promise((resolve) => setTimeout(resolve, 1000));
    });
    it("Can send to destination2 owner", async () => {
        // 1 tokens
        const amount = 1 * 10 ** decimals;


        // Standard token transfer instruction
        const transferInstruction = createTransferCheckedInstruction(
            sourceTokenAccount,
            mint.publicKey,
            destination2TokenAccount,
            wallet.publicKey,
            amount,
            decimals,
            [],
            TOKEN_2022_PROGRAM_ID,
        );
        const ix = await addExtraAccountsToInstruction(
            connection,
            transferInstruction,
            mint.publicKey,
            "confirmed",
            TOKEN_2022_PROGRAM_ID,
        );


        const transaction = new Transaction().add(
            ix,
        );
        const txSig = await sendAndConfirmTransaction(
            connection,
            transaction,
            [wallet.payer],
        );
        console.log("Transaction signature:", txSig);

        await new Promise((resolve) => setTimeout(resolve, 1000));
    });
});
