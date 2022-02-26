use serenity::async_trait;
use serenity::client::{Client, Context, EventHandler};
use serenity::framework::standard::StandardFramework;
use serenity::model::guild::PartialGuild;
use serenity::model::{
    channel::{ChannelType, GuildChannel},
    gateway::Ready,
    id::GuildId,
    voice::VoiceState,
    prelude::{Member, Guild, ChannelId},
};
use dotenv::dotenv;
use std::env;

struct Handler;

async fn search_notf_channels(
    ctx: &Context,
    guild: GuildId,
) -> serenity::Result<Vec<GuildChannel>> {
    let mut notf_channels = Vec::new();
    let notf_channel_name = env::var("NOTF_CHANNEL_NAME").unwrap_or("vc-notf".to_string());
    let channel_map = guild.channels(ctx).await?;
    for (_channel_id, guild_channel) in channel_map {
        if guild_channel.name == notf_channel_name && guild_channel.kind == ChannelType::Text {
            notf_channels.push(guild_channel);
        }
    }
    Ok(notf_channels)
}


fn user_count(guild: &Guild, channel_id: ChannelId) -> usize {
    guild
        .voice_states
        .values()
        .filter(|state| { 
            match state.channel_id {
                Some(c) => c == channel_id,
                None => false,
            }
        })
        .count()
}


#[async_trait]
impl EventHandler for Handler {

    async fn voice_state_update(
        &self,
        ctx: Context,
        guild_id_opt: Option<GuildId>,
        old_vs_opt: Option<VoiceState>,
        new_vs: VoiceState,
    ) {
        if let Some(guild_id) = guild_id_opt {
            if let Ok(notf_channels) = search_notf_channels(&ctx, guild_id).await {
                if let Some(member) = new_vs.member {
                    let member_display_name = member.display_name();
                    let member_user_id = member.user.id;
                    let member_avatar_url = member.face();
                    if let Some(old_vs) = old_vs_opt {
                        if let Some(old_channel_id) = old_vs.channel_id {
                            if let Some(old_channel_name) = old_channel_id.name(&ctx).await {
                                if let Some(new_channel_id) = new_vs.channel_id {
                                    if let Some(new_channel_name) = new_channel_id.name(&ctx).await
                                    {
                                        if old_channel_id != new_channel_id {
                                            for notf_channel in notf_channels {
                                                notf_channel
                                                    .send_message(&ctx, |m| {
                                                        m.add_embed(|e| {
                                                            e.title(format!(
                                                                "{} moved VC!",
                                                                member_display_name
                                                            ))
                                                            .description(format!(
                                                                "<@{}> moved from {} to {}!",
                                                                member_user_id,
                                                                old_channel_name,
                                                                new_channel_name,
                                                            ))
                                                
                                                            .thumbnail(&member_avatar_url)
                                                        })
                                                    })
                                                    .await
                                                    .ok();
                                            }
                                        }
                                    }
                                } else {
                                    for notf_channel in notf_channels {
                                        notf_channel
                                            .send_message(&ctx, |m| {
                                                m.add_embed(|e| {
                                                    e.title(format!(
                                                        "{} left VC!",
                                                        member_display_name
                                                    ))
                                                    .description(format!(
                                                        "<@{}> left {}!",
                                                        member_user_id, old_channel_name,
                                                    ))
                                                    .thumbnail(&member_avatar_url)
                                                })
                                            })
                                            .await
                                            .ok();
                                    }
                                }
                            }
                        }
                    } else {
                        if let Some(new_channel_id) = new_vs.channel_id {
                            if let Some(new_channel_name) = new_channel_id.name(&ctx).await {
                                for notf_channel in notf_channels {
                                    notf_channel
                                        .send_message(&ctx, |m| {
                                            m.add_embed(|e| {
                                                e.title(format!(
                                                    "{} joined VC!",
                                                    member_display_name
                                                ))
                                                .description(format!(
                                                    "<@{}> joined {}!",
                                                    member_user_id, new_channel_name,
                                                ))
                                                .thumbnail(&member_avatar_url)
                                            })
                                        })
                                        .await
                                        .ok();
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    async fn ready(&self, _ctx: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);
    }
}





#[tokio::main]
async fn main() {
    dotenv().ok();
    let token = env::var("DISCORD_BOT_TOKEN").expect("Discord bot token missing!");
    let application_id: u64 = env::var("DISCORD_APPLICATION_ID")
        .expect("Discord application ID missing!")
        .parse()
        .expect("Invalid application ID");
    let framework = StandardFramework::new();
    let mut client = Client::builder(token)
        .event_handler(Handler)
        .application_id(application_id)
        .framework(framework)
        .await
        .expect("Error creating client");

    if let Err(why) = client.start().await {
        println!("An error occurred while running the client: {:?}", why);
    }
}
