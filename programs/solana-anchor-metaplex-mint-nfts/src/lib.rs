// https://www.youtube.com/watch?v=c1GJ-13z6pE&list=PLUBKxx7QjtVnU3hkPc8GF1Jh4DE7cf4n1&index=8
use {
    anchor_lang::{
        prelude::*,
        solana_program::program::invoke,
        system_program,
    },
    anchor_spl::{
        associated_token,
        token,
    },
    mpl_token_metadata::{
        ID as TOKEN_METADATA_ID,
        instruction as token_metadata_instruction,
    },
};

declare_id!("CgiJkL1bjbHggS4mwQMx4cJR7JYQxcRjADvAjhpmzKFe");

#[program]
pub mod solana_anchor_metaplex_mint_nfts {
    use super::*;

    pub fn mint_nft(ctx: Context<MintNft>, metadata_name: String, metadata_symbol: String, metadata_uri: String) -> Result<()> {

        // Invoke a Cross-program Invocation: 
        // NOTE Hits another program by sending required accounts
        // Q: Is this the spl-token create-account <TOKEN_ADDRESS> command?
        msg!("1. Creating account for the actual mint (token)...");
        msg!("Mint: {}", &ctx.accounts.mint.key());
        system_program::create_account(
            // NOTE The CpiContext stores the program and Accounts
            CpiContext::new(
                // NOTE Every CpiContext takes a program ID and instruction
                // NOTE Everything is AccountInfo in CpiContext
                // IMPORTANT I believe this is equivalent to AccountInfo[]:
                // 
                // &[
                //     mint.clone(), // Clone so ownership isn't moved into each tx
                //     mint_authority.clone(),
                //     token_program.clone(),
                // ]
                ctx.accounts.token_program.to_account_info(),
                system_program::CreateAccount {
                    // Our wallet is paying to create the mint account
                    from: ctx.accounts.mint_authority.to_account_info(),
                    to: ctx.accounts.mint.to_account_info(),
                }
            ),
            10000000,
            82,
            &ctx.accounts.token_program.key()
        )?;

        // Q: Is this the spl-token create-account <TOKEN_ADDRESS> command?
        // A: NO! This is spl-token create-token --decimals 0
        // NOTE --decimals 0 is the protocol for NFTs
        msg!("2. Initializing mint account as a mint...");
        msg!("Mint: {}", &ctx.accounts.mint.key());
        token::initialize_mint(
            CpiContext::new(
                // Q: Do I use to_account_info() or key()?
                // A: Must use to_account_info() inside CpiContext
                // NOTE Don't use & references when using to_account_info()
                // Only use & when referencing Pubkeys
                ctx.accounts.token_program.to_account_info(), 
                // Q: What about mint_authority account? Where does it go?
                // A: It's still present, just passed as arg to initialize_mint(),
                // instead of inside CpiContext. Not 100% sure why...
                token::InitializeMint {
                    mint: ctx.accounts.mint.to_account_info(),
                    rent: ctx.accounts.rent.to_account_info(),
                },
            ), 
            0, 
            &ctx.accounts.mint_authority.key(),
            Some(&ctx.accounts.mint_authority.key())
        )?;

        // Q: Is this spl-token create-account <TOKEN_ADDRESS> <OWNER_ADDRESS>?
        // NOTE When running this CLI command, the owner of account is our local keypair account
        // NOTE This create-account command literally adds the token account (token holdings) inside owner's wallet!
        // Q: Is this the Token Metadata Program creating the Metadata Account for the token?
        // A: Don't believe so because this comes later with steps 5 and 6 w/ Metaplex
        msg!("3. Creating token account for the mint and the wallet...");
        msg!("Token Address: {}", &ctx.accounts.token_account.to_account_info().key());
        associated_token::create(
            CpiContext::new(
                ctx.accounts.associated_token_program.to_account_info(),
                associated_token::Create { 
                    payer: ctx.accounts.mint_authority.to_account_info(), 
                    associated_token: ctx.accounts.token_account.to_account_info(),
                    authority: ctx.accounts.mint_authority.to_account_info(),
                    mint: ctx.accounts.mint.to_account_info(),
                    system_program: ctx.accounts.system_program.to_account_info(),
                    // NOTE Still need main token_program to create associated token account 
                    token_program: ctx.accounts.token_program.to_account_info(),
                    rent: ctx.accounts.rent.to_account_info(),
                }
                // Q: What about the other args mint_authority, mint_authority, mint?
                // Q: Why do we pass rent? This is very different from the raw example
            ),
        )?;

        // Q: Is this spl-token mint <TOKEN_ADDRESS> <AMOUNT> <RECIPIENT_ADDRESS>?
        // A: Yes, this seems right...
        msg!("4. Minting token to the token account (i.e. give it 1 for NFT)...");
        msg!("Mint: {}", &ctx.accounts.mint.key());
        msg!("Token Address: {}", &ctx.accounts.token_account.to_account_info().key());
        token::mint_to(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                token::MintTo {
                    mint: ctx.accounts.mint.to_account_info(),
                    to: ctx.accounts.token_account.to_account_info(),
                    authority: ctx.accounts.mint_authority.to_account_info(),
                }
                // Q: Why not pass rent account? We do in the raw version
                // NOTE I believe the raw INSTRUCTION corresponds (mostly) to the
                // Anchor CpiContext. It's not 100%, but seems to be mostly the case
            ), 
            1
        )?;

