//! X/Twitter V2 Tweets API

// crates.io
use serde::{Deserialize, Serialize};
// self
use crate::{ApiResponse, prelude::*};

pub trait ApiTweet {
	fn tweet(&self, text: String) -> impl Send + Future<Output = Result<ApiResponse<TweetObject>>>;
}
impl ApiTweet for Api {
	async fn tweet(&self, text: String) -> Result<ApiResponse<TweetObject>> {
		self.post("https://api.x.com/2/tweets", &TweetRequest { text }).await
	}
}

#[derive(Debug, Serialize)]
pub struct TweetRequest {
	pub text: String,
}

#[derive(Debug, Deserialize)]
pub struct TweetObject {
	pub data: TweetData,
}

#[derive(Debug, Deserialize)]
pub struct TweetData {
	pub id: String,
	#[serde(default)]
	pub text: String,
}
