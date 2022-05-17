import { PublicKey, Signer, SystemProgram } from "@solana/web3.js";
import {
    ASSOCIATED_TOKEN_PROGRAM_ID,
    getOrCreateAssociatedTokenAccount,
    TOKEN_PROGRAM_ID,
    getAssociatedTokenAddress,
} from "@solana/spl-token";
import * as anchor from "@project-serum/anchor";
import { Ctx } from "./ctx";
import { sha256 } from "js-sha256";
import bs58 from 'bs58';

export namespace RPC {
    export async function initialize(ctx: Ctx) {
        await ctx.program.methods.initialize(
            ctx.roundStartAt,
            ctx.endAt,
            ctx.buyingDuration,
            ctx.tradingDuration,
            ctx.initialTokenPrice,
            { tokens: ctx.amountForSale },
            ctx.coeffA,
            ctx.coeffB,
        ).accounts({
            poolAccount: ctx.accounts.pool.key,
            distributionAuthority: ctx.owner.publicKey,
            tokensForDistribution: ctx.tokensForDistribution.address,
            sellingMint: ctx.sellingMint,
            vaultSelling: ctx.vaultSelling,
            systemProgram: SystemProgram.programId,
            tokenProgram: TOKEN_PROGRAM_ID,
            associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
            clock: anchor.web3.SYSVAR_CLOCK_PUBKEY,
        }).signers([ctx.owner]).rpc();
    }

    export async function buyTokens(ctx: Ctx, trader: Signer, tokensAmount: anchor.BN) {
        const ata = await getOrCreateAssociatedTokenAccount(ctx.connection, trader, ctx.sellingMint, trader.publicKey);
        await ctx.program.methods.buy(
            { tokens: tokensAmount },
        ).accounts({
            poolAccount: ctx.accounts.pool.key,
            sellingMint: ctx.sellingMint,
            vaultSelling: ctx.vaultSelling,
            buyer: trader.publicKey,
            buyerTokenAccount: ata.address,
            systemProgram: SystemProgram.programId,
            tokenProgram: TOKEN_PROGRAM_ID,
            clock: anchor.web3.SYSVAR_CLOCK_PUBKEY,
        }).signers([trader]).rpc();
    }

    export async function switchToTrading(ctx: Ctx) {
        await ctx.program.methods.switchToTrading()
            .accounts({
                poolAccount: ctx.accounts.pool.key,
                clock: anchor.web3.SYSVAR_CLOCK_PUBKEY,
            })
            .rpc();
    }

    export async function placeOrder(
        ctx: Ctx,
        seller: Signer,
        amountToSell: anchor.BN,
        priceForToken: anchor.BN
    ): Promise<Order> {
        const [orderPDA, orderBump] = await anchor.web3.PublicKey.findProgramAddress(
            [
                anchor.utils.bytes.utf8.encode("order"),
                seller.publicKey.toBuffer(),
            ],
            ctx.program.programId
        );

        const orderTokenVault = await getAssociatedTokenAddress(ctx.sellingMint, orderPDA, true);

        const sellerTokenAccount = await getOrCreateAssociatedTokenAccount(
            ctx.connection,
            seller,
            ctx.sellingMint,
            seller.publicKey,
        );

        await ctx.program.methods
            .placeOrder(
                { tokens: amountToSell },
                priceForToken,
            )
            .accounts({
                poolAccount: ctx.accounts.pool.key,
                sellingMint: ctx.sellingMint,
                seller: seller.publicKey,
                sellerTokenAccount: sellerTokenAccount.address,
                order: orderPDA,
                orderTokenVault,
                systemProgram: SystemProgram.programId,
                tokenProgram: TOKEN_PROGRAM_ID,
                associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
                clock: anchor.web3.SYSVAR_CLOCK_PUBKEY,
            })
            .signers([seller])
            .rpc();

        return Promise.resolve({
            address: orderPDA,
            bump: orderBump,
            tokenVault: orderTokenVault,
            owner: seller.publicKey,
            tokenPrice: priceForToken,
            tokenAmount: { tokens: amountToSell },
        });
    }

