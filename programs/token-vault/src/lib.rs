use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, Token, TokenAccount, Transfer};

// This is your program's public key and it will update
// automatically when you build the project.
declare_id!("6dHbdCSXq4jhdmSwReXNhqtEKYKcHj2KqMHgAbHS1ZPV");

#[program]
mod token_vault {
    use super::*;
    pub fn initialize(_ctx: Context<Initialize>) -> Result<()> {
        Ok(())
    }
    pub fn create_stake_info(ctx: Context<CreateStakeInfo>) -> Result<()> {
        let stake_info = &mut ctx.accounts.stake_info;
        stake_info.staker = ctx.accounts.signer.to_account_info().key();
        Ok(())
    }
    pub fn transfer_in(ctx: Context<TransferAccounts>, amount: u64) -> Result<()> {
        msg!("Token amount transfer in: {}!", amount);
        // See Tutorial page 6
        // Below is the actual instruction that we are going to send to the Token program.
        let transfer_instruction = Transfer {
            from: ctx.accounts.sender_token_account.to_account_info(),
            to: ctx.accounts.vault_token_account.to_account_info(),
            authority: ctx.accounts.signer.to_account_info(),
        };

        let cpi_ctx = CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            transfer_instruction,
        );

        anchor_spl::token::transfer(cpi_ctx, amount)?;
        Ok(())
    }

    // pub fn transfer_out(ctx: Context<TransferAccounts>, amount: u64) -> Result<()> {
    //     msg!("Token amount transfer out: {}!", amount);
    //     // See Tutorial page 7
    //     let transfer_instruction = Transfer {
    //         from: ctx.accounts.vault_token_account.to_account_info(),
    //         to: ctx.accounts.sender_token_account.to_account_info(),
    //         authority: ctx.accounts.token_account_owner_pda.to_account_info(),
    //     };

    //     let bump = ctx.bumps.token_account_owner_pda;
    //     let seeds = &[b"token_account_owner_pda".as_ref(), &[bump]];
    //     let signer = &[&seeds[..]];

    //     let cpi_ctx = CpiContext::new_with_signer(
    //         ctx.accounts.token_program.to_account_info(),
    //         transfer_instruction,
    //         signer,
    //     );
    //     anchor_spl::token::transfer(cpi_ctx, amount)?;
    //     Ok(())
    // }

    pub fn stake(ctx: Context<Stake>, amount: u64, duration: u64) -> Result<()> {
        // msg!("Token amount transfer in: {}!", amount);
        // See Tutorial page 6
        // Below is the actual instruction that we are going to send to the Token program.
        let transfer_instruction = Transfer {
            from: ctx.accounts.sender_token_account.to_account_info(),
            to: ctx.accounts.vault_token_account.to_account_info(),
            authority: ctx.accounts.signer.to_account_info(),
        };

        let cpi_ctx = CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            transfer_instruction,
        );

        anchor_spl::token::transfer(cpi_ctx, amount)?;

        let base_duration: u64 = BASE_STAKING_DURATION;
        let _duration = determine_term(duration);

        let point_weight = _duration / base_duration;

        let stake_info = &mut ctx.accounts.stake_info;

        if stake_info.stake_amount > 0 || stake_info.hold_amount > 0 {
            assert!(stake_info.staker == *ctx.accounts.signer.key, "Error Owner");
        }

        // stake_info.staker = *ctx.accounts.signer.key;
        stake_info.staker = *ctx.accounts.signer.key;
        stake_info.stake_amount += amount;
        stake_info.duration = _duration;
        stake_info.point += amount * point_weight / 10;
        stake_info.stake_created_at = Clock::get()?.unix_timestamp;
        Ok(())
    }

    pub fn unstake(ctx: Context<Stake>) -> Result<()> {
        let stake_info = &mut ctx.accounts.stake_info;

        assert!(
            stake_info.staker == *ctx.accounts.signer.key && stake_info.stake_amount > 0,
            "Error Owner"
        );
        assert!(
            Clock::get()?.unix_timestamp - stake_info.stake_created_at
                >= stake_info.duration.try_into().unwrap(),
            "Error Staking time hasn't ended yet"
        );
        let amount = stake_info.stake_amount;
        // See Tutorial page 7
        let transfer_instruction = Transfer {
            from: ctx.accounts.vault_token_account.to_account_info(),
            to: ctx.accounts.sender_token_account.to_account_info(),
            authority: ctx.accounts.token_account_owner_pda.to_account_info(),
        };

        let bump = ctx.bumps.token_account_owner_pda;
        let seeds = &[b"token_account_owner_pda".as_ref(), &[bump]];
        let signer = &[&seeds[..]];

        let cpi_ctx = CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            transfer_instruction,
            signer,
        );
        anchor_spl::token::transfer(cpi_ctx, amount)?;

        stake_info.stake_amount -= amount;

        Ok(())
    }
    pub fn start_holding(ctx: Context<Stake>, amount: u64) -> Result<()> {
        // msg!("Token amount transfer in: {}!", amount);
        // See Tutorial page 6
        // Below is the actual instruction that we are going to send to the Token program.
        let transfer_instruction = Transfer {
            from: ctx.accounts.sender_token_account.to_account_info(),
            to: ctx.accounts.vault_token_account.to_account_info(),
            authority: ctx.accounts.signer.to_account_info(),
        };

        let cpi_ctx = CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            transfer_instruction,
        );

        anchor_spl::token::transfer(cpi_ctx, amount)?;

        let stake_info = &mut ctx.accounts.stake_info;

        if stake_info.stake_amount > 0 || stake_info.hold_amount > 0 {
            assert!(stake_info.staker == *ctx.accounts.signer.key, "Error Owner");
        }

        // stake_info.staker = *ctx.accounts.signer.key;
        stake_info.staker = *ctx.accounts.signer.key;
        stake_info.hold_amount += amount;
        stake_info.hold_created_at = Clock::get()?.unix_timestamp;
        Ok(())
    }
    pub fn end_holding(ctx: Context<Stake>) -> Result<()> {
        let stake_info = &mut ctx.accounts.stake_info;

        assert!(
            stake_info.staker == *ctx.accounts.signer.key && stake_info.hold_amount > 0,
            "Error Owner"
        );
        let amount = stake_info.hold_amount;
        // See Tutorial page 7
        let transfer_instruction = Transfer {
            from: ctx.accounts.vault_token_account.to_account_info(),
            to: ctx.accounts.sender_token_account.to_account_info(),
            authority: ctx.accounts.token_account_owner_pda.to_account_info(),
        };

        let bump = ctx.bumps.token_account_owner_pda;
        let seeds = &[b"token_account_owner_pda".as_ref(), &[bump]];
        let signer = &[&seeds[..]];

        let cpi_ctx = CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            transfer_instruction,
            signer,
        );
        anchor_spl::token::transfer(cpi_ctx, amount)?;

        stake_info.hold_amount -= amount;

        Ok(())
    }
    pub fn claim_rewards(ctx: Context<Stake>) -> Result<()> {
        let stake_info = &mut ctx.accounts.stake_info;

        assert!(
            stake_info.staker == *ctx.accounts.signer.key && stake_info.reward > 0,
            "Error Owner"
        );
        let amount = stake_info.reward;
        // See Tutorial page 7
        let transfer_instruction = Transfer {
            from: ctx.accounts.vault_token_account.to_account_info(),
            to: ctx.accounts.sender_token_account.to_account_info(),
            authority: ctx.accounts.token_account_owner_pda.to_account_info(),
        };

        let bump = ctx.bumps.token_account_owner_pda;
        let seeds = &[b"token_account_owner_pda".as_ref(), &[bump]];
        let signer = &[&seeds[..]];

        let cpi_ctx = CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            transfer_instruction,
            signer,
        );
        anchor_spl::token::transfer(cpi_ctx, amount)?;

        stake_info.reward -= amount;

        Ok(())
    }
    pub fn create_proposal(
        ctx: Context<CreateProposal>,
        description: String,
        statistr_id: String,
        hash: String,
    ) -> Result<()> {
        let proposal = &mut ctx.accounts.proposal;
        proposal.creator = ctx.accounts.signer.to_account_info().key();
        proposal.description = description;
        proposal.statistr_id = statistr_id;
        proposal.hash = hash;
        proposal.votes_yes = 0;
        proposal.votes_no = 0;
        proposal.reward = BASE_REWARD_FOR_PROPOSAL;
        proposal.creator_reward_rate = 10;
        proposal.created_at = Clock::get()?.unix_timestamp;
        proposal.duration = BASE_VOTING_DURATION;
        Ok(())
    }
    pub fn create_vote(ctx: Context<CreateVote>, vote_type: bool) -> Result<()> {
        let proposal = &mut ctx.accounts.proposal;
        let vote = &mut ctx.accounts.vote;
        let stake_ticket = &mut ctx.accounts.stake_ticket;

        if ((stake_ticket.stake_amount > 0
            || (stake_ticket.hold_amount > 0
                && Clock::get()?.unix_timestamp - stake_ticket.hold_created_at
                    > proposal.duration.try_into().unwrap()))
            && stake_ticket.staker == ctx.accounts.signer.to_account_info().key())
            && !proposal
                .voters
                .contains(&stake_ticket.to_account_info().key())
        {
            // get user weight
            let user_weight = user_weight(stake_ticket.point);

            // end get user weight

            vote.owner = ctx.accounts.signer.to_account_info().key();
            let mut vote_weight = stake_ticket.stake_amount;
            if Clock::get()?.unix_timestamp - stake_ticket.hold_created_at
                > proposal.duration.try_into().unwrap()
            {
                vote_weight += stake_ticket.hold_amount;
            }
            vote.vote_weight = vote_weight * (100 + user_weight) / 100;
            vote.vote_type = vote_type;
            vote.proposal_id = proposal.to_account_info().key();
            vote.created_at = Clock::get()?.unix_timestamp;
            if vote_type == true {
                proposal.votes_yes += vote_weight * (100 + user_weight) / 100;
            } else {
                proposal.votes_no += vote_weight * (100 + user_weight) / 100;
            }
            proposal.voters.push(stake_ticket.to_account_info().key());
        }

        Ok(())
    }

    pub fn collect_rewards(ctx: Context<ClaimReward>) -> Result<()> {
        let vote = &mut ctx.accounts.vote;
        let proposal = &mut ctx.accounts.proposal;
        let stake_ticket = &mut ctx.accounts.stake_ticket;
        let valid_proposal = valid_proposal(proposal).unwrap();

        assert!(valid_proposal, "Proposal is not qualified");
        assert!(
            stake_ticket.staker == ctx.accounts.signer.to_account_info().key(),
            "Error Stake ticket owner"
        );

        if proposal.votes_yes > proposal.votes_no
            && !proposal.creator_claimed_reward
            && proposal.creator == ctx.accounts.signer.to_account_info().key()
        {
            stake_ticket.reward += proposal.reward * proposal.creator_reward_rate / 100;
            stake_ticket.point += proposal.reward * proposal.creator_reward_rate / 1000;
            proposal.creator_claimed_reward = true;
        }

        if vote.proposal_id == proposal.to_account_info().key()
            && !vote.claimed_rewards
            && vote.vote_type == (proposal.votes_yes > proposal.votes_no)
            && vote.owner == ctx.accounts.signer.to_account_info().key()
        {
            let total_rewards = proposal.reward * (100 - proposal.creator_reward_rate) / 100;
            let rewards = vote.vote_weight * total_rewards / if vote.vote_type {
                proposal.votes_yes
            } else {
                proposal.votes_no
            };
            stake_ticket.reward += rewards;
            stake_ticket.point += rewards / 10;
            vote.claimed_rewards = true;
        }
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize<'info> {
    // Derived PDAs
    #[account(
        init_if_needed,
        payer = signer,
        seeds=[b"token_account_owner_pda"],
        bump,
        space = 8
    )]
    token_account_owner_pda: AccountInfo<'info>,
    #[account(init, payer = signer, seeds=[b"Proposal", mint_of_token_being_sent.key().as_ref()], bump, space = 8 + 256)]
    pub proposal: Account<'info, Proposal>,
    #[account(
        init_if_needed,
        payer = signer,
        seeds=[b"token_vault", mint_of_token_being_sent.key().as_ref()],
        token::mint=mint_of_token_being_sent,
        token::authority=token_account_owner_pda,
        bump
    )]
    vault_token_account: Account<'info, TokenAccount>,

    mint_of_token_being_sent: Account<'info, Mint>,

    #[account(mut)]
    signer: Signer<'info>,
    system_program: Program<'info, System>,
    token_program: Program<'info, Token>,
    rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
