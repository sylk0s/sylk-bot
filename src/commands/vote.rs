use serenity::framework::standard::macros::{command, group};
use serenity::framework::standard::CommandResult;
use serenity::model::prelude::*;
use serenity::prelude::*;

use serde::{Serialize, Deserialize};
use chrono::prelude::*;
use std::cmp::{PartialOrd, Ordering};

//use crate::utils::cloud::{CloudSync, Unique};

#[group]
#[prefixes("vote", "v")]
#[default_command(list)]
#[commands(post, force, end, list)]
struct Voting;

// Post a new vote
#[command]
async fn post(ctx: &Context, msg: &Message) -> CommandResult {
    msg.reply(&ctx.http, "aaa").await?;
    Ok(())
}

// Force the end of a vote with an id
#[command]
async fn force(ctx: &Context, msg: &Message) -> CommandResult {
    msg.reply(&ctx.http, "aaa").await?;
    Ok(())
}

// Quitely end a vote with no result handling
#[command]
async fn end(ctx: &Context, msg: &Message) -> CommandResult {
    msg.reply(&ctx.http, "aaa").await?;
    Ok(())
}

// List the current active votes
#[command]
async fn list(ctx: &Context, msg: &Message) -> CommandResult {
    msg.reply(&ctx.http, "aaa").await?;
    Ok(())
    /*
    let mut data = ctx.data.write().await;
    let votevec = data.get::<Vec<Vote>>().unwrap();
    msg.reply("AAA {} ", votevec[0]);

    Ok(())
    */
}

#[derive(Serialize, Deserialize)]
pub struct Vote {
    name:String,
    description:String,
    id:u32,
    creator:u32,
    end_time:DateTime<Utc>,
}

impl Vote {
    pub fn new(name:String, description:String, id:u32, creator:u32) -> Vote {
        Vote {
            name,
            description,
            id,
            creator,
            end_time: Utc::now(),
        }
    }

    // Initialized the vote in all sorts of places
    fn on_vote_create() -> () {
        unimplemented!();
    }

    // Cleans up a vote, publishes the results
    fn on_vote_end(&self) -> () {
        unimplemented!();
    }

    fn update_database() {
        unimplemented!();
    }
/*
    // implement proper time comparison
    fn reload(votestr: &String, vote_list: &mut Vec<Vote>) {
        let t: Vec<Vote> = Self::clget();
        for vote in t {
            match vote.end_time.partial_cmp(&Utc::now()) {
                Some(Ordering::Greater) => vote_list.push(vote),
                _ => vote.on_vote_end(),
            }
        }
    }
*/

    // Prints out the list of votes?
    // tldr; basically a subcommand i need to figure out
    fn query(votes: Vec<Vote>) {
        unimplemented!(); // list out votes
    }

    // Function to check if the vote has ended
    // use tokio when i get around to it
    fn check_votes_end() {
        unimplemented!();
    }

}
/*
impl CloudSync for Vote {
    fn clname() -> String {
        String::from("votes")
    }
}

impl Unique for Vote {
    fn uuid(&self) -> u32 {
        self.id 
    }
}
*/
