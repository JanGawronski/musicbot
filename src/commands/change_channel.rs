use serenity::builder::CreateCommand;
use serenity::model::application::CommandInteraction;
use serenity::prelude::Context;

use musicbot::utils::response::*;

pub async fn run(ctx: &Context, command: &CommandInteraction) {
    let guild_id = match command.guild_id {
        Some(id) => id,
        None => {
            normal_response(ctx, command, Some("This command can only be used in a server.".to_string()), None).await;
            return;
        }
    };

    let voice_states = match guild_id.to_guild_cached(&ctx.cache) {
        Some(guild) => Some(guild.voice_states.clone()),
        None => None,
    };

    let voice_states = match voice_states {
        Some(states) => states,
        None => {
            normal_response(ctx, command, Some("Failed to change voice channel.".to_string()), None).await;
            return;
        }
    };

    let channel_id = match voice_states.get(&command.user.id) {
        Some(state) => match state.channel_id {
            Some(id) => id,
            None => {
                normal_response(ctx, command, Some("You are not connected to a voice channel.".to_string()), None).await;
                return;
            }
        },
        None => {
            normal_response(ctx, command, Some("You are not connected to a voice channel.".to_string()), None).await;
            return;
        }
    };

    let manager = songbird::get(ctx)
        .await
        .expect("Songbird Voice client placed in at initialisation.");

    if let None = manager.get(guild_id) {
        normal_response(ctx, command, Some("Not connected to a voice channel.".to_string()), None).await;
        return;
    };

    match manager.join(guild_id, channel_id).await {
        Ok(_) => normal_response(ctx, command, Some("Changed voice channel.".to_string()), None).await,
        Err(why) => {
            eprintln!("Failed to change voice channel: {why:?}");
            normal_response(ctx, command, Some("Failed to change voice channel.".to_string()), None).await;
        },
    }

}

pub fn register() -> CreateCommand {
    CreateCommand::new("change_channel")
        .description("Joins the voice channel of requester")
        .name_localized("pl", "zmień_kanał")
        .description_localized("pl", "Przechodzi do kanału głosowego żądającego")
}