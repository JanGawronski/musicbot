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

use super::{
    audio::Metadata,
    localization::Text,
};

pub async fn normal_response(ctx: &Context, command: &CommandInteraction, text: Option<Text>, embed: Option<CreateEmbed>) {
    let message = if let Some(embed) = embed {
            CreateInteractionResponseMessage::new().embed(embed)
        } else if let Some(text) = text {
            CreateInteractionResponseMessage::new().content(text.localization(&command.locale))
        } else {
            eprintln!("No content or embed provided for normal_response");
            return;
        };

    let builder = CreateInteractionResponse::Message(message);

    if let Err(why) = command.create_response(&ctx.http, builder).await {
        eprintln!("Failed to create interaction response: {why:?}");
    }
}

pub async fn edit_response(ctx: &Context, command: &CommandInteraction, text: Option<Text>, embed: Option<CreateEmbed>) {
    let builder = if let Some(embed) = embed {
            EditInteractionResponse::new().embed(embed)
        } else if let Some(text) = text {
            EditInteractionResponse::new().content(text.localization(&command.locale))
        } else {
            eprintln!("No content or embed provided for edit_response");
            return;
        };

    if let Err(why) = command.edit_response(&ctx.http, builder).await {
        eprintln!("Failed to edit interaction response: {why:?}");
    }
}

pub async fn followup_response(ctx: &Context, command: &CommandInteraction, embed: CreateEmbed) {
    let builder = CreateInteractionResponseFollowup::new()
        .embed(embed);

    if let Err(why) = command.create_followup(&ctx.http, builder).await {
        eprintln!("Failed to create followup response: {why:?}");
    }

}

pub fn create_track_embed(metadata: &Metadata, queue_length: usize, is_now_playing: bool, locale: &String) -> CreateEmbed {
    let mut embed = CreateEmbed::new();

    if let Some(track) = &metadata.track {
        embed = embed.title(track);
    } else if let Some(title) = &metadata.title {
        embed = embed.title(title);
    } else {
        embed = embed.title(Text::UnknownTitle.localization(locale));
    }

    if let Some(source_url) = &metadata.webpage_url {
        embed = embed.url(source_url);
    }

    if let Some(thumbnail) = &metadata.thumbnail {
        embed = embed.thumbnail(thumbnail);
    }

    if let Some(artist) = &metadata.artist {
        embed = embed.field(Text::Artist.localization(locale), artist, true);
    } else if let Some(author) = &metadata.uploader {
        embed = embed.field(Text::Author.localization(locale), author, true);
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

        embed = embed.field(Text::Duration.localization(locale), string, true);
    }

    if queue_length > 0 {
        embed = embed.field(Text::QueueLength.localization(locale), queue_length.to_string(), true);
    }

    if is_now_playing {
        embed = embed.author(CreateEmbedAuthor::new(Text::NowPlaying.localization(locale)));
    } else {
        embed = embed.author(CreateEmbedAuthor::new(Text::AddedToQueue.localization(locale)));
    }

    embed
}

pub fn create_queue_embed(queue: &[Metadata], locale: &String) -> CreateEmbed {
    let mut embed = CreateEmbed::new().title(Text::Queue.localization(locale));

    let unknown_title = Text::UnknownTitle.localization(locale);

    let titles = queue.iter().take(50).map(|m| 
        if let Some(track) = &m.track {
            track
        } else if let Some(title) = &m.title {
            title
        } else {
            &unknown_title
        }
    ).collect::<Vec<_>>();

    let chunk_titles = titles.chunks_exact(2);

    let chunk_remainder = chunk_titles.remainder();

    for (index, chunk) in chunk_titles.enumerate() {
        embed = embed.field(
            format!("{}. {}", 2 * index + 1, chunk[0]), 
            format!("{}. {}", 2 * index + 2, chunk[1]), 
            false
        );
    }

    if chunk_remainder.len() == 1 {
        embed = embed.field(format!("{}. {}", titles.len(), chunk_remainder[0]), "", false);
    }

    embed
}