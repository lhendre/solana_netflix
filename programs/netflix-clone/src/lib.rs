/// Include libraries for program
use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token};
use std::mem::size_of;
use anchor_lang::solana_program::log::{
    sol_log_compute_units
};
// Declare program ID
declare_id!("B3LNGaKnMwGQ1XQRseRbQbEzc1FjbztHrqmbXHruGgfm");

// Video and comment text length
const TEXT_LENGTH: usize = 1024;
// Username length
const USER_NAME_LENGTH: usize = 100;
// User profile imaage url length
const USER_URL_LENGTH: usize = 255;
const VIDEO_URL_LENGTH: usize = 255;
//https://solongwallet.medium.com/solana-development-tutorial-things-you-should-know-before-structuring-your-code-807f0e2ee43
const NUMBER_OF_ALLOWED_USERS_SPACE: usize = 2;
const NUMBER_OF_ALLOWED_USERS: u8 = 2;
const NUMBER_OF_ALLOWED_SUBSCRIBERS: u64 = 2;

const SUBSCRIBE_PRICE: u64 = 1;

/// Netflix Clone program
#[program]
pub mod netflix_clone {
    use super::*;

    /// Create state to save the video counts
    /// There is only one state in the program
    /// This account should be initialized before video
    pub fn create_state(
        ctx: Context<CreateState>,
    ) -> Result<()> {
        // Get state from context
        let state = &mut ctx.accounts.state;
        // Save authority to state
        state.authority = ctx.accounts.authority.key();
        // Set video count as 0 when initializing
        state.video_count = 0;
        Ok(())
    }

    /// Add movie
    /// @param text:        text of video
    /// @param creater_name: name of video creator
    /// @param creater_url:  url of video creator avatar
    pub fn add_movie(
        ctx: Context<AddMovie>,
        description: String,
        video_url: String,
        creater_name: String,
        creater_url: String,
    ) -> Result<()> {
        // Get State
       msg!(&description);  //logging

       if description.trim().is_empty() || video_url.trim().is_empty() {
           return Err(error!(Errors::CannotAddMovie));
       }
        let state = &mut ctx.accounts.state;

        // Get video
        let video = &mut ctx.accounts.video;
        // Set authority
        video.authority = ctx.accounts.authority.key();
        // Set text
        video.description = description;
        video.video_url = video_url;

        // Set creator name
        video.creater_name = creater_name;
        // Set creator avatar url
        video.creater_url = creater_url;
        // Set comment count as 0
        video.comment_count = 0;
        video.subscribe_count = 0;
        // Set video index as state's video count
        video.index = state.video_count;
        // Set video time
        video.creater_time = ctx.accounts.clock.unix_timestamp;

        video.likes = 0;

        video.remove = 0;

        // Increase state's video count by 1
        state.video_count += 1;
        msg!("Video Added!");  //logging
        sol_log_compute_units(); //Logs how many compute units are left, important for budget
        Ok(())
    }


    /// Create comment for video
    /// @param text:            text of comment
    /// @param commenter_name:  name of comment creator
    /// @param commenter_url:   url of comment creator avatar
    pub fn create_comment(
        ctx: Context<CreateComment>,
        text: String,
        commenter_name: String,
        commenter_url: String,
    ) -> Result<()> {

        // Get video
        let video = &mut ctx.accounts.video;
        if video.remove <= -500 {
            return Err(error!(Errors::UserCensoredVideo));
        }
        // Get comment
        let comment = &mut ctx.accounts.comment;
        // Set authority to comment
        comment.authority = ctx.accounts.authority.key();
        // Set comment text
        comment.text = text;
        // Set commenter name
        comment.commenter_name = commenter_name;
        // Set commenter url
        comment.commenter_url = commenter_url;
        // Set comment index to video's comment count
        comment.index = video.comment_count;
        // Set video time
        comment.video_time = ctx.accounts.clock.unix_timestamp;

        // Increase video's comment count by 1
        video.comment_count += 1;

        Ok(())
    }

