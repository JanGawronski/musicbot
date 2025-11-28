use serenity::{
    all::{
        Command,
        CreateAutocompleteResponse,
        CreateInteractionResponse,
        AutocompleteChoice,
    },
    async_trait,
    model::{
        application::{
            Interaction,
            CommandDataOptionValue,
        },
        gateway::Ready,
    },
    prelude::*,
};

use crate::commands;

use super::{
    response::normal_response,
    localization::Text,
};

pub struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        match interaction {
            Interaction::Command(command) => {
                match command.data.name.as_str() {
                    "play" => commands::play::run(&ctx, &command).await,
                    "skip" => commands::skip::run(&ctx, &command).await,
                    "disconnect" => commands::disconnect::run(&ctx, &command).await,
                    "change_channel" => commands::change_channel::run(&ctx, &command).await,
                    "queue" => commands::queue::run(&ctx, &command).await,
                    "clear_queue" => commands::clear_queue::run(&ctx, &command).await,
                    "shuffle" => commands::shuffle::run(&ctx, &command).await,
                    "play_local" => commands::play_local::run(&ctx, &command).await,
                    _ => normal_response(&ctx, &command, Text::UnknownCommand.into()).await,
                }
            },
            Interaction::Autocomplete(command) => {
                let value = match &command.data.options.first() {
                    Some(option) => &option.value,
                    None => {
                        eprintln!("No options found in {command:?}");
                        return;
                    }
                };

                let query = match value {
                    CommandDataOptionValue::Autocomplete {  value: query, .. } => query,
                    _ => {
                        eprintln!("Expected a string query, got: {value:?}");
                        return;
                    },
                };

                let autocomplete = match command.data.name.as_str() {
                    "play_local" => commands::play_local::autocomplete(&ctx, query).await,
                    _ => Vec::new(),
                };
                
                let autocomplete_response = CreateAutocompleteResponse::new()
                    .set_choices(
                        autocomplete.into_iter()
                            .map(|choice| AutocompleteChoice::new(choice.clone(), choice))
                            .collect::<Vec<AutocompleteChoice>>()
                    );

                let response = CreateInteractionResponse::Autocomplete(autocomplete_response);

                if let Err(e) = command.create_response(&ctx.http, response).await {
                    eprintln!("Failed to create autocomplete response: {e:?}");
                }
            }
            _ => {},
        }
    }

    async fn ready(&self, ctx: Context, _: Ready) {
        let commands = vec![
            commands::play::register(),
            commands::skip::register(),
            commands::disconnect::register(),
            commands::change_channel::register(),
            commands::queue::register(),
            commands::clear_queue::register(),
            commands::shuffle::register(),
            commands::play_local::register(),
        ];

        for cmd in commands {
            let _ = Command::create_global_command(&ctx.http, cmd).await;
        }
    }
}