use serenity::builder::{CreateCommand, CreateCommandOption};
use serenity::model::application::{CommandInteraction, CommandOptionType};
use serenity::prelude::Context;

use discordbot::utils::audio::{join, play};

pub async fn run(ctx: &Context, command: &CommandInteraction) -> Option<String> {
    if let Err(why) = join(ctx, command).await {
        return Some(why);
    }

    if let Err(why) = play(ctx, command).await {
        return Some(why);
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