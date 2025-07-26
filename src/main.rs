mod commands;

use std::env;
use dotenv::dotenv;

use serenity::{
    async_trait,
    model::{
        application::Interaction,
        gateway::Ready,
        id::GuildId,
    },
    prelude::*,
};

use songbird::SerenityInit;

use reqwest::Client as HttpClient;

use discordbot::utils::{
    audio::HttpKey,
    response::normal_response,
};

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        if let Interaction::Command(command) = interaction {
            match command.data.name.as_str() {
                "play" => commands::play::run(&ctx, &command).await,
                _ => normal_response(&ctx, &command, Some("Unknown command".to_string()), None).await,
            };
        }
    }

    async fn ready(&self, ctx: Context, _: Ready) {
        let guild_id = GuildId::new(
            env::var("GUILD_ID")
                .expect("Expected GUILD_ID in environment")
                .parse()
                .expect("GUILD_ID must be an integer"),
        );

        let _ = guild_id 
            .set_commands(&ctx.http, vec![
                commands::play::register(),
            ])
            .await;
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
        .await
        .expect("Err creating client");

    if let Err(why) = client.start().await {
        println!("Client error: {why:?}");
    }
}