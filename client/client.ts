import BN from "bn.js";
import * as web3 from "@solana/web3.js";
import * as anchor from "@coral-xyz/anchor";
import {
  getAccount,
  getOrCreateAssociatedTokenAccount,
} from "@solana/spl-token";
import { PublicKey } from "@solana/web3.js";
import { keypairIdentity, token, Metaplex } from "@metaplex-foundation/js";
import type { TokenVault } from "../target/types/token_vault";

// Configure the client to use the local cluster
anchor.setProvider(anchor.AnchorProvider.env());

const program = anchor.workspace.TokenVault as anchor.Program<TokenVault>;


const mintAuthority = program.provider.wallet.payer;

const decimals = 9;

let [tokenAccountOwnerPda] = PublicKey.findProgramAddressSync(
  [Buffer.from("token_account_owner_pda")],
  program.programId
);

const metaplex = new Metaplex(program.provider.connection).use(
  keypairIdentity(program.provider.wallet.payer)
);

const createdSFT = await metaplex.nfts().createSft({
  uri: "https://shdw-drive.genesysgo.net/AzjHvXgqUJortnr5fXDG2aPkp2PfFMvu4Egr57fdiite/PirateCoinMeta",
  name: "Gold",
  symbol: "GOLD",
  sellerFeeBasisPoints: 1000,
  updateAuthority: mintAuthority,
  mintAuthority: mintAuthority,
  decimals: decimals,
  isMutable: true,
});

console.log(
  "Creating semi fungible spl token with address: " + createdSFT.sft.address
);

const mintDecimals = Math.pow(10, decimals);

let mintResult = await metaplex.nfts().mint({
  nftOrSft: createdSFT.sft,
  authority: program.provider.wallet.payer,
  toOwner: program.provider.wallet.payer.publicKey,
  amount: token(1000 * mintDecimals),
});

await metaplex.nfts().mint({
  nftOrSft: createdSFT.sft,
  authority: program.provider.wallet.payer,
  toOwner: new PublicKey("CEy2oCNZXWVkCgo4L6pYk2rswo5Hzbh4SjYUWhgpY5fj"),
  amount: token(1000 * mintDecimals),
});
await metaplex.nfts().mint({
  nftOrSft: createdSFT.sft,
  authority: program.provider.wallet.payer,
  toOwner: new PublicKey("BNkS3r6YiV8R8kBfTjGo7CBEw1YzLATfze42HdSKu4VU"),
  amount: token(1000 * mintDecimals),
});
console.log("Mint to result: " + mintResult.response.signature);

const tokenAccount = await getOrCreateAssociatedTokenAccount(
  program.provider.connection,
  program.provider.wallet.payer,
  createdSFT.mintAddress,
  program.provider.wallet.payer.publicKey
);

console.log("tokenAccount: " + tokenAccount.address);
console.log("TokenAccountOwnerPda: " + tokenAccountOwnerPda);

let tokenAccountInfo = await getAccount(program.provider.connection, tokenAccount.address);
console.log(
  "Owned token amount: " + tokenAccountInfo.amount / BigInt(mintDecimals)
);
let [tokenVault] = PublicKey.findProgramAddressSync(
  [Buffer.from("token_vault"), createdSFT.mintAddress.toBuffer()],
  program.programId
);
console.log("VaultAccount: " + tokenVault);

let confirmOptions = {
  skipPreflight: true,
};

let [initProposalAccountOwnerPda] = PublicKey.findProgramAddressSync(
  [Buffer.from("Proposal"), createdSFT.mintAddress.toBuffer()],
  program.programId
);

let txHash = await program.methods
  .initialize()
  .accounts({
    tokenAccountOwnerPda: tokenAccountOwnerPda,
    vaultTokenAccount: tokenVault,
    proposal: initProposalAccountOwnerPda,
    mintOfTokenBeingSent: createdSFT.mintAddress,
    signer: program.provider.publicKey,
  })
  .rpc(confirmOptions);

console.log(`Initialize`);
await logTransaction(txHash);

console.log(`Vault initialized.`);

txHash = await program.methods
  .transferIn(new anchor.BN(500 * mintDecimals))
  .accounts({
    tokenAccountOwnerPda: tokenAccountOwnerPda,
    vaultTokenAccount: tokenVault,
    senderTokenAccount: tokenAccount.address,
    mintOfTokenBeingSent: createdSFT.mintAddress,
    signer: program.provider.publicKey,
  })
  .signers([program.provider.wallet.payer])
  .rpc(confirmOptions);

tokenAccountInfo = await getAccount(program.provider.connection, tokenAccount.address);
console.log(
  "Owned token amount: " + tokenAccountInfo.amount / BigInt(mintDecimals)
);
tokenAccountInfo = await getAccount(program.provider.connection, tokenVault);
console.log(
  "Vault token amount: " + tokenAccountInfo.amount / BigInt(mintDecimals)
);

async function logTransaction(txHash) {
  const { blockhash, lastValidBlockHeight } =
    await program.provider.connection.getLatestBlockhash();

  await program.provider.connection.confirmTransaction({
    blockhash,
    lastValidBlockHeight,
    signature: txHash,
  });

  console.log(
    `Solana Explorer: https://explorer.solana.com/tx/${txHash}?cluster=devnet`
  );
}

