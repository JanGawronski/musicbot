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
        .expect("Songbird Voice client placed in at initialisation.");

    match manager.leave(guild_id).await {
        Ok(_) => normal_response(ctx, command, Some("Disconnected.".to_string()), None).await,
        Err(why) => {
            eprintln!("Failed to disconnect: {why:?}");
            normal_response(ctx, command, Some("Failed to disconnect".to_string()), None).await;
        }
    }
}

pub fn register() -> CreateCommand {
    CreateCommand::new("disconnect")
        .description("Disconnects from the voice channel")
        .name_localized("pl", "rozłącz")
        .description_localized("pl", "Rozłącza z kanału głosowego")
}