    pub fn subscribe(
        ctx: Context<AddSubscribe>,
    ) -> Result<()> {
        // Get video
        let video = &mut ctx.accounts.video;
        let to_address_signer = video.authority;
        let to_address = &mut ctx.accounts.creator;
        let from_address = &mut ctx.accounts.authority;
        if **from_address.try_borrow_mut_lamports()? < SUBSCRIBE_PRICE {
            return Err(error!(Errors::InsufficientFundsForTransaction));
        }
        // Debit and credit to_account
        **from_address.try_borrow_mut_lamports()? -= SUBSCRIBE_PRICE;
        **to_address.try_borrow_mut_lamports()? += SUBSCRIBE_PRICE;

        //no null in rust
        if video.subscribe_count<NUMBER_OF_ALLOWED_SUBSCRIBERS {
            video.people_who_subcribed.push(to_address_signer);
            video.subscribe_count += 1;
        }
        Ok(())

    }


    pub fn approve(
        ctx: Context<Approve>,
    ) -> Result<()> {
        // Get video
        let video = &mut ctx.accounts.video;

        // Increase video's comment count by 1
        video.remove += 1;

        Ok(())
    }

    pub fn disapprove(
        ctx: Context<DisApprove>,

    ) -> Result<()> {
        // Get video
        let video = &mut ctx.accounts.video;

        // Increase video's comment count by 1
        video.remove -= 1;

        Ok(())
    }

    pub fn like_video(ctx: Context<LikeVideo>, user_liking_video: Pubkey) -> Result<()> {
        let video = &mut ctx.accounts.video;

        if video.likes == NUMBER_OF_ALLOWED_USERS {
            return Err(error!(Errors::ReachedMaxLikes));
        }
        if video.remove == -500 {
            return Err(error!(Errors::UserCensoredVideo));
        }

        // Iterating accounts is safer then indexing
        let mut iter = video.people_who_liked.iter();

        if iter.any(|&v| v == user_liking_video) {
            return Err(error!(Errors::UserLikedVideo));
        }

        video.likes += 1;
        video.people_who_liked.push(user_liking_video);

        Ok(())
    }

}

/// Contexts
/// CreateState context
#[derive(Accounts)]
pub struct CreateState<'info> {
    // Authenticating state account
    #[account(
        init,
        seeds = [b"state".as_ref()],
        bump,
        payer = authority,
        space = size_of::<StateAccount>() + 8
    )]
    pub state: Account<'info, StateAccount>,

    // Authority (this is signer who paid transaction fee)
    #[account(mut)]
    pub authority: Signer<'info>,

    /// System program
    /// CHECK: Simple test account for netflix
    pub system_program: UncheckedAccount<'info>,

    // Token program
    #[account(constraint = token_program.key == &token::ID)]
    pub token_program: Program<'info, Token>,
}

/// AddMovie context
#[derive(Accounts)]
pub struct AddMovie<'info> {
    // Authenticate state account
    #[account(mut, seeds = [b"state".as_ref()], bump)]
    pub state: Account<'info, StateAccount>,

    // Authenticate video account
    #[account(
        init,
        // Video account use string "video" and index of video as seeds
        seeds = [b"video".as_ref(), state.video_count.to_be_bytes().as_ref()],
        bump,
        payer = authority,
        space = size_of::<VideoAccount>() + TEXT_LENGTH + USER_NAME_LENGTH + USER_URL_LENGTH+VIDEO_URL_LENGTH+8+32*NUMBER_OF_ALLOWED_USERS_SPACE*2 // 32 bits in a pubkey and we have 5
    )]
    pub video: Account<'info, VideoAccount>,

    // Authority (this is signer who paid transaction fee)
    #[account(mut)]
    pub authority: Signer<'info>,

    /// System program
    /// CHECK: Simple test account for netflix
    pub system_program: UncheckedAccount<'info>,

    // Token program
    #[account(constraint = token_program.key == &token::ID)]
    pub token_program: Program<'info, Token>,

    // Clock to save time
    pub clock: Sysvar<'info, Clock>,
}

