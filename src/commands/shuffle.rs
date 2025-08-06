use serenity::builder::CreateCommand;
use serenity::model::application::CommandInteraction;
use serenity::prelude::Context;

use rand::seq::SliceRandom;

use musicbot::utils::response::*;

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
    
    handler.queue().modify_queue(|queue| {
        let mut rng = rand::rng();
        let mut items = queue.drain(1..).collect::<Vec<_>>();
        items.shuffle(&mut rng);
        queue.extend(items);
    });

    drop(handler);

    normal_response(ctx, command, Some("Shuffled.".to_string()), None).await;
}

pub fn register() -> CreateCommand {
    CreateCommand::new("shuffle")
        .description("Shuffles queue")
        .name_localized("pl", "przetasuj")
        .description_localized("pl", "Przetasowuje kolejkę")
}