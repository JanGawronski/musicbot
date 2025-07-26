use serenity::{
    async_trait,
    model::{
        application::{
            CommandDataOptionValue, CommandInteraction
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
        YoutubeDl,
        Input,
    },
    Songbird,
    Event,
    EventContext,
    EventHandler,
    TrackEvent,
    CoreEvent,
    tracks::Track
};

use reqwest::Client as HttpClient;

use std::{
    ops::Deref,
    sync::Arc,
    process::Command,
};

use super::response::{
    followup_response,
    create_track_embed,
    edit_response,
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

                let embed = create_track_embed(metadata, queue_length, true);

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
                        println!("Failed to leave voice channel: {}", why);
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
                    println!("Failed to remove voice handler: {}", why);
                }
        }
        None
    }
}

pub fn get_channel_to_join(ctx: &Context, command: &CommandInteraction) -> Result<Option<ChannelId>, String> {
    let guild_id = command.guild_id.ok_or("This command can only be used in a guild.")?;
    
    let voice_states = guild_id.to_guild_cached(&ctx.cache)
        .ok_or("Guild not found in cache.")?
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
        .ok_or("You must be in a voice channel to use this command.")?;

    Ok(Some(channel_id))
}

pub async fn join(ctx: &Context, command: &CommandInteraction, channel_id: ChannelId) -> Result<(), String> {
    let guild_id = command.guild_id.ok_or("This command can only be used in a guild.")?;

    let manager = songbird::get(ctx)
        .await
        .expect("Songbird Voice client placed in at initialisation.")
        .clone();


    let handle_lock = manager.join(guild_id, channel_id)
        .await
        .map_err(|e| {
        format!("Failed to join voice channel: {}", e)
    })?;

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

pub async fn play(ctx: &Context, command: &CommandInteraction, track: Track, metadata: Metadata, add_to_queue: bool) -> Result<(), String> {
    let guild_id = command.guild_id.ok_or("This command can only be used in a guild.")?;

    let manager = songbird::get(ctx)
        .await
        .expect("Songbird Voice client placed in at initialisation.")
        .clone();

    let handler_lock = manager.get(guild_id).ok_or("No voice handler found for this guild.")?;
    let mut handler = handler_lock.lock().await;

    handler.enqueue(track).await;

    let embed = create_track_embed(&metadata, handler.queue().len() - 1, !add_to_queue);

    drop(handler);

    edit_response(ctx, command, None, Some(embed)).await;

    Ok(())
}

pub async fn process_query(ctx: &Context, command: &CommandInteraction) -> Result<(Track, Metadata), String> {
    let value = &command.data.options.get(0)
        .ok_or("No query provided.")?
        .value;

    let query = match value {
        CommandDataOptionValue::String(query) => query,
        _ => return Err("Query must be a string.".to_string()),
    };

    let ytdlp_query = if query.contains("/") {
        query.to_string()
    } else {
        format!("ytsearch:{}", query)
    };

    let ytdlp_output = Command::new("./yt-dlp")
        .args(["--format", 
            "bestaudio/best",
            "--ignore-config",
            "--no-playlist",
            "--no-download",
            "--dump-json",
            ytdlp_query.as_str()
            ])
        .output()
        .map_err(|e| format!("Failed to run yt-dlp: {}", e))?;

    let metadata: Metadata = serde_json::from_slice(&ytdlp_output.stdout)
        .map_err(|e| format!("yt-dlp output was not valid UTF-8: {}", e))?;

    let http_client = {
        let data = ctx.data.read().await;
        data.get::<HttpKey>()
            .cloned()
            .expect("Guaranteed to exist in the typemap.")
    };

    let source = if let Some(ref url) = metadata.url {
            YoutubeDl::new_ytdl_like("./yt-dlp", http_client, url.clone())
                .user_args(vec!["--no-config".to_string()])
        } else {
            return Err("No valid URL found in metadata.".to_string());
        };

    let input = Input::from(source).make_live_async()
        .await
        .map_err(|e| format!("Failed to create live input: {}", e))?;

    let track = Track::from(input);

    Ok((track, metadata))
}
