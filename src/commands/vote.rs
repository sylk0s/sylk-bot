use serde::{Serialize, Deserialize};
use chrono::{prelude::*, Duration};
use std::cmp::{PartialOrd, Ordering};
use std::sync::{Arc, Mutex};

use crate::utils::cloud::{CloudSync, Unique};
use crate::{Context, Error, State};

use poise::serenity_prelude as serenity;

/// A command with two subcommands: `child1` and `child2`
///
/// Running this function directly, without any subcommand, is only supported in prefix commands.
/// Discord doesn't permit invoking the root command of a slash command if it has subcommands.
#[poise::command(prefix_command, slash_command, subcommands("post", "end", "cancel", "list", "debug"))]
pub async fn vote(ctx: Context<'_>) -> Result<(), Error> {
    ctx.say("Please select a subcommand").await?;
    Ok(())
}

/// A subcommand of `parent`
/// Posts a new vote
#[poise::command(prefix_command, slash_command)]
pub async fn post(ctx: Context<'_>,
                  #[description("The name of the vote")] 
                  name: String,
                  #[description("A description for the vote")]
                  description: Option<String>,
                  #[description("Minutes to add to the vote timer")]    
                  minutes: u16,
                  #[description("Hours to add to the vote timer")]
                  hours: u16
) -> Result<(), Error> {
    Vote::on_vote_create(name, description, 
        ctx.author.id.0,
        ctx.guild_id.unwrap().0,
        ctx,
        ctx.channel_id,
        hours,
        minutes,
        ).await.expect("Vote creation failed");
    ctx.send("Vote created").await?; // add epethmeral response
    Ok(())
}

/// Another subcommand of `parent`
/// Forces the end of a vote
#[poise::command(prefix_command, slash_command)]
pub async fn end(ctx: Context<'_>) -> Result<(), Error> {
    ctx.say("You invoked the second child command!").await?;
    Ok(())
}

/// A subcommand of `parent`
/// Quietly ends a vote
#[poise::command(prefix_command, slash_command)]
pub async fn cancel(ctx: Context<'_>) -> Result<(), Error> {
    ctx.say("You invoked the first child command!").await?;
    Ok(())
}

/// A subcommand of `parent`
/// Lists the current votes
#[poise::command(prefix_command, slash_command)]
pub async fn list(ctx: Context<'_>) -> Result<(), Error> {
    ctx.say("You invoked the first child command!").await?;
    Ok(())
}

/// A subcommand of `parent`
/// For testing; gets the debug information about votes
#[poise::command(prefix_command, slash_command)]
pub async fn debug(ctx: Context<'_>) -> Result<(), Error> {
    ctx.say("You invoked the first child command!").await?;
    Ok(())
}

/*
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
        ar[3..].iter().map(|a| a.to_string()).reduce(|a, b| { format!("{} {}",a ,b) }).unwrap(), 
        msg.author.id.0,
        msg.guild_id.unwrap().0,
        ctx,
        &msg.channel_id,
        ar[1].parse::<u32>().unwrap(),
        ar[2].parse::<u32>().unwrap(),
        ).await.expect("Vote creation failed");
    Ok(())
}

// Force the end of a vote with an id
#[command]
async fn end(ctx: &Context, msg: &Message) -> CommandResult {
    let mut data = ctx.data.write().await;
    let votes = data.get_mut::<VoteContainer>().unwrap();
    if let Some(r) = &msg.referenced_message {
        let i = Vote::get_vote_index(r.id.try_into().unwrap(), &votes).unwrap();
        let v = &votes[i];
        v.on_vote_end(&ctx.http).await.unwrap();
        Vote::remove_vote(i.try_into().unwrap(), &ctx).await.unwrap();
    } else {
        msg.reply(&ctx.http, "Didn't reply to a message!").await?;
    };
    Ok(())
}

// Quitely end a vote with no result handling
#[command]
async fn cancel(ctx: &Context, msg: &Message) -> CommandResult {
    let mut data = ctx.data.write().await;
    let votes = data.get_mut::<VoteContainer>().unwrap();
    if let Some(r) = &msg.referenced_message {
        let i = Vote::get_vote_index(r.id.try_into().unwrap(), &votes).unwrap();
        Vote::remove_vote(i.try_into().unwrap(), &ctx).await.unwrap();
        msg.delete(&ctx.http).await.unwrap();
    } else {
        msg.reply(&ctx.http, "Didn't reply to a message!").await?;
    };
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
*/

