import * as anchor from "@project-serum/anchor";
import { Program } from "@project-serum/anchor";
import { TokenSale } from "../target/types/token_sale";
import {
    createUserWithLamports,
    sleep,
} from "./helpers";
import {
    PublicKey,
    SystemProgram,
    Keypair,
    Connection,
    Signer, LAMPORTS_PER_SOL,
} from '@solana/web3.js';
import {
    createMint,
    TOKEN_PROGRAM_ID,
    createAccount as createTokenAccount,
    getOrCreateAssociatedTokenAccount,
    Account as TokenAccount,
    mintTo,
    getAccount as getTokenAccount,
    createApproveInstruction,
    NATIVE_MINT,
    ASSOCIATED_TOKEN_PROGRAM_ID,
    getAssociatedTokenAddress,
} from '@solana/spl-token';
import { expect, assert } from 'chai';
import { Round } from "./round";

describe("token-sale", () => {
    // Configure the client to use the local cluster.
    anchor.setProvider(anchor.AnchorProvider.env());

    const program = anchor.workspace.TokenSale as Program<TokenSale>;
    const connection = new Connection("http://localhost:8899", 'recent');

    let owner: Signer;
    let sellingMint: PublicKey;
    // Owner's ATA with tokens for IDO
    let tokensForDistribution: TokenAccount;

    it('Inits the state of the world', async  () => {
        owner = await createUserWithLamports(connection, 1);
        sellingMint = await createMint(
            connection,
            owner, // payer
            owner.publicKey, // mintAuthority
            owner.publicKey, // freezeAuthority
            6 // decimals
        );
        tokensForDistribution = await getOrCreateAssociatedTokenAccount(connection, owner, sellingMint, owner.publicKey);
        await mintTo(
            connection,
            owner,
            sellingMint,
            tokensForDistribution.address,
            owner,
            100_000,
        );
    });

    const buyingDuration = 3,
        tradingDuration = 5,
        // 1 Token = 100_000_000 Lamports = 0.1 SOL
        initialTokenPrice = new anchor.BN(0.1 * LAMPORTS_PER_SOL),
        tokensPerRound = new anchor.BN(10_000),
        amountForSale = new anchor.BN(100_00);

    let poolPDA: PublicKey,
        nowBn: anchor.BN,
        roundStartAt: anchor.BN,
        endAt: anchor.BN,
        poolRentBalance: anchor.BN;

    let vaultSelling: PublicKey;

    it("Is initialized!", async () => {
        const [_poolPDA, poolBump] = await anchor.web3.PublicKey.findProgramAddress(
            [sellingMint.toBuffer()],
            program.programId
        );
        poolPDA = _poolPDA;

        vaultSelling = await getAssociatedTokenAddress(sellingMint, poolPDA, true);

        nowBn = new anchor.BN(Date.now() / 1000);
        roundStartAt = nowBn.add(new anchor.BN(5));
        endAt = nowBn.add(new anchor.BN(30));

        await program.methods.initialize(
            roundStartAt,
            endAt,
            buyingDuration,
            tradingDuration,
            initialTokenPrice,
            tokensPerRound,
            poolBump,
            amountForSale,
        ).accounts({
            poolAccount: poolPDA,
            distributionAuthority: owner.publicKey,
            tokensForDistribution: tokensForDistribution.address,
            sellingMint,
            vaultSelling,
            systemProgram: SystemProgram.programId,
            tokenProgram: TOKEN_PROGRAM_ID,
            associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
            rent: anchor.web3.SYSVAR_RENT_PUBKEY,
        }).signers([owner]).rpc();

        const pool = await program.account.poolAccount.fetch(poolPDA);
        expect(`${pool.owner}`).to.be.eq(`${owner.publicKey}`);
        expect(`${pool.sellingMint}`).to.be.eq(`${sellingMint}`);
        expect(`${pool.bump}`).to.be.eq(`${poolBump}`);
        expect(`${pool.vaultSelling}`).to.be.eq(`${vaultSelling}`);
        expect(`${pool.currentRound}`).to.be.eq(`${Round.Buying}`);
        expect(`${pool.roundStartAt}`).to.be.eq(`${roundStartAt}`);
        expect(`${pool.endAt}`).to.be.eq(`${endAt}`);
        expect(`${pool.buyingDuration}`).to.be.eq(`${buyingDuration}`);
        expect(`${pool.tradingDuration}`).to.be.eq(`${tradingDuration}`);
        expect(`${pool.tokenPrice}`).to.be.eq(`${initialTokenPrice}`);
        expect(`${pool.tokensPerRound}`).to.be.eq(`${tokensPerRound}`);
        expect(`${pool.latsRoundTradingAmount}`).to.be.eq(`${0}`);

        poolRentBalance = new anchor.BN((await connection.getAccountInfo(poolPDA)).lamports);
    });

    let buyerFirst: Signer;
    let buyerFirstATA: TokenAccount;
    let buyerSecond: Signer
    let buyerSecondATA: TokenAccount;
    let buyerThird: Signer;
    let buyerThirdATA: TokenAccount;

    it("Creates buyers and the ATAs", async () => {
        let [user1, ata1] = await createUserWithATA(sellingMint);
        let [user2, ata2] = await createUserWithATA(sellingMint);
        let [user3, ata3] = await createUserWithATA(sellingMint);
        buyerFirst = user1;
        buyerFirstATA = ata1;
        buyerSecond = user2;
        buyerSecondATA = ata2;
        buyerThird = user3;
        buyerThirdATA = ata3;

        expect(`${buyerFirstATA.amount}`).to.be.eq(`${0}`);
        expect(`${buyerSecondATA.amount}`).to.be.eq(`${0}`);
        expect(`${buyerThirdATA.amount}`).to.be.eq(`${0}`);
    });

    const firstTraderBuy = new anchor.BN(5);
    const secondTraderBuy = new anchor.BN(10);

    it("Buys tokens from the program", async () => {
        const tokenPriceBN = await getTokenPrice();

        await buyTokens(firstTraderBuy, buyerFirst);
        await buyTokens(secondTraderBuy, buyerSecond);

        await expectPoolBalance((firstTraderBuy.add(secondTraderBuy)).mul(tokenPriceBN));
        await expectTokenBalance(buyerFirstATA.address, firstTraderBuy);
        await expectTokenBalance(buyerSecondATA.address, secondTraderBuy);
    });

    it("Switches to trading round", async () => {
        // Wait until the current round is over.
        let currentRoundEndsAt = (roundStartAt.toNumber() + buyingDuration) * 1000;
        if (Date.now() < currentRoundEndsAt) {
            await sleep(currentRoundEndsAt - Date.now() + 1000);
        }

        await program.methods.switchToTrading()
            .accounts({
                poolAccount: poolPDA,
                clock: anchor.web3.SYSVAR_CLOCK_PUBKEY,
            })
            .rpc();

        const pool = await program.account.poolAccount.fetch(poolPDA);

        expect(`${pool.currentRound}`).to.be.eq(`${Round.Trading}`);
        // TODO check pool.roundStartAt is near the current time
    });

    const amountToSellFirst = new anchor.BN(2);
    const amountToSellSecond = new anchor.BN(3);
    const priceForTokenFirst = new anchor.BN(0.12 * LAMPORTS_PER_SOL);
    const priceForTokenSecond = new anchor.BN(0.13 * LAMPORTS_PER_SOL);
    let placedOrderFirst: PlacedOrder;
    let placedOrderSecond: PlacedOrder;

    it("Place orders for selling tokens", async () => {
        // Place two orders
        placedOrderFirst =  await placeOrderRPC(buyerFirst, amountToSellFirst, priceForTokenFirst);
        placedOrderSecond = await placeOrderRPC(buyerSecond, amountToSellSecond, priceForTokenSecond);

        // Check pool
        const pool = await program.account.poolAccount.fetch(poolPDA);
        const orders = pool.orders as OrderAddress[];
        expect(orders.length).to.be.eq(2);
        expect(`${orders[0].pubkey}`).to.be.eq(`${placedOrderFirst.address}`);
        expect(`${orders[0].bump}`).to.be.eq(`${placedOrderFirst.bump}`);
        expect(`${orders[1].pubkey}`).to.be.eq(`${placedOrderSecond.address}`);
        expect(`${orders[1].bump}`).to.be.eq(`${placedOrderSecond.bump}`);

        // Check token balances
        await expectTokenBalance(buyerFirstATA.address, firstTraderBuy.sub(amountToSellFirst));
        await expectTokenBalance(buyerSecondATA.address, secondTraderBuy.sub(amountToSellSecond));
    });

    it("Buy tokens from other traders", async () => {
        const amountToBuy = new anchor.BN(1);
        const buyer: Signer = buyerThird;
        const buyerTokenAccount: PublicKey = buyerThirdATA.address;
        const placedOrder: PlacedOrder = placedOrderFirst;

        const orderBefore = await program.account.order.fetch(placedOrder.address);
        const orderTokenVaultBefore = await getTokenAccount(connection, placedOrder.tokenVault);
        const orderOwnerAccountBefore = await anchor.getProvider().connection.getAccountInfo(placedOrder.owner);

        await redeemOrderRPC(amountToBuy, buyer, placedOrder);

        // Checking token balances
        const order = await program.account.order.fetch(placedOrder.address);
        expect(`${order.tokenAmount}`).to.be.eq(`${orderBefore.tokenAmount.sub(amountToBuy)}`);
        await expectTokenBalance(
            order.tokenVault,
            (new anchor.BN(orderTokenVaultBefore.amount.toString())).sub(amountToBuy)
        );
        await expectTokenBalance(buyerTokenAccount, amountToBuy);

        // Checking the order's owner lamport balance
        const expectedLamportsIncome = orderBefore.tokenPrice.mul(amountToBuy).toNumber();
        const orderOwnerAccount = await anchor.getProvider().connection.getAccountInfo(placedOrder.owner);
        expect(orderOwnerAccount.lamports - orderOwnerAccountBefore.lamports).to.be.eq(expectedLamportsIncome);
    });


    // Helper functions
    async function redeemOrderRPC(amountToBuy: anchor.BN, buyer: Signer, placedOrder: PlacedOrder) {
        const buyerTokenAccount: PublicKey = await getAssociatedTokenAddress(sellingMint, buyer.publicKey);

        await program.methods
            .redeemOrder(
                amountToBuy,
            )
            .accounts({
                poolAccount: poolPDA,
                sellingMint,
                buyer: buyer.publicKey,
                buyerTokenAccount,
                order: placedOrder.address,
                orderOwner: placedOrder.owner,
                orderTokenVault: placedOrder.tokenVault,
                tokenProgram: TOKEN_PROGRAM_ID,
                clock: anchor.web3.SYSVAR_CLOCK_PUBKEY,
                systemProgram: SystemProgram.programId,
            })
            .signers([buyer])
            .rpc();
    }

    async function placeOrderRPC(seller: Signer, amountToSell: anchor.BN, priceForToken: anchor.BN): Promise<PlacedOrder> {
        const [orderPDA, orderBump] = await anchor.web3.PublicKey.findProgramAddress(
            [
                anchor.utils.bytes.utf8.encode("order"),
                seller.publicKey.toBuffer(),
            ],
            program.programId
        );

        const orderTokenVault = await getAssociatedTokenAddress(sellingMint, orderPDA, true);

        const sellerTokenAccount = await getOrCreateAssociatedTokenAccount(
            connection,
            seller,
            sellingMint,
            seller.publicKey,
        );

        await program.methods
            .placeOrder(
                orderBump,
                amountToSell,
                priceForToken,
            )
            .accounts({
                poolAccount: poolPDA,
                sellingMint,
                seller: seller.publicKey,
                sellerTokenAccount: sellerTokenAccount.address,
                order: orderPDA,
                orderTokenVault,
                systemProgram: SystemProgram.programId,
                tokenProgram: TOKEN_PROGRAM_ID,
                associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
                rent: anchor.web3.SYSVAR_RENT_PUBKEY,
                clock: anchor.web3.SYSVAR_CLOCK_PUBKEY,
            })
            .signers([seller])
            .rpc();

        const order = await program.account.order.fetch(orderPDA);

        expect(`${order.bump}`).to.be.eq(`${orderBump}`);
        expect(`${order.owner}`).to.be.eq(`${seller.publicKey}`);
        expect(`${order.tokenVault}`).to.be.eq(`${orderTokenVault}`);
        expect(`${order.tokenPrice}`).to.be.eq(`${priceForToken}`);
        expect(`${order.tokenAmount}`).to.be.eq(`${amountToSell}`);
        await expectTokenBalance(orderTokenVault, amountToSell);

        return Promise.resolve({
            address: orderPDA,
            bump: orderBump,
            tokenVault: order.tokenVault,
            owner: order.owner,
        });
    }

    async function createUserWithATA(mint, lamports = 100): Promise<[Signer, TokenAccount]> {
        let user = await createUserWithLamports(connection, lamports);
        let ata = await getOrCreateAssociatedTokenAccount(
            connection,
            user,
            mint,
            user.publicKey
        );

        return Promise.all([user, ata]);
    }

    async function getTokenPrice(): Promise<anchor.BN> {
        const pool = await program.account.poolAccount.fetch(poolPDA);
        return new anchor.BN(pool.tokenPrice);
    }

    async function buyTokens(amount, buyer) {
        const buyerTokenAcc = await getOrCreateAssociatedTokenAccount(connection, buyer, sellingMint, buyer.publicKey);

        await program.methods.buy(
            amount,
        ).accounts({
            poolAccount: poolPDA,
            sellingMint,
            vaultSelling,
            buyer: buyer.publicKey,
            buyerTokenAccount: buyerTokenAcc.address,
            systemProgram: SystemProgram.programId,
            tokenProgram: TOKEN_PROGRAM_ID,
            clock: anchor.web3.SYSVAR_CLOCK_PUBKEY,
        }).signers([buyer]).rpc();
    }

    async function expectPoolBalance(expectedBalance: anchor.BN) {
        await expectLamportsBalance(poolPDA, expectedBalance.add(poolRentBalance));
    }

    async function expectLamportsBalance(account: PublicKey, expectedBalance: anchor.BN) {
        let info = await connection.getAccountInfo(poolPDA);
        expect(`${info.lamports}`).to.be.eq(`${expectedBalance}`);
    }

    async function expectTokenBalance(tokenAcc: PublicKey, expectedBalance: anchor.BN) {
        let acc = await getTokenAccount(connection, tokenAcc);
        expect(`${acc.amount}`).to.be.eq(`${expectedBalance}`);
    }
});

// TODO move
export interface OrderAddress {
    pubkey: PublicKey,
    bump: number,
}

export interface PlacedOrder {
    address: PublicKey,
    bump: number,
    tokenVault: PublicKey,
    owner: PublicKey,
}