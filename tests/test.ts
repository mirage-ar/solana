import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { Keypair, PublicKey, SystemProgram, LAMPORTS_PER_SOL } from "@solana/web3.js";
import { readFileSync } from "fs";
import { Gg } from "../target/types/gg";

import { mint, buy, sell, withdrawFromMint } from "./functions";
import { rand } from "./utils";

const PLAYER_COUNT = 10;
const SPONSOR_COUNT = 10;

const ownerPath = `${process.env.HOME}/.config/solana/id.json`;
const ownerSecretKey = JSON.parse(readFileSync(ownerPath, "utf8"));
const owner = new anchor.Wallet(Keypair.fromSecretKey(Uint8Array.from(ownerSecretKey))).payer;

const generateUserKeys = (name: string): Keypair => {
  const path = `${process.env.HOME}/.config/solana/wallets/${name}.json`;
  const secret = JSON.parse(readFileSync(path, "utf8"));
  return new anchor.Wallet(Keypair.fromSecretKey(Uint8Array.from(secret))).payer;
};

anchor.setProvider(anchor.AnchorProvider.env());
const program = anchor.workspace.Gg as Program<Gg>;

let players: Keypair[] = [];
let sponsors: Keypair[] = [];

for (let i = 0; i < PLAYER_COUNT; i++) {
  players.push(generateUserKeys(`player-${i}`));
}

for (let i = 0; i < SPONSOR_COUNT; i++) {
  sponsors.push(generateUserKeys(`sponsor-${i}`));
}

describe("INIT", () => {
  it("initialized", async () => {
    // AIRDROP USERS SOL
    console.log("AIRDROPING SOL TO USERS");
    for (let i = 0; i < players.length; i++) {
      const airdropSignature = await program.provider.connection.requestAirdrop(
        players[i].publicKey,
        LAMPORTS_PER_SOL * 2
      );

      await program.provider.connection.confirmTransaction(airdropSignature);
    }

    console.log("AIRDROPING SOL TO SPONSORS");
    for (let i = 0; i < sponsors.length; i++) {
      const airdropSignature = await program.provider.connection.requestAirdrop(
        sponsors[i].publicKey,
        LAMPORTS_PER_SOL * 10
      );

      await program.provider.connection.confirmTransaction(airdropSignature);
    }

    // INITIALIZE PROTOCOL ACCOUNT
    const [potPubkey, potBump] = await PublicKey.findProgramAddressSync([Buffer.from("POT")], program.programId);
    const [protocolPubkey, protocolBump] = await PublicKey.findProgramAddressSync(
      [Buffer.from("PROTOCOL")],
      program.programId
    );

    const tx = await program.rpc.initialize({
      accounts: {
        pot: potPubkey,
        protocol: protocolPubkey,
        authority: owner.publicKey,
        systemProgram: SystemProgram.programId,
      },
      signers: [owner],
    });

    console.log(tx);
  });
});

// PLAYER CARD MINTING
describe("MINT", () => {
  for (let i = 0; i < players.length; i++) {
    it(`player ${i} mints`, async () => {
      const user = players[i];
      await mint(user, program);
    });
  }
});

describe("BUY", () => {
  for (let i = 0; i < sponsors.length; i++) {
    const amount = rand(1, 10);
    // const amount = 1;
    it(`sponsor ${i} buys ${amount} shares`, async () => {
      const sponsor = sponsors[i];
      await buy(sponsor, players[i].publicKey, amount, program);
    });
  }
});


describe("SELL", () => {
  for (let i = 0; i < sponsors.length; i++) {
    // let amount = rand(1, 10);
    const amount = 1;
    it(`sponsor ${i} sells ${amount} shares`, async () => {
      const sponsor = sponsors[i];
      await sell(sponsor, players[i].publicKey, amount, program);
    });
  }
});

// PLAYER WITHDRAW FROM MINT + OWNER FROM PROTOCOL
describe("WITHDRAW", () => {
  for (let i = 0; i < players.length; i++) {
    it(`user ${i} withdraws`, async () => {
      const user = players[i];
      await withdrawFromMint(user, program);
    });
  }

  it("owner withdraws from protocol", async () => {
    const [protocolPubkey, protocolBump] = await PublicKey.findProgramAddressSync(
      [Buffer.from("PROTOCOL")],
      program.programId
    );

    const tx = await program.rpc.withdrawFromProtocol({
      accounts: {
        protocol: protocolPubkey,
        authority: owner.publicKey,
      },
      signers: [owner],
    });
    console.log(tx);
  });
});
