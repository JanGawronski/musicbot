use serenity::async_trait;
use serenity::model::application::CommandInteraction;
use serenity::prelude::*;
use songbird::{
    Event,
    EventContext,
    EventHandler as VoiceEventHandler,
    TrackEvent,
};

struct TrackStartNotifier;

#[async_trait]
impl VoiceEventHandler for TrackStartNotifier {
    async fn act(&self, ctx: &EventContext<'_>) -> Option<Event> {
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