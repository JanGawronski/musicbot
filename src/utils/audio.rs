use serenity::{
    async_trait,
    model::{
        application::{
            CommandDataOptionValue, 
            CommandInteraction
        },
        id::{
            ChannelId,
            GuildId
        },
    },
    prelude::*
};

use songbird::{
    input::{
        HttpRequest,
        Input,
    }, 
    tracks::Track, 
    CoreEvent, 
    Event, 
    EventContext, 
    EventHandler, 
    Songbird, 
    TrackEvent
};

use reqwest::Client as HttpClient;

use std::{
    collections::HashMap, 
    ops::Deref, 
    process::Command, 
    sync::Arc,
};

use super::{
    response::{
        followup_response,
        create_track_embed,
        edit_response,
    },
    localization::Text,
};

use serde::Deserialize;

#[derive(Deserialize, Clone)]
pub struct Metadata {
    pub title: Option<String>,
    pub uploader: Option<String>,
    pub track: Option<String>,
    pub artist: Option<String>,
    pub duration: Option<u32>,
    pub thumbnail: Option<String>,
    pub webpage_url: Option<String>,
    pub url: Option<String>,
}

pub struct HttpKey;

impl TypeMapKey for HttpKey {
    type Value = HttpClient;
}

pub struct MetadataCache;

impl TypeMapKey for MetadataCache {
    type Value = HashMap<String, Metadata>;
}

struct TrackStartNotifier {
    ctx: Context,
    guild_id: GuildId,
}

#[async_trait]
impl EventHandler for TrackStartNotifier {
    async fn act(&self, ctx: &EventContext<'_>) -> Option<Event> {
        if let EventContext::Track([(_, handle)]) = ctx {
            let data = handle.data::<(Metadata, Option<CommandInteraction>)>();
            let (metadata, some_command) = data.deref();
            if let Some(command) = some_command {
                let manager = songbird::get(&self.ctx)
                    .await
                    .expect("Songbird Voice client placed in at initialisation.")
                    .clone();

                let queue_length = if let Some(handler_lock) = manager.get(self.guild_id) {
                        let handler = handler_lock.lock().await;
                        handler.queue().len() - 1
                    } else {
                        0
                    };

                let embed = create_track_embed(metadata, queue_length, true, &command.locale);

                followup_response(&self.ctx, &command, embed).await;
            }
        }
        None
    }
}

struct TrackEndNotifier {
    manager: Arc<Songbird>,
    guild_id: GuildId,
}

#[async_trait]
impl EventHandler for TrackEndNotifier {
    async fn act(&self, ctx: &EventContext<'_>) -> Option<Event> {
        if let EventContext::Track(_) = ctx {
            if let Some(handler_lock) = self.manager.get(self.guild_id) {
                if handler_lock.lock().await.queue().is_empty() {
                    if let Err(why) = self.manager.leave(self.guild_id).await {
                        eprintln!("Failed to leave voice channel: {why:?}");
                    }
                }
            }
        }
        
        None
    }
}

struct DriverDisconnectNotifier {
    manager: Arc<Songbird>,
    guild_id: GuildId,
}

#[async_trait]
impl EventHandler for DriverDisconnectNotifier {
    async fn act(&self, ctx: &EventContext<'_>) -> Option<Event> {
        if let EventContext::DriverDisconnect(_) = ctx {
            if let Err(why) = self.manager.remove(self.guild_id).await {
                    eprintln!("Failed to remove voice handler: {why:?}");
                }
        }
        None
    }
}

pub fn get_channel_to_join(ctx: &Context, command: &CommandInteraction) -> Result<Option<ChannelId>, Text> {
    let guild_id = command.guild_id.ok_or(Text::CommandOnlyInGuild)?;
    
    let voice_states = guild_id.to_guild_cached(&ctx.cache)
        .ok_or(Text::FailedToJoin)?
        .voice_states
        .clone();

    if let Some(voice_state) = voice_states.get(&ctx.cache.current_user().id) {
        if voice_state.channel_id.is_some() {
            return Ok(None);
        }
    }

    let channel_id = voice_states
        .get(&command.user.id)
        .and_then(|voice_state| voice_state.channel_id)
        .ok_or(Text::UserMustBeInVoiceChannel)?;

    Ok(Some(channel_id))
}

pub async fn join(ctx: &Context, command: &CommandInteraction, channel_id: ChannelId) -> Result<(), Text> {
    let guild_id = command.guild_id.ok_or(Text::CommandOnlyInGuild)?;

    let manager = songbird::get(ctx)
        .await
        .expect("Songbird Voice client placed in at initialisation.")
        .clone();


    let handle_lock = manager.join(guild_id, channel_id)
        .await
        .map_err(|why| {
            eprintln!("Failed to join voice channel: {why:?}");
            Text::FailedToJoin
        }
    )?;

    let mut handle = handle_lock.lock().await;

    handle.add_global_event(
        Event::Track(TrackEvent::Play),
        TrackStartNotifier {
            ctx: ctx.clone(),
            guild_id: guild_id,
        }
    );

    handle.add_global_event(
        Event::Track(TrackEvent::End),
        TrackEndNotifier {
            manager: manager.clone(),
            guild_id,
        }
    );

    handle.add_global_event(
        Event::Core(CoreEvent::DriverDisconnect),
        DriverDisconnectNotifier {
            manager: manager,
            guild_id,
        }
    );
        
    Ok(())
}

