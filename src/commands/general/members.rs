use crate::{consts, Context};

use eyre::{OptionExt, Result};
use log::trace;
use poise::serenity_prelude::CreateEmbed;
use poise::CreateReply;

/// Returns the number of members in the server
#[poise::command(slash_command, guild_only = true)]
pub async fn members(ctx: Context<'_>) -> Result<()> {
	trace!("Running members command");
	let guild = ctx
		.http()
		.get_guild_with_counts(ctx.guild_id().unwrap())
		.await?;

	let embed = CreateEmbed::new()
		.title(format!(
			"{} total members!",
			guild
				.approximate_member_count
				.ok_or_eyre("Missing member count")?
		))
		.description(format!(
			"{} online members",
			guild
				.approximate_presence_count
				.ok_or_eyre("Missing online count")?
		))
		.color(consts::COLORS["blue"]);
	let reply = CreateReply::default().embed(embed);

	ctx.send(reply).await?;
	Ok(())
}
