import * as anchor from "@project-serum/anchor";
import { Program } from "@project-serum/anchor";
import { TokenSale } from "../target/types/token_sale";
import { sleepTill } from "./helpers/helpers";
import { Connection, LAMPORTS_PER_SOL } from '@solana/web3.js';
import { Round } from "./helpers/round";
import { createCtx, Ctx } from "./helpers/ctx";
import { OrderAddress, RPC } from "./helpers/rpc";
import { CheckCtx } from "./helpers/check";
import { expect } from "chai";
import { getAccount as getTokenAccount } from "@solana/spl-token";

describe("token-sale", () => {
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

    it("Switches to trading round", async () => {
        const currentRoundEndsAtMs = (ctx.roundStartAt + ctx.buyingDuration) * 1000;
        await sleepTill(currentRoundEndsAtMs);
        await RPC.switchToTrading(ctx);
        await CheckCtx.currentRound(ctx, Round.Trading, Date.now());
    });

    it("Place orders for selling tokens", async () => {
        const amountToSellFirst = new anchor.BN(2),
            priceForTokenFirst = new anchor.BN(0.12 * LAMPORTS_PER_SOL),
            amountToSellSecond = new anchor.BN(5),
            priceForTokenSecond = new anchor.BN(0.13 * LAMPORTS_PER_SOL);

        const firstTraderBalanceBefore = await getTokenAccount(connection, ctx.traderFirst.ata);
        const secondTraderBalanceBefore = await getTokenAccount(connection, ctx.traderSecond.ata);

        const placedOrderFirst =  await RPC.placeOrder(ctx, ctx.traderFirst.signer, amountToSellFirst, priceForTokenFirst);
        await CheckCtx.lastPlacedOrder(ctx, placedOrderFirst);
        await CheckCtx.tokenBalance(ctx, ctx.traderFirst.ata, firstTraderBalanceBefore.amount, -amountToSellFirst);

        const placedOrderSecond = await RPC.placeOrder(ctx, ctx.traderSecond.signer, amountToSellSecond, priceForTokenSecond);
        await CheckCtx.lastPlacedOrder(ctx, placedOrderSecond);
        await CheckCtx.tokenBalance(ctx, ctx.traderSecond.ata, secondTraderBalanceBefore.amount, -amountToSellSecond);

        const orders: OrderAddress[] = await RPC.getOrders(ctx);
        expect(orders.length).to.be.eq(2);
    });

    it("Buy tokens from other traders", async () => {
        const orders: OrderAddress[] = await RPC.getOrders(ctx);
        const orderAddress = orders[0].pubkey;
        const orderBefore = await program.account.order.fetch(orderAddress);
        const orderOwnerAccountBefore = await connection.getAccountInfo(orderBefore.owner);
        const orderTokens = orderBefore.tokenAmount.tokens;
        const halfOfAllTokens = orderTokens.div(new anchor.BN(2));

        await RPC.redeemOrder(ctx, orderAddress, ctx.traderThird.signer, halfOfAllTokens);
        await RPC.redeemOrder(ctx, orderAddress, ctx.traderThird.signer, halfOfAllTokens);

        await CheckCtx.redeemedOrder(ctx, orderAddress, ctx.traderThird.signer.publicKey, orderTokens, orderTokens);

        const expectedLamportsIncome: number = ctx.initialTokenPrice * Number(orderTokens);
        await CheckCtx.lamportsBalance(ctx, orderBefore.owner, orderOwnerAccountBefore.lamports, expectedLamportsIncome);

        const pool = await program.account.poolAccount.fetch(ctx.accounts.pool.key);
        expect(`${pool.lastRoundTradingAmount.lamports}`).to.be.eq(`${expectedLamportsIncome}`);
    });

});
