use serenity::all::EditInteractionResponse;
use serenity::builder::{CreateCommand, CreateCommandOption};
use serenity::model::application::{CommandInteraction, CommandOptionType};
use serenity::prelude::Context;

use discordbot::utils::audio::{process_query, join, play};

pub async fn run(ctx: &Context, command: &CommandInteraction) -> Option<String> {
    command.defer(&ctx.http).await.ok()?;
    
    let (track, aux_metadata) = match process_query(ctx, command).await {
        Ok((input, metadata)) => (input, metadata),
        Err(err) => return Some(err),
    };

    if let Err(why) = join(ctx, command).await {
        return Some(why);
    }

    if let Err(why) = play(ctx, command, track).await {
        return Some(why);
    }

    let response = format!("Now playing: {}", aux_metadata.title.unwrap_or("Unknown title".to_string()));

    let builder = EditInteractionResponse::new()
        .content(response);

    if let Err(why) = command.edit_response(&ctx.http, builder).await {
        println!("Failed to edit interaction response: {}", why);
    }

    None
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