let [proposalAccountOwnerPda] = PublicKey.findProgramAddressSync(
  [
    Buffer.from("Proposal"),
    initProposalAccountOwnerPda.toBuffer(),
    program.provider.publicKey.toBuffer(),
  ],
  program.programId
);
txHash = await program.methods
  .createProposal("update_info", "statistr01", "0x0034248bc12a")
  .accounts({
    proposal: proposalAccountOwnerPda,
    otherProposal: initProposalAccountOwnerPda,
    signer: program.provider.publicKey,
  })
  .signers([program.provider.wallet.payer])
  .rpc(confirmOptions);

console.log(`create a proposal ` + proposalAccountOwnerPda);
await logTransaction(txHash);

let [stakeAccountOwnerPda] = PublicKey.findProgramAddressSync(
  [
    Buffer.from("StakeInfo"),
    createdSFT.mintAddress.toBuffer(),
    program.provider.publicKey.toBuffer(),
  ],
  program.programId
);
txHash = await program.methods
  .stake(new anchor.BN(1 * mintDecimals), new anchor.BN(15))
  .accounts({
    tokenAccountOwnerPda: tokenAccountOwnerPda,
    vaultTokenAccount: tokenVault,
    senderTokenAccount: tokenAccount.address,
    mintOfTokenBeingSent: createdSFT.mintAddress,
    stakeInfo: stakeAccountOwnerPda,
    signer: program.provider.publicKey,
  })
  .signers([program.provider.wallet.payer])
  .rpc(confirmOptions);

console.log(`Stake one token into the vault.` + stakeAccountOwnerPda);
await logTransaction(txHash);

tokenAccountInfo = await getAccount(program.provider.connection, tokenAccount.address);
console.log(
  "Owned token amount: " + tokenAccountInfo.amount / BigInt(mintDecimals)
);

tokenAccountInfo = await getAccount(program.provider.connection, tokenVault);
console.log(
  "Vault token amount: " + tokenAccountInfo.amount / BigInt(mintDecimals)
);
txHash = await program.methods
  .startHolding(new anchor.BN(1 * mintDecimals))
  .accounts({
    tokenAccountOwnerPda: tokenAccountOwnerPda,
    vaultTokenAccount: tokenVault,
    senderTokenAccount: tokenAccount.address,
    mintOfTokenBeingSent: createdSFT.mintAddress,
    stakeInfo: stakeAccountOwnerPda,
    signer: program.provider.publicKey,
  })
  .signers([program.provider.wallet.payer])
  .rpc(confirmOptions);

console.log(`Start hold one token into the vault.`);
await logTransaction(txHash);

tokenAccountInfo = await getAccount(program.provider.connection, tokenAccount.address);
console.log(
  "Owned token amount: " + tokenAccountInfo.amount / BigInt(mintDecimals)
);

tokenAccountInfo = await getAccount(program.provider.connection, tokenVault);
console.log(
  "Vault token amount: " + tokenAccountInfo.amount / BigInt(mintDecimals)
);

let [voteAccountOwnerPda] = PublicKey.findProgramAddressSync(
  [
    Buffer.from("Vote"),
    proposalAccountOwnerPda.toBuffer(),
    program.provider.wallet.payer.publicKey.toBuffer(),
  ],
  program.programId
);
txHash = await program.methods
  .createVote(true)
  .accounts({
    vote: voteAccountOwnerPda,
    proposal: proposalAccountOwnerPda,
    stakeTicket: stakeAccountOwnerPda,
  })
  .signers([program.provider.wallet.payer])
  .rpc(confirmOptions);
console.log(`create a vote` + voteAccountOwnerPda);
await logTransaction(txHash);

const accountInfo = await program.provider.connection.getAccountInfo(proposalAccountOwnerPda);
console.log("Account data:", accountInfo.data);

txHash = await program.methods
  .endHolding()
  .accounts({
    tokenAccountOwnerPda: tokenAccountOwnerPda,
    vaultTokenAccount: tokenVault,
    senderTokenAccount: tokenAccount.address,
    mintOfTokenBeingSent: createdSFT.mintAddress,
    stakeInfo: stakeAccountOwnerPda,
    signer: program.provider.publicKey,
  })
  .signers([program.provider.wallet.payer])
  .rpc(confirmOptions);

console.log(`End hold one token into the vault.`);
await logTransaction(txHash);

tokenAccountInfo = await getAccount(program.provider.connection, tokenAccount.address);
console.log(
  "Owned token amount: " + tokenAccountInfo.amount / BigInt(mintDecimals)
);

tokenAccountInfo = await getAccount(program.provider.connection, tokenVault);
console.log(
  "Vault token amount: " + tokenAccountInfo.amount / BigInt(mintDecimals)
);
txHash = await program.methods
  .unstake()
  .accounts({
    tokenAccountOwnerPda: tokenAccountOwnerPda,
    vaultTokenAccount: tokenVault,
    senderTokenAccount: tokenAccount.address,
    mintOfTokenBeingSent: createdSFT.mintAddress,
    stakeInfo: stakeAccountOwnerPda,
    signer: program.provider.publicKey,
  })
  .signers([program.provider.wallet.payer])
  .rpc(confirmOptions);

console.log(`Unstake 1 hold one token into the vault.`);
await logTransaction(txHash);

tokenAccountInfo = await getAccount(program.provider.connection, tokenAccount.address);
console.log(
  "Owned token amount: " + tokenAccountInfo.amount / BigInt(mintDecimals)
);

tokenAccountInfo = await getAccount(program.provider.connection, tokenVault);
console.log(
  "Vault token amount: " + tokenAccountInfo.amount / BigInt(mintDecimals)
);
