use std::str::FromStr;

use eyre::{eyre, Context as _, Result};
use log::{debug, trace};
use once_cell::sync::Lazy;
use poise::serenity_prelude::{
	Channel, ChannelId, ChannelType, Colour, Context, CreateEmbed, CreateEmbedAuthor,
	CreateEmbedFooter, Message, MessageId, Permissions,
};
use regex::Regex;

static MESSAGE_PATTERN: Lazy<Regex> = Lazy::new(|| {
	Regex::new(r"(?:https?:\/\/)?(?:canary\.|ptb\.)?discord(?:app)?\.com\/channels\/(?<server_id>\d+)\/(?<channel_id>\d+)\/(?<message_id>\d+)").unwrap()
});

fn find_first_image(msg: &Message) -> Option<String> {
	msg.attachments
		.iter()
		.find(|a| {
			a.content_type
				.as_ref()
				.unwrap_or(&String::new())
				.starts_with("image/")
		})
		.map(|res| res.url.clone())
}

pub async fn resolve(ctx: &Context, msg: &Message) -> Result<Vec<CreateEmbed>> {
	let Some(source_server_id) = msg.guild_id else {
		debug!("Not resolving any messages in DM");
		return Ok(Vec::new());
	};

	let source_server = source_server_id
		.to_guild_cached(ctx)
		.ok_or_else(|| eyre!("Guild {} not cached", source_server_id))?
		.to_owned();

	let member = source_server.member(&ctx.http, &msg.author).await?;

	let matches = MESSAGE_PATTERN
		.captures_iter(&msg.content)
		.map(|capture| capture.extract());

	let mut embeds: Vec<CreateEmbed> = vec![];

	for (url, [server_id, channel_id, message_id]) in matches {
		if server_id != source_server_id.to_string() {
			debug!("Not resolving message from other server");
			continue;
		}

		trace!("Attempting to resolve message {message_id} from URL {url}");

		let Ok(Channel::Guild(channel)) = ChannelId::from_str(channel_id)
			.wrap_err_with(|| format!("Couldn't parse channel ID {channel_id}!"))?
			.to_channel(ctx)
			.await
		else {
			debug!(
				"Not resolving message in {} as it does not appear to exist",
				channel_id
			);
			continue;
		};

		let author_can_view = match channel.kind {
			ChannelType::Text | ChannelType::News => source_server
				.user_permissions_in(&channel, member.as_ref())
				.contains(Permissions::VIEW_CHANNEL),

			ChannelType::PublicThread => {
				let parent = channel
					.parent_id
					.ok_or_else(|| eyre!("Thread {} has no parent", channel.id))?
					.to_channel_cached(&ctx.cache)
					.ok_or_else(|| eyre!("Parent channel of {} not cached", channel.id))?;

				source_server
					.user_permissions_in(&parent, member.as_ref())
					.contains(Permissions::VIEW_CHANNEL)
			}

			_ => false,
		};

		if !author_can_view {
			debug!("Not resolving message for author who can't see it");
			continue;
		}

		let Ok(original_message) = channel
			.message(
				ctx,
				MessageId::from_str(message_id)
					.wrap_err_with(|| format!("Couldn't parse message ID {message_id}!"))?,
			)
			.await
		else {
			debug!("No message found in {channel_id} by {message_id}");
			continue;
		};

		let author = CreateEmbedAuthor::new(original_message.author.tag()).icon_url(
			original_message
				.author
				.avatar_url()
				.unwrap_or_else(|| original_message.author.default_avatar_url()),
		);
		let footer = CreateEmbedFooter::new(format!("#{}", channel.name));

		let mut embed = CreateEmbed::new()
			.author(author)
			.color(Colour::BLITZ_BLUE)
			.timestamp(original_message.timestamp)
			.footer(footer)
			.description(format!(
				"{}\n\n[Jump to original message]({})",
				original_message.content,
				original_message.link()
			));

		if !original_message.attachments.is_empty() {
			embed = embed.fields(original_message.attachments.iter().map(|a| {
				(
					"Attachments".to_string(),
					format!("[{}]({})", a.filename, a.url),
					false,
				)
			}));

			if let Some(image) = find_first_image(msg) {
				embed = embed.image(image);
			}
		}

		embeds.push(embed);
	}

	Ok(embeds)
}