pub struct CreateStakeInfo<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,
    #[account(init, payer = signer, seeds=[b"StakeInfo", mint_of_token_being_sent.key().as_ref(), signer.key().as_ref()], bump, space = 8 + 256)]
    pub stake_info: Account<'info, StakeInfo>,
    mint_of_token_being_sent: Account<'info, Mint>,
    system_program: Program<'info, System>,
    token_program: Program<'info, Token>,
    rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
pub struct TransferAccounts<'info> {
    // Derived PDAs
    #[account(mut,
        seeds=[b"token_account_owner_pda"],
        bump
    )]
    token_account_owner_pda: AccountInfo<'info>,

    #[account(mut,
        seeds=[b"token_vault", mint_of_token_being_sent.key().as_ref()],
        bump,
        token::mint=mint_of_token_being_sent,
        token::authority=token_account_owner_pda,
    )]
    vault_token_account: Account<'info, TokenAccount>,

    #[account(mut)]
    sender_token_account: Account<'info, TokenAccount>,

    mint_of_token_being_sent: Account<'info, Mint>,

    #[account(mut)]
    signer: Signer<'info>,
    system_program: Program<'info, System>,
    token_program: Program<'info, Token>,
    rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
pub struct Stake<'info> {
    // Derived PDAs
    #[account(mut,
        seeds=[b"token_account_owner_pda"],
        bump
    )]
    token_account_owner_pda: AccountInfo<'info>,

    #[account(mut,
        seeds=[b"token_vault", mint_of_token_being_sent.key().as_ref()],
        bump,
        token::mint=mint_of_token_being_sent,
        token::authority=token_account_owner_pda,
    )]
    vault_token_account: Account<'info, TokenAccount>,

    #[account(mut)]
    sender_token_account: Account<'info, TokenAccount>,

    mint_of_token_being_sent: Account<'info, Mint>,

    #[account(mut)]
    signer: Signer<'info>,

    #[account(init_if_needed, payer = signer, seeds=[b"StakeInfo", mint_of_token_being_sent.key().as_ref(), signer.key().as_ref()], bump, space = 8 + 256)]
    pub stake_info: Account<'info, StakeInfo>,
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
pub struct CreateProposal<'info> {
    #[account(init, payer = signer, seeds=[b"Proposal", other_proposal.key().as_ref() ,signer.key().as_ref()], bump, space = 8 + 256)]
    pub proposal: Account<'info, Proposal>,
    #[account()]
    pub other_proposal: Account<'info, Proposal>,
    #[account(mut)]
    signer: Signer<'info>,
    system_program: Program<'info, System>,
    token_program: Program<'info, Token>,
    rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
