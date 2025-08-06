mod commands;

use std::{
    env,
    collections::HashMap,
};
use dotenv::dotenv;

use serenity::{
    all::Command,
    async_trait,
    model::{
        application::Interaction,
        gateway::Ready,
    },
    prelude::*,
};

use songbird::SerenityInit;

use reqwest::Client as HttpClient;

use musicbot::utils::{
    audio::HttpKey,
    audio::MetadataCache,
    response::normal_response,
    localization::Text,
};

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        if let Interaction::Command(command) = interaction {
            match command.data.name.as_str() {
                "play" => commands::play::run(&ctx, &command).await,
                "skip" => commands::skip::run(&ctx, &command).await,
                "disconnect" => commands::disconnect::run(&ctx, &command).await,
                "change_channel" => commands::change_channel::run(&ctx, &command).await,
                "queue" => commands::queue::run(&ctx, &command).await,
                "clear_queue" => commands::clear_queue::run(&ctx, &command).await,
                "shuffle" => commands::shuffle::run(&ctx, &command).await,
                _ => normal_response(&ctx, &command, Some(Text::UnknownCommand), None).await,
            };
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
        ];

        for cmd in commands {
            let _ = Command::create_global_command(&ctx.http, cmd).await;
        }
    }
}

#[tokio::main]
async fn main() {
    dotenv().ok();

    let token = env::var("DISCORD_TOKEN").expect("env variable `DISCORD_TOKEN` should be set by `.env` file");

    let intents = GatewayIntents::GUILDS | GatewayIntents::GUILD_VOICE_STATES;

    let mut client =
        Client::builder(&token, intents)
        .event_handler(Handler)
        .register_songbird()
        .type_map_insert::<HttpKey>(HttpClient::new())
        .type_map_insert::<MetadataCache>(HashMap::new())
        .await
        .expect("Err creating client");

    if let Err(why) = client.start().await {
        eprintln!("Client error: {why:?}");
    }
}