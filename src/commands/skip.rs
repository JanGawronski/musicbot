use serenity::builder::CreateCommand;
use serenity::model::application::CommandInteraction;
use serenity::prelude::Context;

use crate::utils::{
    response::*,
    localization::Text,
};

pub async fn run(ctx: &Context, command: &CommandInteraction) {
    let guild_id = match command.guild_id {
        Some(id) => id,
        None => {
            normal_response(ctx, command, Text::CommandOnlyInGuild.into()).await;
            return;
        }
    };

    let manager = songbird::get(ctx)
        .await
        .expect("Songbird Voice client placed in at initialisation.")
        .clone();

    let handler_lock = match manager.get(guild_id) {
        Some(handler) => handler,
        None => {
            normal_response(ctx, command, Text::BotMustBeInVoiceChannel.into()).await;
            return;
        },
    };

    let handler = handler_lock.lock().await;

    if handler.queue().is_empty() {
        normal_response(ctx, command, Text::QueueEmpty.into()).await;
        return;
    }

    if let Err(why) = handler.queue().skip() {
        println!("Failed to skip track: {}", why);
        normal_response(ctx, command, Text::FailedToSkip.into()).await;
        return;
    }

    drop(handler);

    normal_response(ctx, command, Text::Skipped.into()).await;
}

pub fn register() -> CreateCommand {
    CreateCommand::new("skip")
        .description("Skips the currently playing track")
        .name_localized("pl", "pomiń")
        .description_localized("pl", "Niezwłocznie przechodzi do następnego utworu")
}