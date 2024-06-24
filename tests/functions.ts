import * as anchor from "@coral-xyz/anchor";
import { Keypair, PublicKey, SystemProgram, LAMPORTS_PER_SOL } from "@solana/web3.js";
import { Program } from "@coral-xyz/anchor";
import { Gg } from "../target/types/gg";

export const mint = async (user: Keypair, program: Program<Gg>) => {
  const [mintPubkey, mintBump] = await PublicKey.findProgramAddressSync(
    [Buffer.from("MINT"), user.publicKey.toBuffer()],
    program.programId
  );

  const [protocolPubkey, protocolBump] = await PublicKey.findProgramAddressSync(
    [Buffer.from("PROTOCOL")],
    program.programId
  );

  const tx = await program.rpc.mint({
    accounts: {
      mint: mintPubkey,
      protocol: protocolPubkey,
      authority: user.publicKey,
      systemProgram: SystemProgram.programId,
    },
    signers: [user],
  });

  console.log(tx);
};

export const buy = async (user: Keypair, subject: PublicKey, amount: number, program: Program<Gg>) => {
  const [protocolPubkey, protocolBump] = await PublicKey.findProgramAddressSync(
    [Buffer.from("PROTOCOL")],
    program.programId
  );

  const [potPubkey, potBump] = await PublicKey.findProgramAddressSync([Buffer.from("POT")], program.programId);

  const [tokenPubkey, tokenBump] = await PublicKey.findProgramAddressSync(
    [Buffer.from("TOKEN"), user.publicKey.toBuffer(), subject.toBuffer()],
    program.programId
  );

  const [mintPubkey, mintBump] = await PublicKey.findProgramAddressSync(
    [Buffer.from("MINT"), subject.toBuffer()],
    program.programId
  );

  const tx = await program.rpc.buyShares(subject, new anchor.BN(amount), {
    accounts: {
      token: tokenPubkey,
      mint: mintPubkey,
      protocol: protocolPubkey,
      pot: potPubkey,
      authority: user.publicKey,
      systemProgram: SystemProgram.programId,
    },
    signers: [user],
  });
  console.log(tx);
};

export const sell = async (user: Keypair, subject: PublicKey, amount: number, program: Program<Gg>) => {
  const [protocolPubkey, protocolBump] = await PublicKey.findProgramAddressSync(
    [Buffer.from("PROTOCOL")],
    program.programId
  );

  const [potPubkey, potBump] = await PublicKey.findProgramAddressSync([Buffer.from("POT")], program.programId);

  const [tokenPubkey, tokenBump] = await PublicKey.findProgramAddressSync(
    [Buffer.from("TOKEN"), user.publicKey.toBuffer(), subject.toBuffer()],
    program.programId
  );

  const [mintPubkey, mintBump] = await PublicKey.findProgramAddressSync(
    [Buffer.from("MINT"), subject.toBuffer()],
    program.programId
  );

  const tx = await program.rpc.sellShares(subject, new anchor.BN(amount), {
    accounts: {
      token: tokenPubkey,
      mint: mintPubkey,
      protocol: protocolPubkey,
      pot: potPubkey,
      authority: user.publicKey,
    },
    signers: [user],
  });
  console.log(tx);
};

export const withdrawFromMint = async (user: Keypair, program: Program<Gg>) => {
  const [mintPubkey, mintBump] = await PublicKey.findProgramAddressSync(
    [Buffer.from("MINT"), user.publicKey.toBuffer()],
    program.programId
  );

  // Test Initialize
  const tx = await program.rpc.withdrawFromMint({
    accounts: {
      mint: mintPubkey,
      authority: user.publicKey,
      systemProgram: SystemProgram.programId,
    },
    signers: [user],
  });
  console.log(tx);
};