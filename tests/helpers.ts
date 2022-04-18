import { 
    PublicKey, 
    SystemProgram, 
    LAMPORTS_PER_SOL, 
    Keypair,
    Connection,
    Signer,
} from '@solana/web3.js';
import { 
    createMint,
    TOKEN_PROGRAM_ID, 
    getOrCreateAssociatedTokenAccount,
    Account as TokenAccount,
    mintTo,
    getAccount,
    createApproveInstruction,
    NATIVE_MINT,
} from '@solana/spl-token';

export async function createUserWithLamports(
    connection: Connection, 
    lamports: number,
): Promise<Signer> {
    const account = Keypair.generate();
    const signature = await connection.requestAirdrop(
        account.publicKey, 
        lamports * LAMPORTS_PER_SOL
    );
    await connection.confirmTransaction(signature);
    return account;
}

export async function sleep(ms) {
    await new Promise((resolve) => setTimeout(resolve, ms));
}