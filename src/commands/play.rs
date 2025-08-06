use serenity::builder::{CreateCommand, CreateCommandOption};
use serenity::model::application::{CommandInteraction, CommandOptionType};
use serenity::prelude::Context;
use std::sync::Arc;

use musicbot::utils::{
    audio::*,
    response::*,
    localization::Text,
};

pub async fn run(ctx: &Context, command: &CommandInteraction) {
    if let Err(why) = command.defer(&ctx.http).await {
        eprintln!("Failed to defer interaction: {why:?}");
        normal_response(ctx, command, Some(Text::FailedToPlay), None).await;
        return;
    }

    let channel_id = match get_channel_to_join(ctx, command) {
        Ok(id) => id,
        Err(err) => return edit_response(ctx, command, Some(err), None).await,
    };

    let (track, metadata) = match process_query(ctx, command).await {
        Ok((mut track, metadata)) => {
            if channel_id.is_none() {
                track.user_data = Arc::new((metadata.clone(), Some(command.clone())));
            } else {
                track.user_data = Arc::new((metadata.clone(), None::<CommandInteraction>));
            }
            
            (track, metadata)
        },
        Err(_) => {
            edit_response(ctx, command, Some(Text::FailedToFetch), None).await;
            return;
        },
    };


    if let Some(id) = channel_id {
        if let Err(why) = join(ctx, command, id).await {
            edit_response(ctx, command, Some(why), None).await;
            return;
        }
    }

    if let Err(why) = play(ctx, command, track, metadata, channel_id.is_none()).await {
        edit_response(ctx, command, Some(why), None).await;
        return;
    }
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