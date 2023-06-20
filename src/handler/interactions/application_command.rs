use serenity::client::Context;
use serenity::model::prelude::interaction::application_command::ApplicationCommandInteraction;
use sqlx::{Pool, Postgres};
use crate::commands::slash_command::SlashCommand;

pub async fn call(ctx: &Context, aci: &ApplicationCommandInteraction, cmd: &Box<dyn SlashCommand>, database: &Pool<Postgres> ) {
    if let Err(why) = cmd.execute(ctx, aci, &database)
        .await
    {
        println!("Cannot respond to slash command '/{}': {}", cmd.name(), why);
    }
}