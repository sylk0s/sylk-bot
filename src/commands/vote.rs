use serenity::framework::standard::macros::{command, group};
use serenity::framework::standard::{CommandResult, Args};
use serenity::model::prelude::*;
use serenity::prelude::*;

use serde::{Serialize, Deserialize};
use chrono::prelude::*;
//use std::cmp::{PartialOrd, Ordering};

use crate::VoteContainer;
use crate::utils::cloud::{CloudSync, Unique};

#[group]
#[prefixes("vote", "v")]
#[default_command(list)]
#[commands(post, cancel, end, list, debug)]
struct Voting;

// Post a new vote
#[command]
async fn post(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    let ar = args.raw().collect::<Vec<&str>>();
    let v = Vote::on_vote_create(ar[0].to_string(), ar[1..].iter().map(|a| a.to_string()).reduce(
            |a, b| { format!("{} {}",a ,b) }
            ).unwrap(), msg.author.id.0, ctx, &msg.channel_id).await;
    let mut data = ctx.data.write().await;
    let votes = data.get_mut::<VoteContainer>().unwrap();
    votes.push(v);
    Ok(())
}

// Force the end of a vote with an id
#[command]
async fn end(ctx: &Context, msg: &Message) -> CommandResult {
    let mut data = ctx.data.write().await;
    let votes = data.get_mut::<VoteContainer>().unwrap();
    votes[0].name = String::from("c");
    println!("{:?}", votes);
    Ok(())
}

// Quitely end a vote with no result handling
#[command]
async fn cancel(ctx: &Context, msg: &Message) -> CommandResult {
    let v: Vote = Vote::new(String::from("testvote"), String::from("testing"), 1234);
    v.clsave::<Vote>("votes").await?;
    msg.reply(&ctx.http, "it works").await?;
    Ok(())
}

// List the current active votes
#[command]
async fn list(ctx: &Context, msg: &Message) -> CommandResult {
    let mut data = ctx.data.write().await;
    let votes = data.get_mut::<VoteContainer>().unwrap();
    let votelist = votes.iter().map(|v| v.name.clone()).reduce(|a, b| { format!("{}\n{}", a, b) }).unwrap();
    msg.channel_id.send_message(&ctx.http, |m| {
        m.embed(|e| {
            e.title("List of votes:")
                .description(votelist)
        })
    }).await?;
    Ok(())
}

#[command]
async fn debug(ctx: &Context, msg: &Message) -> CommandResult {
    let cloud_votes = Vote::clget::<Vote>().await?;
    let mut data = ctx.data.write().await;
    let bot_votes = data.get_mut::<VoteContainer>().unwrap();
    let m = msg.channel_id.send_message(&ctx.http, |m| {
        m.embed(|e| {
            e.title("Voting Debug")
            .field("Cloud votes", format!("{:?}", cloud_votes), false)
            .field("Local votes", format!("{:?}", bot_votes), false)
        })
    }).await?;
    m.react(&ctx.http, '👀').await?;
    Ok(())
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Vote {
    name:String,
    description:String,
    id:u64,
    creator:u64,
    end_time:DateTime<Utc>,
}

impl Vote {
    pub fn new(name:String, description:String, creator:u64) -> Vote {
        Vote {
            name,
            description,
            id: 0,
            creator,
            end_time: Utc::now(),
        }
    }

    // Initialized the vote in all sorts of places
    async fn on_vote_create(name: String, desc: String, creator: u64, ctx: &Context, ch_id: &ChannelId) -> Vote {
        let mut v: Vote = Vote::new(name.clone(), desc.clone(), creator);
        v.clsave::<Vote>("votes").await;
        let me = ch_id.send_message(&ctx.http, |m| {
            m.embed(|e| {
                e.title(name)
                    .description(desc)
            })
        }).await.expect("aaa");
        me.react(&ctx.http, '👍').await;
        me.react(&ctx.http, '👎').await;
        v.id = me.id.0;
        v
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

impl CloudSync for Vote {
    fn clname<Vote>() -> &'static str {
        "votes"
    }
}

impl Unique for Vote {
    fn uuid(&self) -> u64 {
        self.id 
    }
}
