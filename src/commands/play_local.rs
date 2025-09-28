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
    let channel_id = match get_channel_to_join(ctx, command) {
        Ok(id) => id,
        Err(err) => return normal_response(ctx, command, err.into()).await,
    };

    let (track, metadata) = match process_local_query(ctx, command).await {
        Ok((mut track, metadata)) => {
            if channel_id.is_none() {
                track.user_data = Arc::new((metadata.clone(), Some(command.clone())));
            } else {
                track.user_data = Arc::new((metadata.clone(), None::<CommandInteraction>));
            }
            
            (track, metadata)
        },
        Err(_) => {
            normal_response(ctx, command, Text::NoSuchFile.into()).await;
            return;
        },
    };

    if let Some(id) = channel_id {
        if let Err(why) = join(ctx, command, id).await {
            normal_response(ctx, command, why.into()).await;
            return;
        }
    }

    match play(ctx, command, track, metadata, channel_id.is_none()).await {
        Ok(embed) => normal_response(ctx, command, embed.into()).await,
        Err(why) => normal_response(ctx, command, why.into()).await,
    }
}

pub fn register() -> CreateCommand {
    CreateCommand::new("play_local")
        .description("Plays content from curated list")
        .name_localized("pl", "graj_lokalne")
        .description_localized("pl", "Odtwarza zawartość z przygotowanej listy")
        .add_option(
            CreateCommandOption::new(CommandOptionType::String, "query", "The name of a file to play")
                .name_localized("pl", "zapytanie")
                .description_localized("pl", "Nazwa pliku do odtworzenia")
                .required(true)
                .set_autocomplete(true)
        )
}

pub async fn autocomplete(ctx: &Context, partial: &str) -> Vec<String> {
    let data = ctx.data.read().await;
    let cache = data.get::<FileCache>()
        .cloned()
        .expect("Guaranteed to exist in the typemap.");

    cache.keys()
        .filter(|key| key.to_lowercase().contains(&partial.to_lowercase()))
        .take(25)
        .cloned()
        .collect()
}