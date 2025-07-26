use serenity::builder::{CreateCommand, CreateCommandOption};
use serenity::model::application::{CommandInteraction, CommandOptionType};
use serenity::prelude::Context;
use std::sync::Arc;

use discordbot::utils::{
    audio::*,
    response::*,
};

pub async fn run(ctx: &Context, command: &CommandInteraction) {
    if let Err(e) = command.defer(&ctx.http).await {
        println!("Failed to defer interaction: {}", e);
        normal_response(ctx, command, "Server error".to_string()).await;
        return;
    }

    let channel_id = match get_channel_to_join(ctx, command) {
        Ok(id) => id,
        Err(err) => return edit_response(ctx, command, err).await,
    };

    let (track, aux_metadata) = match process_query(ctx, command).await {
        Ok((mut track, aux_metadata)) => {
            if channel_id.is_none() {
                track.user_data = Arc::new((aux_metadata.clone(), Some(command.clone())));
            } else {
                track.user_data = Arc::new((aux_metadata.clone(), None::<CommandInteraction>));
            }
            
            (track, aux_metadata)
        },
        Err(err) => {
            println!("Failed to process query: {}", err);
            edit_response(ctx, command, "Failed to fetch song.".to_string()).await;
            return;
        },
    };


    if let Some(id) = channel_id {
        if let Err(why) = join(ctx, command, id).await {
            println!("Failed to join voice channel: {}", why);
            edit_response(ctx, command, "Failed to join voice channel.".to_string()).await;
            return;
        }
    }

    if let Err(why) = play(ctx, command, track).await {
        println!("Failed to play track: {}", why);
        edit_response(ctx, command, "Failed to play track.".to_string()).await;
        return;
    }

    edit_response(ctx, command, format!("Now playing: {}", aux_metadata.title.unwrap_or("Unknown title".to_string()))).await;
}

pub fn register() -> CreateCommand {
    CreateCommand::new("play")
        .description("Plays music from given url or search term")
        .name_localized("pl", "graj")
        .description_localized("pl", "Odtwarza muzykę z podanego adresu URL lub wyszukiwanej frazy")
        .add_option(
            CreateCommandOption::new(CommandOptionType::String, "query", "The URL or search term to play")
                .name_localized("pl", "zapytanie")
                .description_localized("pl", "Adres URL lub wyszukiwana fraza do odtwarzania")
                .required(true)
        )
}