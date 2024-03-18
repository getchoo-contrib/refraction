use crate::api;
use std::time::Duration;

use eyre::Result;
use log::{debug, trace};
use poise::serenity_prelude::{Message, UserId};
use tokio::time::sleep;

const PK_DELAY: Duration = Duration::from_secs(1);

pub async fn is_message_proxied(msg: &Message) -> Result<bool> {
	if msg.webhook_id.is_some() {
		return Ok(false);
	}

	trace!(
		"Waiting on PluralKit API for {} seconds",
		PK_DELAY.as_secs()
	);
	sleep(PK_DELAY).await;

	let proxied = api::pluralkit::get_sender(msg.id).await.is_ok();

	Ok(proxied)
}

pub async fn get_original_author(msg: &Message) -> Result<Option<UserId>> {
	if msg.webhook_id.is_none() {
		return Ok(None);
	}

	debug!(
		"Message {} has a webhook ID. Checking if it was sent through PluralKit",
		msg.id
	);

	trace!(
		"Waiting on PluralKit API for {} seconds",
		PK_DELAY.as_secs()
	);
	sleep(PK_DELAY).await;

	Ok(Some(api::pluralkit::get_sender(msg.id).await?))
}
