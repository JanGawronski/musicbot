use serenity::all::EditInteractionResponse;
use serenity::async_trait;
use serenity::model::application::{CommandInteraction, CommandDataOptionValue};
use serenity::prelude::*;
use songbird::input::Compose;
use songbird::{
    input::YoutubeDl,
    Event,
    EventContext,
    EventHandler as VoiceEventHandler,
    TrackEvent,
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

pub async fn play(ctx: &Context, command: &CommandInteraction) -> Result<(), String> {
    let guild_id = command.guild_id.ok_or("This command can only be used in a guild.")?;

    let value = &command.data.options.get(0)
        .ok_or("You must provide a URL to play.")?
        .value;

    let query = match value {
        CommandDataOptionValue::String(s) => s.clone(),
        _ => return Err("Incorrect argument.".to_string()),
    };

    let manager = songbird::get(ctx)
        .await
        .expect("Songbird Voice client placed in at initialisation.")
        .clone();

    let http_client = {
        let data = ctx.data.read().await;
        data.get::<HttpKey>()
            .cloned()
            .expect("Guaranteed to exist in the typemap.")
    };

    let handler_lock = manager.get(guild_id).ok_or("No voice handler found for this guild.")?;
    let mut handler = handler_lock.lock().await;

    command.defer(&ctx.http).await.map_err(|e| {
        format!("Failed to defer command response: {}", e)
    })?;

    let mut source = YoutubeDl::new(http_client, query);

    let _ = handler.play_input(source.clone().into());

    let metadata = source .aux_metadata()
        .await
        .map_err(|e| format!("Failed to get metadata: {}", e))?;


    let content = format!("Now playing: {}", metadata.title.unwrap_or("Unknown Title".to_string()));

    let data = EditInteractionResponse::new().content(content);
    if let Err(why) = command.edit_response(&ctx.http, data).await {
        println!("Cannot respond to slash command: {why}");
    }
    

    Ok(())
}
