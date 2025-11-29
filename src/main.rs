mod commands;
mod utils;

use std::{
    env,
    collections::HashMap,
};

use serenity::prelude::*;

use songbird::SerenityInit;

use reqwest::Client as HttpClient;

use clap::Parser;

use crate::utils::{
    audio::{
        HttpKey,
        MetadataCache,
        FileCache,
    },
    event_handler::Handler,
    local_files::get_audio_files,
    cli::Config,
};

#[tokio::main]
async fn main() {
    let cli = Config::parse();

    let token = env::var("DISCORD_TOKEN").expect("env variable `DISCORD_TOKEN` should be set");

    let intents = GatewayIntents::GUILDS | GatewayIntents::GUILD_VOICE_STATES;

    let mut client =
        Client::builder(&token, intents)
        .event_handler(Handler)
        .register_songbird()
        .type_map_insert::<HttpKey>(HttpClient::new())
        .type_map_insert::<MetadataCache>(HashMap::new())
        .type_map_insert::<FileCache>(get_audio_files(&cli.audio_directory))
        .type_map_insert::<Config>(cli)
        .await
        .expect("Err creating client");

    if let Err(why) = client.start().await {
        eprintln!("Client error: {why:?}");
    }
}