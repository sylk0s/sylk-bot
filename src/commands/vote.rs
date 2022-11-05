use serenity::framework::standard::macros::{command, group};
use serenity::framework::standard::{CommandResult, Args};
use serenity::model::prelude::*;
use serenity::prelude::*;

use serde::{Serialize, Deserialize};
use chrono::{prelude::*, Duration};
use std::cmp::{PartialOrd, Ordering};
use std::error::Error;

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
    Vote::on_vote_create(
        ar[0].to_string(), 
        ar[1..].iter().map(|a| a.to_string()).reduce(|a, b| { format!("{} {}",a ,b) }).unwrap(), 
        msg.author.id.0, 
        ctx,
        &msg.channel_id).await.expect("Vote creation failed");
    Ok(())
}

// Force the end of a vote with an id
#[command]
async fn end(ctx: &Context, _msg: &Message, args: Args) -> CommandResult {
    let _ar = args.raw().collect::<Vec<&str>>();
    let mut data = ctx.data.write().await;
    let votes = data.get_mut::<VoteContainer>().unwrap();
    votes[0].name = String::from("c");
    Ok(())
}

// Quitely end a vote with no result handling
#[command]
async fn cancel(ctx: &Context, _msg: &Message, args: Args) -> CommandResult {
    let ar = args.raw().collect::<Vec<&str>>();
    Vote::remove_vote(ar[0].parse::<u64>().unwrap(), ctx).await?;
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
            end_time: Utc::now() + Duration::minutes(1),
        }
    }

    // Initialized the vote in all sorts of places
    async fn on_vote_create(name: String, desc: String, creator: u64, ctx: &Context, ch_id: &ChannelId) -> Result<(), Box<dyn Error>>{
        let mut v: Vote = Vote::new(name.clone(), desc.clone(), creator);

        // sends message
        let me = ch_id.send_message(&ctx.http, |m| {
            m.embed(|e| {
                e.title(name)
                    .description(desc)
            })
        }).await?;

        // adds reactions
        me.react(&ctx.http, '👍').await?;
        me.react(&ctx.http, '👎').await?;

        // Update vote data with id of msg
        v.id = me.id.0;

        // push vote to cloud list of votes
        v.clsave::<Vote>("votes").await.expect("Cloud sync failed");

        // push vote to internal list of votes
        let mut data = ctx.data.write().await;
        let votes = data.get_mut::<VoteContainer>().unwrap();
        votes.push(v);

        Ok(())
    }

    // Cleans up a vote, publishes the results
    async fn on_vote_end(&self) -> Result<(), String> {
        // determine winner
        
        // update final message
        

        if let Err(_e) = self.clrm::<Vote>("votes").await {
            return Err(String::from("Error syncing with the cloud"));
        }
        Ok(())
    }

    async fn remove_vote(id: u64, ctx: &Context) -> Result<(), String> {
        let mut data = ctx.data.write().await;
        let votes = data.get_mut::<VoteContainer>().unwrap();
        for i in 0..votes.len() {
            if votes[i].id == id {
                votes.remove(i).clrm::<Vote>("votes").await.unwrap();
                return Ok(());
            }
        } 
        Err(String::from("Vote not found"))
    }

    // implement proper time comparison
    pub async fn reload(ctx: &Context) -> Result<(), Box<dyn Error>> {
        // Get both sources of votes
        let mut t: Vec<Vote> = Self::clget().await.unwrap();
        let mut data = ctx.data.write().await;
        let votes = data.get_mut::<VoteContainer>().unwrap();

        // original things weren't actually breaking ;)
        while t.len() != 0 {
            votes.push(t.remove(0));
        }
       
        Self::check_votes_over(votes).await;
        Ok(())
    }
    
    pub async fn check_votes_over(votes: &mut Vec<Vote>) {
        let mut to_remove = Vec::new();
        // mark for deletion (god i hate this)
        for i in 0..votes.len() {
            if let Some(Ordering::Less) = votes[i].end_time.partial_cmp(&Utc::now()) {
                votes[i].on_vote_end().await.unwrap();
                to_remove.push(i);
            }
        }

        for i in to_remove.iter().rev() {
            votes.remove(*i);
            println!("removed vote");
        }
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
