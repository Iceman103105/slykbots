#[macro_use]
extern crate log;

use std::env::var;
use std::sync::OnceLock;
use std::time::Duration;
use async_openai::config::OpenAIConfig;

use serenity::framework::StandardFramework;
use serenity::framework::standard::buckets::LimitedFor;
use serenity::prelude::{Client, GatewayIntents};
use songbird::SerenityInit;
use sqlx::{Pool, Postgres};
use sqlx::postgres::PgPoolOptions;

use crate::ai::{AI_GROUP, dm_chatting};
use crate::handler::EventHandler;
use crate::voice::VOICE_GROUP;
use crate::yoking::{dispatch_error_hook, YOKES_GROUP};

mod markov_chains;
mod voice;
mod ai;
mod yoking;
pub mod settings;
pub mod handler;
pub mod commands;

pub const LEGACY_CMD: &str = ">";

static PG: OnceLock<Pool<Postgres>> = OnceLock::new();
static AI: OnceLock<async_openai::Client<OpenAIConfig>> = OnceLock::new();

#[tokio::main]
async fn main() {
    // load configs
    pretty_env_logger::init();
    let url = var("DATABASE_URL").expect("DATABASE_URL not found");
    let token = var("MURKOV_TOKEN").expect("MURKOV_TOKEN not found");
    let pool = PgPoolOptions::new()
        .max_lifetime(Duration::from_secs(10))
        .max_connections(25)
        .connect(url.as_str()).await.expect("Cannot create Database Pool");

    let framework = StandardFramework::new()
        .configure(|c| c
            .prefix(LEGACY_CMD))
        .group(&VOICE_GROUP)
        .group(&AI_GROUP)
        .group(&YOKES_GROUP)
        .bucket("ping", |b| b.limit(var("MAX_PING_PER_USER_PER10MIN").unwrap_or("1".to_string()).parse::<u32>().unwrap()).time_span(60).limit_for(LimitedFor::User)).await
        .bucket("openai", |b| b.limit(var("MAX_ASK_PER_USER_PER10MIN").unwrap_or("5".to_string()).parse::<u32>().unwrap()).time_span(60).limit_for(LimitedFor::User)).await
        .on_dispatch_error(dispatch_error_hook)
        .normal_message(dm_chatting);

    // create bot
    let mut bot = Client::builder(token.clone(),
                                  GatewayIntents::MESSAGE_CONTENT |
                                      GatewayIntents::GUILD_MESSAGES |
                                      GatewayIntents::GUILD_PRESENCES |
                                      GatewayIntents::DIRECT_MESSAGES |
                                      GatewayIntents::GUILDS | GatewayIntents::GUILD_VOICE_STATES)
        .event_handler(EventHandler::init())
        .framework(framework)
        .register_songbird()
        .await
        .expect("Error creating client");

    PG.set(pool).expect("Error setting DB Pool");
    AI.set(async_openai::Client::new()).expect("Error setting OpenAI Api Config");

    // start bot
    if let Err(why) = bot.start().await {
        error!("Client error: {:?}", why);
    }
}