    // Get orders, sorted by the creating time in the descending order
    export async function getOrders(ctx: Ctx, cfg: GetOrderConfig = DEFAULT_GET_ORDER_CONFIG): Promise<Order[]> {
        const orderDiscriminator = Buffer.from(sha256.digest('account:Order')).slice(0, 8);

        const filters = [
            { memcmp: { offset: 0, bytes: bs58.encode(orderDiscriminator) } }, // Ensure it's a Order account.
        ];

        if (cfg.status !== 'all') {
            const isEmpty: 0 | 1 = cfg.status === 'withTokens' ? 0 : 1;
            // Filter orders by is_empty field
            filters.push({ memcmp: { offset: 8, bytes: bs58.encode(Uint8Array.from([isEmpty])) } });
        }

        if (cfg.owner) {
            // Filter orders by owner field
            filters.push({ memcmp: { offset: 8 + 1 + 8, bytes: cfg.owner.toBase58() } });
        }

        const orders = (await ctx.connection.getProgramAccounts(ctx.program.programId, {
            dataSlice: { offset: 8 + 1, length: 8 }, // Fetch the created_at only.
            filters,
        }));

        const ordersSortedByDate = orders.map(({ pubkey, account }) => ({
            pubkey,
            createdAt: new anchor.BN(account.data, 'le'),
        })).sort((a, b) => b.createdAt.cmp(a.createdAt));

        const paginatedOrders = ordersSortedByDate.slice((cfg.page - 1) * cfg.perPage, cfg.page * cfg.perPage);

        if (paginatedOrders.length === 0) {
            return [];
        }

        const paginatedOrdersKeys = paginatedOrders.map(({ pubkey }) => pubkey);
        const ordersWithData: Order[] = (await ctx.program.account.order.fetchMultiple(paginatedOrdersKeys))
            .map((order: any, i) => {
                return {
                    address: paginatedOrders[i].pubkey,
                    bump: order.bump,
                    tokenVault: order.tokenVault,
                    owner: order.owner,
                    tokenPrice: order.tokenPrice,
                    tokenAmount: order.tokenAmount,
                };
            });

        return ordersWithData;
    }

    export async function redeemOrder(ctx: Ctx, orderAddress: PublicKey, buyer: Signer, amountToBuy: anchor.BN) {
        const order = await ctx.program.account.order.fetch(orderAddress);
        const buyerTokenAccount: PublicKey = await getAssociatedTokenAddress(ctx.sellingMint, buyer.publicKey);

        await ctx.program.methods.redeemOrder({ tokens: amountToBuy })
            .accounts({
                poolAccount: ctx.accounts.pool.key,
                sellingMint: ctx.sellingMint,
                buyer: buyer.publicKey,
                buyerTokenAccount,
                order: orderAddress,
                orderOwner: order.owner,
                orderTokenVault: order.tokenVault,
                tokenProgram: TOKEN_PROGRAM_ID,
                clock: anchor.web3.SYSVAR_CLOCK_PUBKEY,
                systemProgram: SystemProgram.programId,
            })
            .signers([buyer])
            .rpc();
    }

    export async function closeOrder(
        ctx: Ctx,
        orderAddress: PublicKey,
        orderTokenVault: PublicKey,
        ownerSigner: Signer,
        ownerTokenAccount: PublicKey,
    ) {
        await ctx.program.methods.closeOrder()
            .accounts({
                poolAccount: ctx.accounts.pool.key,
                sellingMint: ctx.sellingMint,
                order: orderAddress,
                orderTokenVault: orderTokenVault,
                orderOwner: ownerSigner.publicKey,
                ownerTokenVault: ownerTokenAccount,
            })
            .signers([ownerSigner])
            .rpc();
    }

    export async function switchToBuying(ctx: Ctx) {
        await ctx.program.methods.switchToBuying()
            .accounts({
                poolAccount: ctx.accounts.pool.key,
                clock: anchor.web3.SYSVAR_CLOCK_PUBKEY,
            })
            .rpc();
    }

    export async function withdrawLamports(ctx: Ctx) {
        await ctx.program.methods.withdrawLamports()
            .accounts({
                poolAccount: ctx.accounts.pool.key,
                sellingMint: ctx.sellingMint,
                owner: ctx.owner.publicKey,
                tokenProgram: TOKEN_PROGRAM_ID,
                systemProgram: SystemProgram.programId,
                clock: anchor.web3.SYSVAR_CLOCK_PUBKEY,
            })
            .signers([ctx.owner])
            .rpc();
    }

    export async function terminate(ctx: Ctx) {
        await ctx.program.methods.terminate()
            .accounts({
                poolAccount: ctx.accounts.pool.key,
                sellingMint: ctx.sellingMint,
                vaultSelling: ctx.vaultSelling,
                owner: ctx.owner.publicKey,
                tokenProgram: TOKEN_PROGRAM_ID,
                systemProgram: SystemProgram.programId,
                clock: anchor.web3.SYSVAR_CLOCK_PUBKEY,
            })
            .signers([ctx.owner])
            .rpc();
    }
}

export interface Order {
    address: PublicKey,
    bump: number,
    tokenVault: PublicKey,
    owner: PublicKey,
    tokenPrice: anchor.BN,
    tokenAmount: { tokens: anchor.BN },
}

export interface GetOrderConfig {
    status: 'all' | 'empty' | 'withTokens',
    page: number,
    perPage: number,
    owner?: PublicKey,
}

const DEFAULT_GET_ORDER_CONFIG: GetOrderConfig = {
    status: 'withTokens',
    page: 1, 
    perPage: 100,
}
