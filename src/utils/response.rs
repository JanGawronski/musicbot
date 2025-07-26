use serenity::all::{
    Context,
    CommandInteraction,
    CreateInteractionResponseMessage,
    CreateInteractionResponse,
    EditInteractionResponse,
    CreateInteractionResponseFollowup,
};

pub async fn normal_response(ctx: &Context, command: &CommandInteraction, response: String) {
    let message = CreateInteractionResponseMessage::new().content(response);

    let builder = CreateInteractionResponse::Message(message);

    if let Err(why) = command.create_response(&ctx.http, builder).await {
        println!("Failed to create interaction response: {}", why);
    }
}

pub async fn edit_response(ctx: &Context, command: &CommandInteraction, response: String) {
    let builder = EditInteractionResponse::new().content(response);

    if let Err(why) = command.edit_response(&ctx.http, builder).await {
        println!("Failed to edit interaction response: {}", why);
    }
}

pub async fn followup_response(ctx: &Context, command: &CommandInteraction, response: String) {
    let message = CreateInteractionResponseFollowup::new().content(response);

    if let Err(why) = command.create_followup(&ctx.http, message).await {
        println!("Failed to create follow-up response: {}", why);
    }
}