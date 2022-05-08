import * as anchor from "@project-serum/anchor";
import { Program } from "@project-serum/anchor";
import { TokenSale } from "../target/types/token_sale";
import { sleepTill } from "./helpers/helpers";
import { Connection, LAMPORTS_PER_SOL } from '@solana/web3.js';
import { Round } from "./types/round";
import { createCtx, Ctx } from "./helpers/ctx";
import { RPC } from "./helpers/rpc";
import { CheckCtx} from "./helpers/check";
import { expect } from "chai";
import { getAccount as getTokenAccount, getMint } from "@solana/spl-token";
import { Order } from './helpers/rpc';

describe("happy-path", () => {
    // Configure the client to use the local cluster.
    anchor.setProvider(anchor.AnchorProvider.env());

    const program = anchor.workspace.TokenSale as Program<TokenSale>;
    const connection = new Connection("http://localhost:8899", 'recent');
    // The config that passed to every RPC test functions
    let ctx: Ctx;

    it("Initializes!", async () => {
        ctx = await createCtx(connection, program);
        await RPC.initialize(ctx);
        await CheckCtx.poolInitialState(ctx);
    });

    it("Buys tokens from the program", async () => {
        const firstTraderBuy = new anchor.BN(4),
            secondTraderBuy = new anchor.BN(10);
        const poolBalanceBefore = (await connection.getAccountInfo(ctx.accounts.pool.key)).lamports;

        await RPC.buyTokens(ctx, ctx.traderFirst.signer, firstTraderBuy);
        await RPC.buyTokens(ctx, ctx.traderSecond.signer, secondTraderBuy);

        // The amount of lamports the program has to have after the first sales
        const expectedAddedPoolBalance = ((firstTraderBuy.add(secondTraderBuy))
            .mul(new anchor.BN(ctx.initialTokenPrice)));
        await CheckCtx.lamportsBalance(ctx, ctx.accounts.pool.key, poolBalanceBefore, expectedAddedPoolBalance);

        // Traders should receive their tokens
        await CheckCtx.tokenBalance(ctx, ctx.traderFirst.ata, 0, firstTraderBuy);
        await CheckCtx.tokenBalance(ctx, ctx.traderSecond.ata, 0, secondTraderBuy);
    });

    it("Switches to the trading round", async () => {
        const currentRoundEndsAtMs = (Number(ctx.roundStartAt) + ctx.buyingDuration) * 1000;
        await sleepTill(currentRoundEndsAtMs);
        await RPC.switchToTrading(ctx);
        await CheckCtx.currentRound(ctx, Round.Trading, Date.now());
    });

    it("Places orders for selling tokens", async () => {
        const amountToSellFirst = new anchor.BN(2),
            priceForTokenFirst = new anchor.BN(0.12 * LAMPORTS_PER_SOL),
            amountToSellSecond = new anchor.BN(5),
            priceForTokenSecond = new anchor.BN(0.13 * LAMPORTS_PER_SOL);

        const firstTraderBalanceBefore = await getTokenAccount(connection, ctx.traderFirst.ata);
        const secondTraderBalanceBefore = await getTokenAccount(connection, ctx.traderSecond.ata);

        const placedOrderFirst = await RPC.placeOrder(ctx, ctx.traderFirst.signer, amountToSellFirst, priceForTokenFirst);
        await CheckCtx.order(ctx, placedOrderFirst.address, placedOrderFirst);
        await CheckCtx.tokenBalance(ctx, ctx.traderFirst.ata, firstTraderBalanceBefore.amount, -amountToSellFirst);

        const placedOrderSecond = await RPC.placeOrder(ctx, ctx.traderSecond.signer, amountToSellSecond, priceForTokenSecond);
        await CheckCtx.order(ctx, placedOrderSecond.address, placedOrderSecond);
        await CheckCtx.tokenBalance(ctx, ctx.traderSecond.ata, secondTraderBalanceBefore.amount, -amountToSellSecond);

        const orders = await RPC.getOrders(ctx);
        expect(orders.length).to.be.eq(2);
    });

    it("Buys tokens from other traders", async () => {
        const orders = await RPC.getOrders(ctx);
        const orderAddress = orders[0].address;
        const orderBefore = await program.account.order.fetch(orderAddress);
        const orderOwnerAccountBefore = await connection.getAccountInfo(orderBefore.owner);
        const orderTokens = orderBefore.tokenAmount.tokens;
        const isTokenAmountEven = (Number(orderTokens) % 2) === 0;
        const halfOfAllTokens = orderTokens.div(new anchor.BN(2));

        await RPC.redeemOrder(ctx, orderAddress, ctx.traderThird.signer, halfOfAllTokens);
        await RPC.redeemOrder(ctx, orderAddress, ctx.traderThird.signer, halfOfAllTokens.add(new anchor.BN(isTokenAmountEven ? 0 : 1)));

        await CheckCtx.redeemedOrder(ctx, orderAddress, ctx.traderThird.signer.publicKey, orderTokens, orderTokens);

        const expectedLamportsIncome = ctx.initialTokenPrice.mul(orderTokens);
        await CheckCtx.lamportsBalance(ctx, orderBefore.owner, orderOwnerAccountBefore.lamports, expectedLamportsIncome);
    });

    it("Closes orders", async () => {
        const order1 = (await RPC.getOrders(ctx, { page: 1, perPage: 100, status: 'all', owner: ctx.traderFirst.signer.publicKey}))[0];
        const orderSpace = (await ctx.connection.getAccountInfo(order1.address)).data.length;
        const orderTokens1 = (await getTokenAccount(ctx.connection, order1.tokenVault)).amount;
        const ownerKey1 = ctx.traderFirst.signer.publicKey;
        const ownerTokens1 = (await getTokenAccount(ctx.connection, ctx.traderFirst.ata)).amount;
        const ownerLamports1 = (await ctx.connection.getAccountInfo(ownerKey1)).lamports;

        await RPC.closeOrder(ctx, order1.address, order1.tokenVault, ctx.traderFirst.signer, ctx.traderFirst.ata);
        await CheckCtx.closeOrder(ctx, order1.address, orderTokens1, orderSpace, ownerKey1, ownerTokens1, ownerLamports1);

        const order2 = (await RPC.getOrders(ctx, { page: 1, perPage: 100, status: 'all', owner: ctx.traderSecond.signer.publicKey}))[0];
        const orderTokens2 = (await getTokenAccount(ctx.connection, order2.tokenVault)).amount;
        const ownerKey2 = ctx.traderSecond.signer.publicKey;
        const ownerTokens2 = (await getTokenAccount(ctx.connection, ctx.traderSecond.ata)).amount;
        const ownerLamports2 = (await ctx.connection.getAccountInfo(ownerKey2)).lamports;

        await RPC.closeOrder(ctx, order2.address, order2.tokenVault, ctx.traderSecond.signer, ctx.traderSecond.ata);
        await CheckCtx.closeOrder(ctx, order2.address, orderTokens2, orderSpace, ownerKey2, ownerTokens2, ownerLamports2);

        expect((await RPC.getOrders(ctx, { page: 1, perPage: 100, status: 'all'})).length).to.be.eq(0);
    });

    it("Waits till the end of the current round and switches to the buying round", async () => {
        const currentRoundEndsAtMs = (Number(ctx.roundStartAt) + ctx.tradingDuration + 1) * 1000;
        await sleepTill(currentRoundEndsAtMs);

        await RPC.switchToBuying(ctx);
        await CheckCtx.currentRound(ctx, Round.Buying, Date.now());

        const expectedTokenPrice = Number(ctx.initialTokenPrice) * ctx.coeffA + ctx.coeffB;
        const pool = await program.account.poolAccount.fetch(ctx.accounts.pool.key);
        expect(Number(pool.tokenPrice)).to.be.eq(expectedTokenPrice);
    });

    it("Waits till the end of the IDO", async () => {
        await sleepTill((Number(ctx.endAt) + 1) * 1000);
    });

    it("Withdraws lamports income to the owner of the IDO", async () => {
        const poolInfoBefore = await connection.getAccountInfo(ctx.accounts.pool.key);
        const rentForPool = await ctx.connection.getMinimumBalanceForRentExemption(poolInfoBefore.data.length);
        const ownerLamportsBefore = (await connection.getAccountInfo(ctx.owner.publicKey)).lamports;

        await RPC.withdrawLamports(ctx);

        const poolLamportsAfter = (await connection.getAccountInfo(ctx.accounts.pool.key)).lamports;
        const ownerLamportsAfter = (await connection.getAccountInfo(ctx.owner.publicKey)).lamports;
        expect(poolLamportsAfter).to.be.eq(rentForPool);
        expect(ownerLamportsAfter - ownerLamportsBefore).to.be.eq( poolInfoBefore.lamports - rentForPool);
    });

    it("Terminates IDO", async () => {
        const vaultSellingRent = (await connection.getAccountInfo(ctx.vaultSelling)).lamports;
        const ownerBalanceBefore = (await connection.getAccountInfo(ctx.owner.publicKey)).lamports;
        const unsoldTokensToBeBurned = (await getTokenAccount(connection, ctx.vaultSelling)).amount;
        const mintSupplyBefore = (await getMint(connection, ctx.sellingMint)).supply;
        const poolInfoBefore = await connection.getAccountInfo(ctx.accounts.pool.key);
        const rentForPool = await ctx.connection.getMinimumBalanceForRentExemption(poolInfoBefore.data.length);

        await RPC.terminate(ctx);

        const mintSupplyAfter = (await getMint(connection, ctx.sellingMint)).supply;
        const ownerBalanceAfter = (await connection.getAccountInfo(ctx.owner.publicKey)).lamports;

        expect(mintSupplyBefore - mintSupplyAfter).to.be.eq(unsoldTokensToBeBurned);
        expect(ownerBalanceAfter - ownerBalanceBefore).to.be.eq(vaultSellingRent + rentForPool);
    });
});
