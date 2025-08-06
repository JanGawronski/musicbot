use serenity::builder::CreateCommand;
use serenity::model::application::CommandInteraction;
use serenity::prelude::Context;

use musicbot::utils::{
    audio::*,
    response::*,
};

pub async fn run(ctx: &Context, command: &CommandInteraction) {
    let guild_id = match command.guild_id {
        Some(id) => id,
        None => {
            normal_response(ctx, command, Some("This command can only be used in a server.".to_string()), None).await;
            return;
        }
    };

    let manager = songbird::get(ctx)
        .await
        .expect("Songbird Voice client placed in at initialisation.")
        .clone();

    let handler_lock = match manager.get(guild_id) {
        Some(handler) => handler,
        None => {
            normal_response(ctx, command, Some("Not connected to a voice channel.".to_string()), None).await;
            return;
        },
    };

    let handler = handler_lock.lock().await;

    if handler.queue().is_empty() || handler.queue().len() == 1 {
        normal_response(ctx, command, Some("Queue is empty.".to_string()), None).await;
        return;
    }

    let queue_metadata = handler.queue().current_queue().iter().skip(1).map(|handle| {
        let data = handle.data::<(Metadata, Option<CommandInteraction>)>();
        data.0.clone()
    }).collect::<Vec<_>>();

    drop(handler);

    let embed = create_queue_embed(&queue_metadata);

    normal_response(ctx, command, None, Some(embed)).await;
}

pub fn register() -> CreateCommand {
    CreateCommand::new("queue")
        .description("Displays the queue")
        .name_localized("pl", "kolejka")
        .description_localized("pl", "Wyświetla kolejkę")
}