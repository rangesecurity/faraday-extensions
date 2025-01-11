import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { SanctionedTransferHook } from "../target/types/sanctioned_transfer_hook";
import {
    PublicKey,
    SystemProgram,
    Transaction,
    sendAndConfirmTransaction,
    Keypair,
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
    NATIVE_MINT,
    TOKEN_PROGRAM_ID,
    getOrCreateAssociatedTokenAccount,
} from "@solana/spl-token";

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
    const destinationTokenAccount = getAssociatedTokenAddressSync(
        mint.publicKey,
        recipient.publicKey,
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

    // Sender wSOL token account address
    const senderWSolTokenAccount = getAssociatedTokenAddressSync(
        NATIVE_MINT, // mint
        wallet.publicKey // owner
    );

    // Delegate PDA wSOL token account address, to receive wSOL tokens from sender
    const delegateWSolTokenAccount = getAssociatedTokenAddressSync(
        NATIVE_MINT, // mint
        delegatePDA, // owner
        true // allowOwnerOffCurve
    );

    const [blockListPda] = PublicKey.findProgramAddressSync(
        [Buffer.from("block_list")],
        program.programId
    );

    // Create the two WSol token accounts as part of setup
    before(async () => {
        // WSol Token Account for sender
        await getOrCreateAssociatedTokenAccount(
            connection,
            wallet.payer,
            NATIVE_MINT,
            wallet.publicKey
        );

        // WSol Token Account for delegate PDA
        await getOrCreateAssociatedTokenAccount(
            connection,
            wallet.payer,
            NATIVE_MINT,
            delegatePDA,
            true
        );
    });

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
            { skipPreflight: true },
        );

        console.log(`Transaction Signature: ${txSig}`);
    });

    // Account to store extra accounts required by the transfer hook instruction
    it("Create ExtraAccountMetaList Account", async () => {
        const initializeExtraAccountMetaListInstruction = await program.methods
            .initializeExtraAccountMetaList()
            .accounts({
                payer: wallet.publicKey,
                extraAccountMetaList: extraAccountMetaListPDA,
                mint: mint.publicKey,
                wsolMint: NATIVE_MINT,
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
    });

    it("Transfer Hook with Extra Account Meta", async () => { });
});
