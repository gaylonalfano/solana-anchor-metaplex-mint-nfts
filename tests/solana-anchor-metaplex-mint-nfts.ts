import * as anchor from "@project-serum/anchor";
import { Program } from "@project-serum/anchor";
import { SolanaAnchorMetaplexMintNfts } from "../target/types/solana_anchor_metaplex_mint_nfts";

describe("solana-anchor-metaplex-mint-nfts", () => {
  const testNftTitle = "YouTube NFT";
  const testNftSymbol = "TUBE";
  const testNftUri;

  anchor.setProvider(anchor.AnchorProvider.env());

  const program = anchor.workspace
    .SolanaAnchorMetaplexMintNfts as Program<SolanaAnchorMetaplexMintNfts>;

  it("Is initialized!", async () => {
    // Add your test here.
    const tx = await program.methods.initialize().rpc();
    console.log("Your transaction signature", tx);
  });
});
