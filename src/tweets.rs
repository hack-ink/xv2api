//! X/Twitter V2 Tweets API

// crates.io
use serde::{Deserialize, Serialize};
// self
use crate::{ApiResponse, prelude::*};

/// Trait for posting tweets to X/Twitter API.
pub trait ApiTweet {
	/// Posts a tweet with the given text content.
	fn tweet(&self, text: String) -> impl Send + Future<Output = Result<ApiResponse<TweetObject>>>;
}
/// Implementation of tweet posting functionality for the main API client.
impl ApiTweet for Api {
	async fn tweet(&self, text: String) -> Result<ApiResponse<TweetObject>> {
		self.post("https://api.x.com/2/tweets", &TweetRequest { text }).await
	}
}

/// Request payload for creating a new tweet.
#[derive(Debug, Serialize)]
pub struct TweetRequest {
	/// The text content of the tweet to be posted.
	pub text: String,
}

/// Response object containing tweet data from the API.
#[derive(Debug, Deserialize)]
pub struct TweetObject {
	/// The actual tweet data returned by the API.
	pub data: TweetData,
}

/// Core tweet data structure containing tweet information.
#[derive(Debug, Deserialize)]
pub struct TweetData {
	/// Unique identifier for the tweet.
	pub id: String,
	/// The text content of the tweet.
	pub text: String,
}
