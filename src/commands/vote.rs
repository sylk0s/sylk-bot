use serde::{Serialize, Deserialize};
use chrono::{prelude::*, Duration};
use std::cmp::{PartialOrd, Ordering};
use std::collections::HashMap;

use crate::utils::cloud::{CloudSync, Unique};
use crate::{Context, Error};

use poise::serenity_prelude as serenity;

/// A command with two subcommands: `child1` and `child2`
///
/// Running this function directly, without any subcommand, is only supported in prefix commands.
/// Discord doesn't permit invoking the root command of a slash command if it has subcommands.

// AAA this is throwing really annoying errors in vs code with rust-analyzer
#[poise::command(slash_command, subcommands("post", "end", "cancel", "list", "debug"))]
pub async fn vote(_ctx: Context<'_>) -> Result<(), Error> {
    // THIS WILL NEVER BE CALLED EVER!!!
    Ok(())
}

/// A subcommand of `parent`
/// Posts a new vote
#[poise::command(slash_command)]
pub async fn post(ctx: Context<'_>,
    #[description = "The name of the vote"]
    name: String,
    #[description = "A description for the vote"]
    description: Option<String>,
    #[description = "Minutes to add to the vote timer"]    
    minutes: u32,
    #[description = "Hours to add to the vote timer"]
    hours: u32
) -> Result<(), Error> {
    let votes = &mut ctx.data().write().await.votes;
    let status;
    if let Ok(new_vote) = Vote::on_vote_create(
            ctx,
            name, 
            if let Some(desc) = description { desc } else { String::new() }, 
            ctx.author().id.0,
            ctx.guild_id().unwrap().0,
            ctx.channel_id().into(),
            hours,
            minutes,
        ).await {
            (*votes).insert(new_vote.uuid(), new_vote);  
            status = "Vote Created";
        } else {
            status = "Error creating a new vote";
        };
        ctx.send(|m| m.content(format!("{status}")).ephemeral(true)).await?; 
    Ok(())
}       

#[poise::command(slash_command)]
pub async fn end(ctx: Context<'_>,
    #[description = "The UUID of the vote"]
    uuid: u128,
) -> Result<(), Error> {
    let votes = &mut ctx.data().write().await.votes;
    if let Some(vote) = votes.get(&(uuid as u64)) {
        vote.on_vote_end(ctx).await.unwrap();
        votes.remove(&(uuid as u64));
        ctx.send(|m| m.content(format!("Vote successfully ended")).ephemeral(true)).await?;
    } else {
        ctx.send(|m| m.content(format!("Invalid Vote ID")).ephemeral(true)).await?;
    }
    
    Ok(())
}

/// A subcommand of vote
/// Quietly ends a vote
#[poise::command(slash_command)]
pub async fn cancel(ctx: Context<'_>,
    #[description = "The UUID of the vote"]
    // Stupid weird bug here
    // TODO explore this more later
    uuid: u128,
) -> Result<(), Error> {
    let votes = &mut ctx.data().write().await.votes;
    if let Some(vote) = votes.get(&(uuid as u64)) {

        let msg = ctx.serenity_context().http.get_message(vote.ch_id,vote.msg_id).await.unwrap(); 
        msg.delete(ctx).await?;

        vote.clrm().await.unwrap();

        votes.remove(&(uuid as u64));
        
        ctx.send(|m| m.content(format!("Vote successfully canceled")).ephemeral(true)).await?;
    } else {
        ctx.send(|m| m.content(format!("Invalid Vote ID")).ephemeral(true)).await?;
    }
    Ok(())
}

//TODO error with unwrap on null values
// stop writing STUPID CODE

/// A subcommand of `parent`
/// Lists the current votes
#[poise::command(slash_command)]
pub async fn list(ctx: Context<'_>) -> Result<(), Error> {    
    let data = ctx.data().write().await;
    let votelist = data.votes.clone().into_values().map(|v| v.name.clone()).fold(String::new(), |a, b| { format!("{}\n{}", a, b) });
    ctx.send(|m| {
        m.embed(|e| {
            e.title("List of votes:")
                .description(votelist)
        })
    }).await?;
    Ok(())
}

