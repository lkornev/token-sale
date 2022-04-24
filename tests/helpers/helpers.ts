import { 
    PublicKey,
    LAMPORTS_PER_SOL, 
    Keypair,
    Connection,
    Signer,
} from '@solana/web3.js';
import {
    getOrCreateAssociatedTokenAccount,
    Account as TokenAccount,
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

export async function createUserWithATA(
    connection: Connection,
    mint: PublicKey,
    lamports = 100
): Promise<[Signer, TokenAccount]> {
    let user = await createUserWithLamports(connection, lamports);
    let ata = await getOrCreateAssociatedTokenAccount(
        connection,
        user,
        mint,
        user.publicKey
    );

    return Promise.all([user, ata]);
}

export async function sleep(ms) {
    await new Promise((resolve) => setTimeout(resolve, ms));
}

export async function sleepTill(tillMs) {
    if (Date.now() < tillMs) {
        await sleep(tillMs - Date.now());
    }
}