pub struct ProposalResult<'info> {
    #[account()]
    pub proposal: Account<'info, Proposal>,
}

#[derive(Accounts)]
pub struct ProposalDetails<'info> {
    #[account()]
    pub proposal: Account<'info, Proposal>,
}

#[derive(Accounts)]
pub struct CreateVote<'info> {
    #[account(init, payer = signer, seeds=[b"Vote", proposal.key().as_ref(), signer.key().as_ref()], bump, space = 8 + 256)]
    pub vote: Account<'info, Vote>,
    #[account(mut)]
    pub proposal: Account<'info, Proposal>,
    #[account(mut)]
    pub stake_ticket: Account<'info, StakeInfo>,
    #[account(mut)]
    pub signer: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct ClaimReward<'info> {
    #[account(mut)]
    pub vote: Account<'info, Vote>,
    #[account(mut)]
    pub proposal: Account<'info, Proposal>,
    #[account(mut)]
    pub stake_ticket: Account<'info, StakeInfo>,

    #[account(mut)]
    signer: Signer<'info>,
}

#[account]
pub struct StakeInfo {
    pub staker: Pubkey,
    pub point: u64,
    pub reward: u64,
    pub stake_amount: u64,
    pub hold_amount: u64,
    pub duration: u64,
    pub stake_created_at: i64,
    pub hold_created_at: i64,
}

