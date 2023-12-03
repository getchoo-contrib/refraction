use std::{sync::Arc, time::Duration};

use color_eyre::eyre::{eyre, Context as _, Report, Result};
use config::Config;
use log::*;
use poise::{
    serenity_prelude as serenity, EditTracker, Framework, FrameworkOptions, PrefixFrameworkOptions,
};

mod api;
mod commands;
mod config;
mod consts;
mod handlers;
mod utils;

type Context<'a> = poise::Context<'a, Data, Report>;

#[derive(Clone)]
pub struct Data {
    config: config::Config,
    redis: redis::Client,
    octocrab: Arc<octocrab::Octocrab>,
}

impl Data {
    pub fn new() -> Result<Self> {
        let config = Config::new_from_env()?;
        let redis = redis::Client::open(config.redis_url.clone())?;
        let octocrab = octocrab::instance();

        Ok(Self {
            config,
            redis,
            octocrab,
        })
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    dotenvy::dotenv().ok();
    color_eyre::install()?;
    env_logger::init();

    let token =
        std::env::var("TOKEN").wrap_err_with(|| eyre!("Couldn't find token in environment!"))?;

    let intents =
        serenity::GatewayIntents::non_privileged() | serenity::GatewayIntents::MESSAGE_CONTENT;

    let options = FrameworkOptions {
        commands: commands::to_global_commands(),
        on_error: |error| Box::pin(handlers::handle_error(error)),
        command_check: Some(|ctx| {
            Box::pin(async move { Ok(ctx.author().id != ctx.framework().bot_id) })
        }),
        event_handler: |ctx, event, framework, data| {
            Box::pin(handlers::handle_event(ctx, event, framework, data))
        },
        prefix_options: PrefixFrameworkOptions {
            prefix: Some("!".into()),
            edit_tracker: Some(EditTracker::for_timespan(Duration::from_secs(3600))),
            ..Default::default()
        },
        ..Default::default()
    };

    let framework = Framework::builder()
        .token(token)
        .intents(intents)
        .options(options)
        .setup(|ctx, _ready, framework| {
            Box::pin(async move {
                poise::builtins::register_globally(ctx, &framework.options().commands).await?;
                info!("Registered global commands!");

                let data = Data::new()?;

                Ok(data)
            })
        });

    tokio::select! {
        result = framework.run() => { result.map_err(Report::from) },
        _ = tokio::signal::ctrl_c() => {
            info!("Interrupted! Exiting...");
            std::process::exit(130);
        }
    }
}
