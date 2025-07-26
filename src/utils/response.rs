use serenity::all::{
    Context,
    CommandInteraction,
    CreateInteractionResponseMessage,
    CreateInteractionResponse,
    EditInteractionResponse,
    CreateInteractionResponseFollowup,
    CreateEmbed,
    CreateEmbedAuthor,
};
use super::audio::Metadata;

pub async fn normal_response(ctx: &Context, command: &CommandInteraction, string: Option<String>, embed: Option<CreateEmbed>) {
    let message = if let Some(embed) = embed {
            CreateInteractionResponseMessage::new().embed(embed)
        } else if let Some(string) = string {
            CreateInteractionResponseMessage::new().content(string)
        } else {
            println!("No content or embed provided for edit_response");
            return;
        };

    let builder = CreateInteractionResponse::Message(message);

    if let Err(why) = command.create_response(&ctx.http, builder).await {
        println!("Failed to create interaction response: {}", why);
    }
}

pub async fn edit_response(ctx: &Context, command: &CommandInteraction, string: Option<String>, embed: Option<CreateEmbed>) {
    let builder = if let Some(embed) = embed {
            EditInteractionResponse::new().embed(embed)
        } else if let Some(string) = string {
            EditInteractionResponse::new().content(string)
        } else {
            println!("No content or embed provided for edit_response");
            return;
        };

    if let Err(why) = command.edit_response(&ctx.http, builder).await {
        println!("Failed to edit interaction response: {}", why);
    }
}


pub async fn followup_response(ctx: &Context, command: &CommandInteraction, embed: CreateEmbed) {
    let builder = CreateInteractionResponseFollowup::new()
        .embed(embed);

    if let Err(why) = command.create_followup(&ctx.http, builder).await {
        println!("Failed to create followup response: {}", why);
    }

}


pub fn create_track_embed(metadata: &Metadata, queue_length: usize, is_now_playing: bool) -> CreateEmbed {
    let mut embed = CreateEmbed::new();

    if let Some(track) = &metadata.track {
        embed = embed.title(track);
    } else if let Some(title) = &metadata.title {
        embed = embed.title(title);
    } else {
        embed = embed.title("Unknown Track");
    }

    if let Some(source_url) = &metadata.webpage_url {
        embed = embed.url(source_url);
    }

    if let Some(thumbnail) = &metadata.thumbnail {
        embed = embed.thumbnail(thumbnail);
    }

    if let Some(artist) = &metadata.artist {
        embed = embed.field("Artist", artist, true);
    } else if let Some(author) = &metadata.uploader {
        embed = embed.field("Author", author, true);
    }
    
    
    if let Some(duration) = metadata.duration {
        let hours = duration / 3600;
        let minutes = (duration % 3600) / 60;
        let seconds = duration % 60;

        let string = if hours > 0 {
                format!("{hours}:{minutes:02}:{seconds:02}")
            } else {
                format!("{minutes}:{seconds:02}")
            };

        embed = embed.field("Duration", string, true);
    }

    if queue_length > 0 {
        embed = embed.field("Queue length", queue_length.to_string(), true);
    }

    if is_now_playing {
        embed = embed.author(CreateEmbedAuthor::new("Now playing"));
    } else {
        embed = embed.author(CreateEmbedAuthor::new("Added to queue"));
    }

    embed
}