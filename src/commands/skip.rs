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

    let manager = songbird::get(ctx)
        .await
        .expect("Songbird Voice client placed in at initialisation.")
        .clone();

    let handler_lock = match manager.get(guild_id) {
        Some(handler) => handler,
        None => {
            normal_response(ctx, command, Some("Not connected to a voice channel.".to_string()), None).await;
            return;
        },
    };

    let handler = handler_lock.lock().await;

    if handler.queue().is_empty() {
        normal_response(ctx, command, Some("No tracks to skip.".to_string()), None).await;
        return;
    }

    if let Err(why) = handler.queue().skip() {
        println!("Failed to skip track: {}", why);
        normal_response(ctx, command, Some("Failed to skip track.".to_string()), None).await;
        return;
    }

    drop(handler);

    normal_response(ctx, command, Some("Skipped.".to_string()), None).await;
}

pub fn register() -> CreateCommand {
    CreateCommand::new("skip")
        .description("Skips the currently playing track")
        .name_localized("pl", "pomiń")
        .description_localized("pl", "Niezwłocznie przechodzi do następnego utworu")
}