/// A subcommand of `parent`
/// For testing; gets the debug information about votes
#[poise::command(slash_command)]
pub async fn debug(ctx: Context<'_>) -> Result<(), Error> {
    let data = ctx.data().write().await;
    // maps local votes into a string
    let votelist = data.votes.clone().into_values().map(|v| v.name.clone()).fold(String::new(), |a, b| { format!("{}\n{}", a, b) });
    // maps external votes into a string
    let votelistcl = Vote::clget().await?.iter().map(|v| v.name.clone()).fold(String::new(), |a, b| { format!("{}\n{}", a, b) });
    
    ctx.send(|m| {
        m.embed(|e| {
            e.title("State out")
                .field("Local:", votelist, false)
                .field("Cloud:", votelistcl, false)
        })
    }).await?;
    Ok(())
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Vote {
    name:String,
    desc:String,
    msg_id:u64,
    ch_id:u64,
    guild_id:u64,
    creator:u64,
    end_time:DateTime<Utc>,
}

impl Vote {
    pub fn new(name: String, desc: String, msg_id: u64, ch_id: u64, guild_id: u64, creator: u64, end_time: DateTime<Utc>) -> Self {
        Vote {
            name,
            desc,
            msg_id,
            ch_id,
            guild_id,
            creator,
            end_time
        }
    }

    // Initialized the vote in all sorts of places
    async fn on_vote_create<H>(
        http: H, 
        //state: State,
        name: String, 
        desc: String, 
        creator: u64, 
        guild_id: u64, 
        ch_id: u64, 
        hrs: u32,
        min: u32,
    ) -> Result<Vote, Box<dyn std::error::Error>> where H: serenity::http::CacheHttp {

        // Calculates the end time of the vote
        let end_time = Utc::now() + Duration::minutes((hrs*60 + min).into());

        // Gets the author member to access their nickname and image
        let auth = http.http().get_member(guild_id, creator).await.unwrap();
        
        // gets the channel to send the message
        let channel = http.http().get_channel(ch_id).await.unwrap();    

        // sends message
        let mut msg = channel.id().send_message(&http.http(), |m| {
            m.embed(|e| {
                e.title(name.clone())
                    .description(desc.clone())
                    .timestamp(end_time)
                    // replaces author's username with nickname if one exists
                    .author(|a| {
                        a.name(if let Some(nick) = auth.nick.clone() {
                            nick
                        } else {
                            auth.user.name.clone()
                        })
                        .icon_url(auth.user.avatar_url().unwrap())
                })
            })
        }).await?;

        // Creates the new Vote object
        let v: Vote = Vote::new(name.clone(), desc.clone(), msg.id.0, ch_id, guild_id, creator, end_time);

        // Updates the message with the actual information
        // So that we can use the msg_id as the UUID for the vote
        msg.edit(&http, |m| {
            m.embed(|e| {
                e.title(name.clone())
                    .description(desc.clone())
                    .field("UUID:", format!("{}", v.uuid()), true)
                    .timestamp(end_time)
                    // replaces author's username with nickname if one exists
                    .author(|a| {
                        a.name(if let Some(nick) = auth.nick {
                            nick
                        } else {
                            auth.user.name.clone()
                        })
                        .icon_url(auth.user.avatar_url().unwrap())
                })
            })
        }).await.unwrap();

        // adds reactions for voting
        msg.react(&http, '👍').await?;
        msg.react(&http, '👎').await?;
        msg.react(&http, '🤚').await?;

        // push vote to cloud list of votes
        v.clsave("votes").await.expect("Cloud sync failed");

        Ok(v)
    }

    // Cleans up a vote, publishes the results
    async fn on_vote_end<H>(&self, http: H) -> Result<(), String> where H: serenity::CacheHttp {
        // determine winner
        let mut msg = http.http().get_message(self.ch_id,self.msg_id).await.unwrap(); 

        // initializes dumb counting vars
        let mut yes = 0;
        let mut no = 0;
        let mut abs = 0;

        // counts reactons for the y/n/m totals
        // ignores the initial reactions by the bot
        for r in msg.reactions.iter() {
            if r.reaction_type == serenity::ReactionType::from('👍') {
                yes = r.count-1;
            }
            if r.reaction_type == serenity::ReactionType::from('👎') {
                no = r.count-1; 
            }
            if r.reaction_type == serenity::ReactionType::from('🤚') {
                abs = r.count-1;
            }
        }

        let passed = yes > no;

        // Author of the vote
        let auth = http.http().get_member(self.guild_id, self.creator).await.unwrap();

        // changes the message to display result
        msg.edit(http, |m| {
                m.embed(|e| {
                    e.title(format!("Vote {} : {}", if passed {"passed"} else {"failed"}, self.name))
                        .description(self.desc.clone())
                        .color(if passed {0x00ff00} else {0xff0000})
                        .field("Yes:", format!("{yes}"), true)
                        .field("No:", format!("{no}"), true)
                        .field("Abstain:", format!("{abs}"), true)
                        .author(|a| {
                            // same in the initial message, uses nickname if one exists
                            a.name(if let Some(nick) = auth.nick {
                                nick
                            } else {
                                auth.user.name.clone()
                            })
                            .icon_url(auth.user.avatar_url().unwrap())
                        })
                        .timestamp(self.end_time)
                })
        }).await.unwrap();

        // removes vote from the cloud
        if let Err(_e) = self.clrm().await {
            return Err(String::from("Error syncing with the cloud"));
        }
        
        Ok(())
    }

    // Returns the votes from the cloud that haven't ended
    pub async fn reload<H>(http: H) -> Result<HashMap<u64, Vote>, Box<dyn std::error::Error>> where H: serenity::CacheHttp + Copy {
        let vs = if let Ok(votes) = Vote::clhash().await { votes } 
            else { println!("Error reading votes from the cloud :("); HashMap::new() };
        Ok(Self::end_finished_votes(vs, http).await)
    }

    // returns a list of all the votes that have not ended
    // ends all of the votes that have fainished
    pub async fn end_finished_votes<H>(votes: HashMap<u64, Vote>, http: H) -> HashMap<u64, Vote>  where H: serenity::CacheHttp + Copy {
        let mut result = HashMap::new();
        for (id, v) in votes {
            if v.ended() {
                v.on_vote_end(http.clone()).await.unwrap();
                continue;
            }
            result.insert(id, v.clone());
        }
        result
    }

    // see if a vote has ended
    fn ended(&self) -> bool {
        Some(Ordering::Less) == self.end_time.partial_cmp(&Utc::now())
    }
}

impl CloudSync for Vote {
    fn clname() -> &'static str {
        "votes"
    }
}

impl Unique for Vote {
    fn uuid(&self) -> u64 {
        self.msg_id 
    }
}
