use serenity::{
    async_trait,
    model::application::{
        CommandInteraction, 
        CommandDataOptionValue
    },
    prelude::*
};

use songbird::{
    input::{
        YoutubeDl,
        Compose,
        Input,
        AuxMetadata
    },
    Event,
    EventContext,
    EventHandler as VoiceEventHandler,
    TrackEvent,
    tracks::Track
};
use reqwest::Client as HttpClient;

pub struct HttpKey;

impl TypeMapKey for HttpKey {
    type Value = HttpClient;
}

struct TrackStartNotifier;

#[async_trait]
impl VoiceEventHandler for TrackStartNotifier {
    async fn act(&self, ctx: &EventContext<'_>) -> Option<Event> {
        if let EventContext::Track(tracks) = ctx {
            for (state, handle) in *tracks {
                println!("{:?}", state);
                println!("{:?}", handle);
            }
        }
        
        None
    }
}



pub async fn join(ctx: &Context, command: &CommandInteraction) -> Result<(), String> {
    let guild_id = command.guild_id.ok_or("This command can only be used in a guild.")?;

    let voice_states = guild_id.to_guild_cached(&ctx.cache)
        .ok_or("Guild not found in cache.")?
        .voice_states
        .clone();

    if let Some(voice_state) = voice_states.get(&ctx.cache.current_user().id) {
        if voice_state.channel_id.is_some() {
            return Ok(());
        }
    }

    let channel_id = voice_states
        .get(&command.user.id)
        .and_then(|voice_state| voice_state.channel_id)
        .ok_or("You must be in a voice channel to use this command.")?;


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
            TrackStartNotifier
        );

    Ok(())
}

pub async fn play(ctx: &Context, command: &CommandInteraction, track: Track) -> Result<(), String> {
    let guild_id = command.guild_id.ok_or("This command can only be used in a guild.")?;

    let manager = songbird::get(ctx)
        .await
        .expect("Songbird Voice client placed in at initialisation.")
        .clone();

    let handler_lock = manager.get(guild_id).ok_or("No voice handler found for this guild.")?;
    let mut handler = handler_lock.lock().await;

    let _ = handler.play(track);


    Ok(())
}

pub async fn process_query(ctx: &Context, command: &CommandInteraction) -> Result<(Track, AuxMetadata), String> {
    let value = &command.data.options.get(0)
        .ok_or("No query provided.")?
        .value;

    let query = match value {
        CommandDataOptionValue::String(query) => query,
        _ => return Err("Query must be a string.".to_string()),
    };

    let http_client = {
        let data = ctx.data.read().await;
        data.get::<HttpKey>()
            .cloned()
            .expect("Guaranteed to exist in the typemap.")
    };

    let mut source = if query.contains("/") {
        YoutubeDl::new(http_client, query.clone())
    } else {
        YoutubeDl::new(http_client, format!("ytsearch:{}", query))
    };

    let input_fut = tokio::spawn(Input::from(source.clone()).make_live_async());

    let metadata = source.aux_metadata()
        .await
        .map_err(|e| format!("Failed to get metadata: {}", e))?;


    let input = input_fut.await
        .map_err(|e| format!("Failed to create input: {}", e))?
        .map_err(|e| format!("Failed to create input: {}", e))?;


    let track = Track::from(input);

    Ok((track, metadata))
}