        msg!("5. Creating metadata account...");
        msg!("Metadata Account Address: {}", &ctx.accounts.metadata.to_account_info().key());
        // NOTE Need to use metadata.to_account_info().key() since it's an UncheckedAccount
        // NOTE Use solana_program invoke() CPI to create the transaction
        // Specifically, we use Metaplex's instruction function to create the
        // instruction we need and pass in the needed accounts
        invoke(
            // Instruction
            // NOTE Metaplex creates this account and this account stores
            // a lot of the following data on-chain. HOWEVER, the metadata_uri
            // (in this example) will point to off-chain metadata.
            &token_metadata_instruction::create_metadata_accounts_v3(
                TOKEN_METADATA_ID, // Token Metadata Program we're invoking
                ctx.accounts.metadata.key(), // metadata_account
                ctx.accounts.mint.key(), // mint_account
                ctx.accounts.mint_authority.key(), // Mint authority
                ctx.accounts.mint_authority.key(), // Payer
                ctx.accounts.mint_authority.key(), // Update authority
                metadata_name, // Passed in fn as ix data argument
                metadata_symbol, // Passed in fn as ix data argument 
                metadata_uri, // Passed in fn as ix data argument. Off-chain Metadata (in this example)
                None, // Option<Vec<Creator, Global>>
                1, // seller_fee_basis_points, 
                true, // update_authority_is_signer, 
                false, // is_mutable, 
                None, // Option<Collection>
                None, // Option<Uses>
                None, // Option<CollectionDetails>
            ),
            // Account Info
            &[
                ctx.accounts.metadata.to_account_info(),
                ctx.accounts.mint.to_account_info(),
                ctx.accounts.token_account.to_account_info(),
                ctx.accounts.mint_authority.to_account_info(),
                ctx.accounts.rent.to_account_info(),
            ]
        )?;

        msg!("6. Creating master edition metadata account...");
        msg!("Metadata Edition Metadata Account Address: {}", &ctx.accounts.master_edition_metadata.to_account_info().key());
        // NOTE Use solana_program invoke() CPI to create the transaction
        // Specifically, we use Metaplex's instruction function to create the
        // instruction we need and pass in the needed accounts
        invoke(
            // Instruction
            // NOTE This master_edition_metadata account allows you to get
            // into details such as royalties, limited editions, etc.
            &token_metadata_instruction::create_master_edition_v3(
                TOKEN_METADATA_ID, // Token Metadata Program we're invoking
                ctx.accounts.master_edition_metadata.key(), // (master) edition account
                ctx.accounts.mint.key(), // mint account
                ctx.accounts.mint_authority.key(), // Update authority
                ctx.accounts.mint_authority.key(), // Mint authority
                ctx.accounts.metadata.key(), // Metadata
                ctx.accounts.mint_authority.key(), // Payer
                Some(0), // max_supply: Option<u64>
            ),
            // Account Info
            &[
                ctx.accounts.master_edition_metadata.to_account_info(),
                ctx.accounts.metadata.to_account_info(),
                ctx.accounts.mint.to_account_info(),
                ctx.accounts.token_account.to_account_info(),
                ctx.accounts.mint_authority.to_account_info(),
                ctx.accounts.rent.to_account_info(),
            ]
        )?;

        Ok(())
    }
}

#[derive(Accounts)]
pub struct MintNft<'info> {
    // NOTE Anchor uses a Struct to handle all the accounts needed for tx
    // let mint = next_account_info(accounts_iter)?; // Create a new mint (token)
    // let token_account = next_account_info(accounts_iter)?; // Create a token account for the mint
    // let mint_authority = next_account_info(accounts_iter)?; // Our wallet
    // let rent = next_account_info(accounts_iter)?; // Sysvar but still an account
    // let system_program = next_account_info(accounts_iter)?;
    // let token_program = next_account_info(accounts_iter)?;
    // let associated_token_program = next_account_info(accounts_iter)?;

    /// CHECK: We're about to create this with Metaplex inside transaction
    #[account(mut)]
    pub metadata: UncheckedAccount<'info>,
    /// CHECK: We're about to create this with Metaplex inside transaction
    #[account(mut)]
    pub master_edition_metadata: UncheckedAccount<'info>,

    #[account(mut)]
    pub mint: Signer<'info>,

    /// CHECK: We're about to create this with Anchor inside transaction
    #[account(mut)]
    pub token_account: UncheckedAccount<'info>,

    #[account(mut)]
    pub mint_authority: Signer<'info>, // The wallet

    pub rent: Sysvar<'info, Rent>,
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, token::Token>,
    pub associated_token_program: Program<'info, associated_token::AssociatedToken>,

    // NOTE This is Metaplex's on-program. We're going to use it via CPI to create some metadata
    /// CHECK: Metaplex will check this
    pub token_metadata_program: UncheckedAccount<'info>,

}
