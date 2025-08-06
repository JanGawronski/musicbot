use serenity::builder::CreateCommand;
use serenity::model::application::CommandInteraction;
use serenity::prelude::Context;

use musicbot::utils::{
    response::*,
    localization::Text,
};

pub async fn run(ctx: &Context, command: &CommandInteraction) {
    let guild_id = match command.guild_id {
        Some(id) => id,
        None => {
            normal_response(ctx, command, Some(Text::CommandOnlyInGuild), None).await;
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
            normal_response(ctx, command, Some(Text::FailedToChangeChannel), None).await;
            return;
        }
    };

    let channel_id = match voice_states.get(&command.user.id) {
        Some(state) => match state.channel_id {
            Some(id) => id,
            None => {
                normal_response(ctx, command, Some(Text::UserMustBeInVoiceChannel), None).await;
                return;
            }
        },
        None => {
            normal_response(ctx, command, Some(Text::UserMustBeInVoiceChannel), None).await;
            return;
        }
    };

    let manager = songbird::get(ctx)
        .await
        .expect("Songbird Voice client placed in at initialisation.");

    if let None = manager.get(guild_id) {
        normal_response(ctx, command, Some(Text::BotMustBeInVoiceChannel), None).await;
        return;
    };

    match manager.join(guild_id, channel_id).await {
        Ok(_) => normal_response(ctx, command, Some(Text::ChangedChannel), None).await,
        Err(why) => {
            eprintln!("Failed to change voice channel: {why:?}");
            normal_response(ctx, command, Some(Text::FailedToChangeChannel), None).await;
        },
    }

}

pub fn register() -> CreateCommand {
    CreateCommand::new("change_channel")
        .description("Joins the voice channel of requester")
        .name_localized("pl", "zmień_kanał")
        .description_localized("pl", "Przechodzi do kanału głosowego żądającego")
}