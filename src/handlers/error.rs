use crate::consts;
use crate::Data;

use eyre::Report;
use log::error;
use poise::serenity_prelude::{CreateEmbed, Timestamp};
use poise::{CreateReply, FrameworkError};

pub async fn handle(error: FrameworkError<'_, Data, Report>) {
	match error {
		FrameworkError::Setup {
			error, framework, ..
		} => {
			error!("Error setting up client! Bailing out");
			framework.shard_manager().shutdown_all().await;

			panic!("{error}")
		}

		FrameworkError::Command { error, ctx, .. } => {
			error!("Error in command {}:\n{error:?}", ctx.command().name);

			let embed = CreateEmbed::new()
				.title("Something went wrong!")
				.description("oopsie")
				.timestamp(Timestamp::now())
				.color(consts::COLORS["red"]);

			let reply = CreateReply::default().embed(embed);

			ctx.send(reply).await.ok();
		}

		FrameworkError::EventHandler {
			error,
			ctx: _,
			event,
			framework: _,
			..
		} => {
			error!(
				"Error while handling event {}:\n{error:?}",
				event.snake_case_name()
			);
		}

		FrameworkError::ArgumentParse {
			error, input, ctx, ..
		} => {
			let mut response = String::new();

			if let Some(input) = input {
				response += &format!("**Cannot parse `{input}` as argument: {error}**\n\n");
			} else {
				response += &format!("**{error}**\n\n");
			}

			if let Some(help_text) = ctx.command().help_text.as_ref() {
				response += &format!("{help_text}\n\n");
			}

			response += "**Tip:** Edit your message to update the response.\n";
			response += &format!(
				"For more information, refer to /help {}.",
				ctx.command().name
			);

			if let Err(e) = ctx.say(response).await {
				error!("Unhandled error displaying ArgumentParse error\n{e:#?}");
			}
		}

		error => {
			if let Err(e) = poise::builtins::on_error(error).await {
				error!("Unhandled error occurred:\n{e:#?}");
			}
		}
	}
}