#[derive(Serialize, Deserialize, Debug)]
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

    // rewrite this crap to be better
    // Initialized the vote in all sorts of places
    async fn on_vote_create<H>(
        http: H, 
        state: State,
        name: String, 
        desc: String, 
        creator: u64, 
        guild_id: u64, 
        ch_id: u64, 
        hrs: u32,
        min: u32,
    ) -> Result<(), Box<dyn std::error::Error>> where H: serenity::http::CacheHttp {

        let end_time = Utc::now() + Duration::minutes((hrs*60 + min).into());

        // Gets the author member to access their nickname and image
        let auth = http.http().get_member(guild_id, creator).await.unwrap();
        
        // gets the channel to send the message
        let channel = http.http().get_channel(ch_id).await.unwrap();    

        // sends message
        let me = channel.id().send_message(&http.http(), |m| {
            m.embed(|e| {
                e.title(name)
                    .description(desc)
                    .timestamp(end_time)
                    .author(|a| {
                        a.name(if let Some(nick) = auth.nick {
                            nick
                        } else {
                            auth.user.name.clone()
                        })
                        .icon_url(auth.user.avatar_url().unwrap())
                    })
            })
        }).await?;

        // adds reactions
        me.react(&http, '👍').await?;
        me.react(&http, '👎').await?;
        me.react(&http, '🤚').await?;

        // Creates the new Vote object
        let mut v: Vote = Vote::new(name.clone(), desc.clone(), creator, me.id.0, ch_id, guild_id, end_time);

        // push vote to cloud list of votes
        v.clsave::<Vote>("votes").await.expect("Cloud sync failed");

        // push vote to internal list of votes
        let mut data = state.write().await;
        let votes = data.get_mut::<VoteContainer>().unwrap();
        votes.push(v);

        Ok(())
    }

    // Cleans up a vote, publishes the results
    async fn on_vote_end<H>(&self, http: H) -> Result<(), String> where H: serenity::CacheHttp {
        // determine winner
        let mut msg = http.http().get_message(self.ch_id,self.msg_id).await.unwrap(); 

        let mut yes = 0;
        let mut no = 0;
        let mut abs = 0;

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

        let auth = http.http().get_member(self.guild_id, self.creator).await.unwrap();
        msg.edit(http, |m| {
                m.embed(|e| {
                    e.title(format!("Vote {} : {}", if passed {"passed"} else {"failed"}, self.name))
                        .description(self.desc.clone())
                        .color(if passed {0x00ff00} else {0xff0000})
                        .field("Yes:", format!("{yes}"), true)
                        .field("No:", format!("{no}"), true)
                        .field("Abstain:", format!("{abs}"), true)
                        .author(|a| {
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

        if let Err(_e) = self.clrm::<Vote>("votes").await {
            return Err(String::from("Error syncing with the cloud"));
        }
        Ok(())
    }

    async fn remove_vote<H>(id: u64, http: H, state: State) -> Result<(), String> where H: serenity::CacheHttp {
        // HELP
        let mut data = state.write().await;
        let votes = data.get_mut::<VoteContainer>().unwrap();
        for i in 0..votes.len() {
            if votes[i].msg_id == id {
                votes.remove(i).clrm::<Vote>("votes").await.unwrap();
                return Ok(());
            }
        } 
        Err(String::from("Vote not found"))
    }

    fn get_vote_index(id: u64, votes: &Vec<Vote>) -> Result<usize, String> {
        for i in 0..votes.len() {
            if votes[i].msg_id == id {
                return Ok(i);
            }
        } 
        Err(String::from("Vote not found"))
    }

    // implement proper time comparison
    pub async fn reload<H>(http: H, state: State) -> Result<(), Box<dyn std::error::Error>> where H: serenity::CacheHttp + Copy {
        // Get both sources of votes
        let mut t: Vec<Vote> = Self::clget().await.unwrap();
        let mut data = state.write().await;
        let votes = data.get_mut::<VoteContainer>().unwrap();

        // original things weren't actually breaking ;)
        while t.len() != 0 {
            votes.push(t.remove(0));
        }
       
        Self::check_votes_over(votes, http).await;
        Ok(())
    }
    
    pub async fn check_votes_over<H>(votes: &mut Vec<Vote>, http: H) where H: serenity::CacheHttp + Copy {
        let mut to_remove = Vec::new();
        // mark for deletion (god i hate this)
        for i in 0..votes.len() {
            if let Some(Ordering::Less) = votes[i].end_time.partial_cmp(&Utc::now()) {
                votes[i].on_vote_end(http).await.unwrap();
                to_remove.push(i);
            }
        }

        for i in to_remove.iter().rev() {
            votes.remove(*i);
            println!("removed vote");
        }
    }
}

impl CloudSync for Vote {
    fn clname<Vote>() -> &'static str {
        "votes"
    }
}

impl Unique for Vote {
    fn uuid(&self) -> u64 {
        self.msg_id 
    }
}