#[account]
pub struct Proposal {
    pub creator: Pubkey,
    pub statistr_id: String,
    pub hash: String,
    pub description: String,
    pub votes_yes: u64,
    pub votes_no: u64,
    pub reward: u64,
    pub creator_reward_rate: u64,
    pub creator_claimed_reward: bool,
    pub created_at: i64,
    pub duration: u64,
    pub voters: Vec<Pubkey>,
}

#[account]
pub struct Vote {
    pub owner: Pubkey,
    pub proposal_id: Pubkey,
    pub vote_weight: u64,
    pub vote_type: bool,
    pub claimed_rewards: bool,
    pub created_at: i64,
}
pub fn valid_proposal(proposal: &Proposal) -> Result<bool> {
    let mut result: bool = false;
    // let proposal = &ctx.accounts.proposal;
    if Clock::get()?.unix_timestamp - proposal.created_at > BASE_VOTING_DURATION.try_into().unwrap()
        && proposal.votes_yes + proposal.votes_no > MIN_VOTE_VALID
    {
        result = true;
    }
    Ok(result)
}

pub fn user_weight(point: u64) -> u64 {
    let mut weight: u64 = 0;
    if point > 10000 {
        weight = 20;
    } else if point > 1000 {
        weight = 15;
    } else if point > 100 {
        weight = 10;
    } else if point > 10 {
        weight = 5;
    }
    weight
}

pub fn determine_term(duration: u64) -> u64 {
    let mut _duration: u64 = BASE_STAKING_DURATION;
    if duration >= 3 * _duration {
        _duration = 3 * _duration;
    } else if duration >= 2 * _duration {
        _duration = 2 * _duration;
    }
    _duration
}

// const BASE_VOTING_DURATION: u64 = 24 * 60 * 60;
const BASE_VOTING_DURATION: u64 = 2 * 60;
// const BASE_STAKING_DURATION: u64 = 24 * 60 * 60 * 90;
const BASE_STAKING_DURATION: u64 = 1;

const MIN_VOTE_VALID: u64 = 10000000000;

const BASE_REWARD_FOR_PROPOSAL: u64 = 1000000000;
