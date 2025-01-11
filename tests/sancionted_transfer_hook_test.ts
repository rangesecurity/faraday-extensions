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


    const [blockListPda] = PublicKey.findProgramAddressSync(
        [Buffer.from("block_list")],
        program.programId
    );
    console.log("extraAccountList", extraAccountMetaListPDA.toString());
    console.log("blockList", blockListPda.toString());
    // Create the two WSol token accounts as part of setup
    /*before(async () => {
        // WSol Token Account for sender

        const ix = createAssociatedTokenAccountInstruction(
            wallet.publicKey,
            senderTokenAccount,
            wallet.publicKey,
            NATIVE_MINT_2022,
            TOKEN_2022_PROGRAM_ID,
            ASSOCIATED_TOKEN_PROGRAM_ID,
        );
        const ix2 = createAssociatedTokenAccountInstruction(
            wallet.publicKey,
            delegateTokenAccount,
            delegatePDA,
            NATIVE_MINT_2022,
            TOKEN_2022_PROGRAM_ID,
            ASSOCIATED_TOKEN_PROGRAM_ID,
        );
        const tx = new Transaction().add(
            ix,
        ).add(ix2);
        await sendAndConfirmTransaction(
            connection,
            tx,
            [wallet.payer],
            {
                skipPreflight: false,
            }
        )
    });*/

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
            { skipPreflight: false },
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
        const extraAccountsInfo = await connection.getAccountInfoAndContext(
            extraAccountMetaListPDA,
        )
        //await new Promise((resolve) => setTimeout(resolve, 5000));

        console.log("extra account info", extraAccountsInfo);
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
        const mintInfo = await getMint(connection, mint.publicKey, "confirmed", TOKEN_2022_PROGRAM_ID);
        const transferHook = getTransferHook(
            mintInfo
        )
        const extraAccountsAccount = getExtraAccountMetaAddress(mint.publicKey, transferHook.programId);
        console.log("extra accounts key ", extraAccountsAccount.toString());
        const extraAccountsInfo = await connection.getAccountInfoAndContext(
            extraAccountsAccount,
        )
        console.log("extra account info", extraAccountsInfo);
        /*const ix = await createTransferCheckedWithTransferHookInstruction(
            connection,
            sourceTokenAccount,
            mint.publicKey,
            destinationTokenAccount,
            wallet.publicKey,
            BigInt(amount),
            decimals,
            [],
            "confirmed",
            TOKEN_2022_PROGRAM_ID,
        );*/
        const ix = await addExtraAccountsToInstruction(
            connection,
            transferInstruction,
            mint.publicKey,
            "confirmed",
            TOKEN_2022_PROGRAM_ID,
        );
        console.log(ix);

      
        // Automatic account resolution not working correctly for the WSol PDA
        // Manually add all the extra accounts required by the transfer hook instruction
        // Also include the address of the ExtraAccountMetaList account and Transfer Hook Program
        transferInstruction.keys.push(
          //{
          //  pubkey: mint.publicKey,
          //  isSigner: false,
          //  isWritable: false,
          //},
          //{
          //  pubkey: TOKEN_2022_PROGRAM_ID,
          //  isSigner: false,
          //  isWritable: false,
          //},
          //{
          //  pubkey: new PublicKey("ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL"),
          //  isSigner: false,
          //  isWritable: false,
          //},
          {
            pubkey: blockListPda,
            isSigner: false,
            isWritable: false
          },
          /*{
            pubkey: delegatePDA,
            isSigner: false,
            isWritable: false,
          },
          {
            pubkey: destinationTokenAccount,
            isSigner: false,
            isWritable: true,
          },
          {
            pubkey: sourceTokenAccount,
            isSigner: false,
            isWritable: true,
          },*/
          {
            pubkey: program.programId,
            isSigner: false,
            isWritable: false,
          },
          {
            pubkey: extraAccountMetaListPDA,
            isSigner: false,
            isWritable: false,
          },
        );
      
        console.log(transferInstruction);
        const transaction = new Transaction().add(
            ix,
        );
        try {
            const txSig = await sendAndConfirmTransaction(
                connection,
                transaction,
                [wallet.payer],
              );
              console.log("Transfer Signature:", txSig);
        } catch (error) {
            if (error instanceof SendTransactionError) {
                console.log("Error logs:", error.logs); // error.logs will show the program logs
                // You can check for specific error messages
                if (error.message.includes("invalid account data for instruction")) {
                    // Handle this specific error
                }
            }
            throw error; // Re-throw if you want the test to fail
        }

      });
});
