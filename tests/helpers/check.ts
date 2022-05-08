import * as anchor from "@project-serum/anchor";
import { Connection, PublicKey } from "@solana/web3.js";
import { expect } from "chai";
import { Ctx } from "./ctx";
import {
    getAccount as getTokenAccount,
    getAssociatedTokenAddress,
    Account as TokenAccount, getMinimumBalanceForRentExemptAccount,
} from '@solana/spl-token';
import { Round } from "../types/round";
import { Order, RPC } from "./rpc";

type Balance = number | anchor.BN | bigint;

export namespace CheckCtx {
    export async function lamportsBalance(ctx: Ctx, key: PublicKey, balanceBefore: Balance, addedBalance: Balance) {
        await Check.lamportsBalance(ctx.connection, key, Number(balanceBefore) + Number(addedBalance));
    }

    export async function tokenBalance(ctx: Ctx, key: PublicKey, balanceBefore: Balance, addedBalance: Balance, msg?: string) {
        await Check.tokenBalance(ctx.connection, key, Number(balanceBefore) + Number(addedBalance), msg);
    }

    export async function currentRound(ctx: Ctx, round: object, startedAtMs?: number) {
        const pool = await ctx.program.account.poolAccount.fetch(ctx.accounts.pool.key);
        expect(`${pool.currentRound}`).to.be.eq(`${round}`);

        if (startedAtMs) {
            expect(Number(pool.roundStartAt) >= startedAtMs - 1000 || Number(pool.roundStartAt) <= startedAtMs + 1000,
                "The round started near the current time"
            ).to.be.true;
        }
    }

    export async function order(ctx: Ctx, orderKey: PublicKey, expectedOrder: Order) {
        const order = await ctx.program.account.order.fetch(orderKey);
        expect(`${order.bump}`).to.be.eq(`${expectedOrder.bump}`);
        expect(`${order.owner}`).to.be.eq(`${expectedOrder.owner}`);
        expect(`${order.tokenVault}`).to.be.eq(`${expectedOrder.tokenVault}`);
        expect(`${order.tokenPrice}`).to.be.eq(`${expectedOrder.tokenPrice}`);
        expect(`${order.tokenAmount.tokens}`).to.be.eq(`${expectedOrder.tokenAmount.tokens}`);

        await tokenBalance(ctx, expectedOrder.tokenVault, 0, expectedOrder.tokenAmount.tokens);
    }

    export async function redeemedOrder(
        ctx: Ctx,
        orderAddress: PublicKey,
        buyer: PublicKey,
        balanceBefore: Balance,
        expectedRedeemedBalance: Balance
    ) {
        const order = await ctx.program.account.order.fetch(orderAddress);
        await CheckCtx.tokenBalance(ctx, orderAddress, balanceBefore, -expectedRedeemedBalance, 'of order');
        await CheckCtx.tokenBalance(ctx, order.tokenVault, balanceBefore, -expectedRedeemedBalance, 'of order token vault');

        const buyerAta = await getAssociatedTokenAddress(ctx.sellingMint, buyer);
        await CheckCtx.tokenBalance(ctx, buyerAta, 0, expectedRedeemedBalance, 'of buyer ATA');
    }

    export async function closeOrder(
        ctx: Ctx,
        orderKey: PublicKey,
        orderTokensAmount: Balance,
        orderDataLength: number,
        ownerKey: PublicKey,
        ownerTokensBefore: Balance,
        ownerLamportsBefore: Balance,
    ) {
        const orderATA = await getAssociatedTokenAddress(ctx.sellingMint, orderKey, true);
        const ownerATA = await getAssociatedTokenAddress(ctx.sellingMint, ownerKey);

        // Tokens moved from orderFirst.tokenVault to ctx.traderFirst.ata
        await CheckCtx.tokenBalance(ctx, orderATA, orderTokensAmount, -orderTokensAmount);
        await CheckCtx.tokenBalance(ctx, ownerATA, ownerTokensBefore, orderTokensAmount);

        // The rent for order account and the order's ata is returned to the order's owner.
        const ownerLamportsAfter = (await ctx.connection.getAccountInfo(ownerKey)).lamports;
        const rentForOrder = await ctx.connection.getMinimumBalanceForRentExemption(orderDataLength);
        const rentForTokenVault = await getMinimumBalanceForRentExemptAccount(ctx.connection);
        expect(Number(ownerLamportsBefore) + rentForOrder + rentForTokenVault).to.be.eq(ownerLamportsAfter);
    }

    export async function poolInitialState(ctx: Ctx) {
        const pool = await ctx.program.account.poolAccount.fetch(ctx.accounts.pool.key);
        expect(`${pool.owner}`).to.be.eq(`${ctx.owner.publicKey}`);
        expect(`${pool.sellingMint}`).to.be.eq(`${ctx.sellingMint}`);
        expect(`${pool.bump}`).to.be.eq(`${ctx.accounts.pool.bump}`);
        expect(`${pool.vaultSelling}`).to.be.eq(`${ctx.vaultSelling}`);
        expect(`${pool.currentRound}`).to.be.eq(`${Round.Buying}`);
        expect(`${pool.roundStartAt}`).to.be.eq(`${ctx.roundStartAt}`);
        expect(`${pool.endAt}`).to.be.eq(`${ctx.endAt}`);
        expect(`${pool.buyingDuration}`).to.be.eq(`${ctx.buyingDuration}`);
        expect(`${pool.tradingDuration}`).to.be.eq(`${ctx.tradingDuration}`);
        expect(`${pool.tokenPrice}`).to.be.eq(`${ctx.initialTokenPrice}`);
    }
}

export namespace Check {
    export async function lamportsBalance(connection: Connection, account: PublicKey, expectedBalance: number, msg?: String) {
        let info = await connection.getAccountInfo(account);
        let message = "Lamports balance";
        if (msg) { message += ` of ${msg}` }
        expect(`${info.lamports}`, message).to.be.eq(`${expectedBalance}`);
    }

    export async function tokenBalance(connection: Connection, key: PublicKey, expectedBalance: number, msg?: string) {
        let acc: TokenAccount | null = await getTokenAccount(connection, key).catch(() => null);

        if (acc) {
            expect(`${acc.amount}`, `Token balance ${msg}`).to.be.eq(`${expectedBalance}`);
        } else {
            // Account does not exist, so it has zero tokens ;)
            expect(`${0}`, `Token balance (acc not found) ${msg}`).to.be.eq(`${expectedBalance}`);
        }
    }
}