pub async fn play(ctx: &Context, command: &CommandInteraction, track: Track, metadata: Metadata, add_to_queue: bool) -> Result<(), Text> {
    let guild_id = command.guild_id.ok_or(Text::CommandOnlyInGuild)?;

    let manager = songbird::get(ctx)
        .await
        .expect("Songbird Voice client placed in at initialisation.")
        .clone();

    let handler_lock = match manager.get(guild_id) {
        Some(handler) => handler,
        None => {
            return Err(Text::BotMustBeInVoiceChannel);
        }
    };

    let mut handler = handler_lock.lock().await;

    handler.enqueue(track).await;

    let embed = create_track_embed(&metadata, handler.queue().len() - 1, !add_to_queue, &command.locale);

    drop(handler);

    edit_response(ctx, command, embed.into()).await;

    Ok(())
}

pub async fn process_query(ctx: &Context, command: &CommandInteraction) -> Result<(Track, Metadata), ()> {
    let value = match &command.data.options.get(0) {
        Some(option) => &option.value,
        None => {
            eprintln!("No options found in {command:?}");
            return Err(());
        }
    };

    let query = match value {
        CommandDataOptionValue::String(query) => query,
        _ => {
            eprintln!("Expected a string query, got: {value:?}");
            return Err(());
        },
    };

    let metadata = match fetch_metadata(ctx, query).await {
        Ok(metadata) => metadata,
        Err(_) => return Err(()),
    };

    let http_client = {
        let data = ctx.data.read().await;
        data.get::<HttpKey>()
            .cloned()
            .expect("Guaranteed to exist in the typemap.")
    };

    let source = if let Some(ref url) = metadata.url {
            HttpRequest::new(http_client, url.clone())
        } else {
            eprintln!("No URL found in metadata: {:?}", metadata.url);
            return Err(());
        };

    let input = match Input::from(source).make_live_async().await {
        Ok(input) => input,
        Err(why) => {
            eprintln!("Failed to create live input: {why:?}");
            return Err(());
        }
    };

    let track = Track::from(input);

    Ok((track, metadata))
}

async fn fetch_metadata(ctx: &Context, query: &String) -> Result<Metadata, ()> {
    let http_client = {
        let data = ctx.data.read().await;
        data.get::<HttpKey>()
            .cloned()
            .expect("Guaranteed to exist in the typemap.")
    };

    if let Some(metadata) = ctx.data.read().await
        .get::<MetadataCache>()
        .expect("Guaranteed to exist in the typemap.")
        .get(query) {
        let response = match http_client
            .head(metadata.url.as_deref().unwrap_or(""))
            .send()
            .await {
            Ok(response) => response,
            Err(why) => {
                eprintln!("Failed to send HEAD request: {why:?}");
                return Err(());
            }
        };

        if response.status().is_success() {
            return Ok(metadata.clone());
        }
    }

    let metadata = match fetch_metadata_ytdlp(query) {
        Ok(metadata) => metadata,
        Err(_) => return Err(()),
    };
    
    let mut data = ctx.data.write().await;

    let cache = data.get_mut::<MetadataCache>().expect("Guaranteed to exist in the typemap.");

    cache.insert(query.to_string(), metadata.clone());

    if let Some(ref webpage_url) = metadata.webpage_url {
        cache.insert(webpage_url.clone(), metadata.clone());
    }
    
    Ok(metadata)
}

fn fetch_metadata_ytdlp(query: &String) -> Result<Metadata, ()> {
    let ytdlp_query = if query.contains("/") {
        query.to_string()
        } else {
            format!("ytsearch:{}", query)
        };

    let ytdlp_output = match Command::new("./yt-dlp")
        .args(["--format", 
            "bestaudio/best",
            "--cookies",
            "cookies.txt",
            "--ignore-config",
            "--no-playlist",
            "--no-download",
            "--dump-json",
            ytdlp_query.as_str()
            ])
        .output() {
        Ok(output) => output,
        Err(why) => {
            eprintln!("Failed to run yt-dlp: {why:?}");
            return Err(());
        }
    };

    let metadata = match serde_json::from_slice(&ytdlp_output.stdout) {
        Ok(metadata) => metadata,
        Err(why) => {
            eprintln!("Failed to parse yt-dlp output: {why:?}");
            return Err(());
        }
    };

    Ok(metadata)
}
