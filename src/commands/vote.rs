use serde::{Serialize, Deserialize};
use chrono::{prelude::*, Duration};
use std::cmp::{PartialOrd, Ordering};
use futures::stream::StreamExt;

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
                  #[description = "The name of the vote"]
                  name: String,
                  #[description = "A description for the vote"]
                  description: Option<String>,
                  #[description = "Minutes to add to the vote timer"]    
                  minutes: u32,
                  #[description = "Hours to add to the vote timer"]
                  hours: u32
) -> Result<(), Error> {
    Vote::on_vote_create(ctx, name, description.unwrap(), // fix this for no description option 
        ctx.author().id.0,
        ctx.guild_id().unwrap().0,
        ctx.channel_id().0,
        hours,
        minutes,
        ).await.expect("Vote creation failed");
    ctx.say("Vote created").await?; // add epethmeral response
    Ok(())
}

/// Another subcommand of `parent`
/// Forces the end of a vote
#[poise::command(prefix_command, slash_command)]
pub async fn end(ctx: Context<'_>) -> Result<(), Error> {
    ctx.say("You invoked the second child command!").await?;
    Ok(())
}

/// A subcommand of `parentu,hjc vb8ucv 2 eik,`
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
        let me = channel.id().send_message(&http.http(), |m| {
            m.embed(|e| {
                e.title(name.clone())
                    .description(desc.clone())
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
        }).await?;

        // adds reactions for voting
        me.react(&http, '👍').await?;
        me.react(&http, '👎').await?;
        me.react(&http, '🤚').await?;

        // Creates the new Vote object
        let v: Vote = Vote::new(name.clone(), desc.clone(), creator, me.id.0, ch_id, guild_id, end_time);

        // push vote to cloud list of votes
        v.clsave::<Vote>("votes").await.expect("Cloud sync failed");

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

        // changes the message to display result information
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
        if let Err(_e) = self.clrm::<Vote>("votes").await {
            return Err(String::from("Error syncing with the cloud"));
        }

        // make it auto remove the vote >:(
        Ok(())
    }

    // do we want to remove the message here too?
    // this is a filter map because a filter was throwing weird lifetime errors :\
    // returns a list of votes without all votes that have id (although it should only be one)
    async fn remove_vote(id: u64, votes: Vec<Vote>) -> Vec<Vote> {
        futures::stream::iter(votes.into_iter()).filter_map(|v| async move { 
            if v.is_vote(id) {
                v.clrm::<Vote>("votes").await.unwrap();
                None
            } else { Some(v) }
        }).collect().await
    }

    // checks if a vote has the given id
    fn is_vote(&self, id: u64) -> bool {
        self.msg_id == id
    }

    // Returns the votes from the cloud that haven't ended
    pub async fn reload<H>(http: H) -> Result<Vec<Vote>, Box<dyn std::error::Error>> where H: serenity::CacheHttp + Copy {
        let t: Vec<Vote> = Self::clget().await.unwrap();
        Ok(Self::end_finished_votes(t, http).await)
    }
    
    // returns a list of all the votes that have not ended
    // ends all of the votes that have fainished
    pub async fn end_finished_votes<H>(votes: Vec<Vote>, http: H) -> Vec<Vote> where H: serenity::CacheHttp + Copy {
        futures::stream::iter(votes.into_iter()).filter_map(|v| async move {
            if v.ended() {
                v.on_vote_end(http.clone()).await.unwrap();
                None
            } else {
                Some(v)
            }
        }).collect().await
    }

    // see of a vote has ended
    fn ended(&self) -> bool {
        Some(Ordering::Less) == self.end_time.partial_cmp(&Utc::now())
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