/// CreateComment context
#[derive(Accounts)]
pub struct CreateComment<'info> {
    // Authenticate video account
    #[account(mut, seeds = [b"video".as_ref(), video.index.to_be_bytes().as_ref()], bump)]
    pub video: Account<'info, VideoAccount>,

    // Authenticate comment account
    #[account(
        init,
        // Video account use string "comment", index of video and index of comment per video as seeds
        seeds = [b"comment".as_ref(), video.index.to_be_bytes().as_ref(), video.comment_count.to_be_bytes().as_ref()],
        bump,
        payer = authority,
        space = size_of::<CommentAccount>() + TEXT_LENGTH + USER_NAME_LENGTH + USER_URL_LENGTH+VIDEO_URL_LENGTH
    )]
    pub comment: Account<'info, CommentAccount>,

    // Authority (this is signer who paid transaction fee)
    #[account(mut)]
    pub authority: Signer<'info>,

    /// System program
    /// CHECK: Simple test account for netflix
    pub system_program: UncheckedAccount<'info>,

    // Token program
    #[account(constraint = token_program.key == &token::ID)]
    pub token_program: Program<'info, Token>,

    // Clock to save time
    pub clock: Sysvar<'info, Clock>,
}

/// AddSubscribe context
#[derive(Accounts)]
pub struct AddSubscribe<'info> {
    // Authenticate video account
    #[account(mut, seeds = [b"video".as_ref(), video.index.to_be_bytes().as_ref()], bump)]
    pub video: Account<'info, VideoAccount>,

    // Authority (this is signer who paid transaction fee)
    /// CHECK: Sample contract
    #[account(mut, signer)]
    pub authority: AccountInfo<'info>,

    /// CHECK: Sample contract
    #[account(mut)]
    pub creator: AccountInfo<'info>,

    /// System program
    /// CHECK: Simple test account for netflix
    pub system_program: UncheckedAccount<'info>,

    // Token program
    #[account(constraint = token_program.key == &token::ID)]
    pub token_program: Program<'info, Token>,

    // Clock to save time
    pub clock: Sysvar<'info, Clock>,
}

// State Account Structure
#[account]
pub struct StateAccount {
    // Signer address
    pub authority: Pubkey,

    // Video count
    pub video_count: u64,
}

// Video Account Structure
#[account]
pub struct VideoAccount {
    // Signer address
    pub authority: Pubkey,

    // description text
    pub description: String,

    // video url
    pub video_url: String,

    // Video creator name
    pub creater_name: String,

    // Video creator url
    pub creater_url: String,

    // Comment counts of videos
    pub comment_count: u64,

    pub subscribe_count:u64,

    // Video index
    pub index: u64,

    // Video time
    pub creater_time: i64,

    // likes: vect of people who liked it,
    pub people_who_liked: Vec<Pubkey>,
    // subscribe list: vect of people who liked it,
    pub people_who_subcribed: Vec<Pubkey>,

    // number of likes
    pub likes: u8,

    pub remove: i64,


}

// Comment Account Structure
#[account]
pub struct CommentAccount {
    // Signer address
    pub authority: Pubkey,

    // Comment text
    pub text: String,

    // commenter_name
    pub commenter_name: String,

    // commenter_url
    pub commenter_url: String,

    // Comment index
    pub index: u64,

    // Video time
    pub video_time: i64,
}

#[derive(Accounts)]
pub struct LikeVideo<'info> {
    #[account(mut)]
    pub video: Account<'info, VideoAccount>
}

#[derive(Accounts)]
pub struct Approve<'info> {
    #[account(mut)]
    pub video: Account<'info, VideoAccount>
}

#[derive(Accounts)]
pub struct DisApprove<'info> {
    #[account(mut)]
    pub video: Account<'info, VideoAccount>
}



#[error_code]
pub enum Errors {
    #[msg("Video cannot be created updated, missing data")]
    CannotAddMovie,

    #[msg("Cannot receive more than 5 likes")]
    ReachedMaxLikes,


    #[msg("User has already liked the tweet")]
    UserLikedVideo,

    #[msg("Video with potentially bad content")]
    UserCensoredVideo,

    #[msg("Insufficient Funds For Transaction")]
    InsufficientFundsForTransaction
}
