import {
    PublicKey,
    Connection,
    Signer,
    LAMPORTS_PER_SOL,
} from '@solana/web3.js';
import {
    createMint,
    getOrCreateAssociatedTokenAccount,
    Account as TokenAccount,
    mintTo,
    getAssociatedTokenAddress,
} from '@solana/spl-token';
import { Program } from "@project-serum/anchor";
import { TokenSale } from "../../target/types/token_sale";
import * as anchor from "@project-serum/anchor";
import {
    createUserWithLamports,
    createUserWithATA,
} from "./helpers";

// This interface is passed to every RPC test functions
export interface Ctx {
    connection: Connection,
    program: Program<TokenSale>,
    // The owner of the IDO program and tokens
    owner: Signer,
    // The mint of the tokens that are selling in the IDO
    sellingMint: PublicKey,
    // Owner's ATA with tokens for IDO
    tokensForDistribution: TokenAccount,
    buyingDuration: number,
    tradingDuration: number,
    initialTokenPrice: anchor.BN,
    amountForSale: anchor.BN,
    // The coefficients that define the value of the token in the next buying round
    // using the formula: nextTokenPrice = tokenPrice * coeffA + coeffB
    coeffA: number,
    coeffB: number,
    // pool ATA for storing the IDO tokens
    vaultSelling: PublicKey,
    traderFirst: CtxTrader,
    traderSecond: CtxTrader,
    traderThird: CtxTrader,
    // When the IDO starts (UNIX in seconds)
    roundStartAt: number,
    // When the IDO completes (UNIX in seconds)
    endAt: number,
    accounts: {
        pool: CtxAccountPDA,
    }
}

export interface CtxTrader {
    signer: Signer,
    ata: PublicKey,
}

export interface CtxAccount {
    key: PublicKey,
}

export interface CtxAccountPDA extends CtxAccount {
    bump: number,
}

export async function createCtx(connection: Connection, program: Program<TokenSale>): Promise<Ctx> {
    const owner = await createUserWithLamports(connection, 1);
    const sellingMint = await createMint(
        connection,
        owner, // payer
        owner.publicKey, // mintAuthority
        owner.publicKey, // freezeAuthority
        6 // decimals
    );
    const tokensForDistribution = await getOrCreateAssociatedTokenAccount(connection, owner, sellingMint, owner.publicKey);
    const tokensForDistributionAmount = 10_000;
    await mintTo(
        connection,
        owner,
        sellingMint,
        tokensForDistribution.address,
        owner,
        tokensForDistributionAmount,
    );
    const [poolPDA, poolBump] = await anchor.web3.PublicKey.findProgramAddress(
        [sellingMint.toBuffer()],
        program.programId
    );
    const vaultSelling = await getAssociatedTokenAddress(sellingMint, poolPDA, true);
    const [user1, ata1] = await createUserWithATA(connection, sellingMint);
    const [user2, ata2] = await createUserWithATA(connection, sellingMint);
    const [user3, ata3] = await createUserWithATA(connection, sellingMint);

    const now = Math.floor(Date.now() / 1000);

    return {
        connection,
        program,
        owner,
        sellingMint,
        tokensForDistribution,
        buyingDuration: 3,
        tradingDuration: 3,
        // 1 Token = 100_000_000 Lamports = 0.1 SOL
        initialTokenPrice: new anchor.BN(0.1 * LAMPORTS_PER_SOL),
        amountForSale: new anchor.BN(10_000),
        coeffA: 1.2,
        coeffB: 0.01 * LAMPORTS_PER_SOL,
        vaultSelling,
        traderFirst: {
            signer: user1,
            ata: ata1.address,
        },
        traderSecond: {
            signer: user2,
            ata: ata2.address,
        },
        traderThird: {
            signer: user3,
            ata: ata3.address,
        },
        roundStartAt: now + 1,
        endAt: now + 12,
        accounts: {
            pool: { key: poolPDA, bump: poolBump },
        }
    }
}
