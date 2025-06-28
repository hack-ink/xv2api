//! Twitter/X v2 API Client Library

// #![deny(clippy::all, missing_docs, unused_crate_dependencies)]

pub mod auth;
pub mod error;
pub mod tweets;

mod prelude {
	pub use serde::{Deserialize, Serialize};
	pub use std::future::Future;

	pub(crate) use crate::{Api, error::*};
}
use prelude::*;

// std
use std::{
	env,
	error::Error as ErrorT,
	fmt::{Display, Formatter, Result as FmtResult},
};
// crates.io
use reqwest::{
	Client, RequestBuilder, Response,
	header::{AUTHORIZATION, CONTENT_TYPE},
};
// self
use auth::Authenticator;

/// Main API client for interacting with X/Twitter v2 API endpoints.
#[derive(Clone, Debug)]
pub struct Api {
	/// OAuth 2.0 authenticator for managing bearer tokens.
	pub authenticator: Authenticator,
	http: Client,
}
impl Api {
	/// Creates API client using credentials from environment variables.
	pub fn from_env() -> Self {
		let id = env::var("X_CLIENT_ID").expect("X_CLIENT_ID not set");
		let secret = env::var("X_CLIENT_SECRET").expect("X_CLIENT_SECRET not set");
		let authenticator = Authenticator::new(id, secret);

		Self { authenticator, http: Client::new() }
	}

	/// Creates API client with provided OAuth 2.0 credentials.
	pub fn new(id: String, secret: String) -> Self {
		let authenticator = Authenticator::new(id, secret);

		Self { authenticator, http: Client::new() }
	}

	/// Executes HTTP requests with automatic token refresh on authentication failure.
	async fn execute_request<T>(
		&self,
		request_builder: impl Fn(&str) -> RequestBuilder,
	) -> Result<T>
	where
		T: for<'de> Deserialize<'de>,
	{
		// First attempt with cached token.
		let mut token = self.authenticator.authenticate(&self.http).await?;

		for attempt in 0..2 {
			let resp = request_builder(&token).send().await?;
			let status = resp.status();

			// If 401 and this is the first attempt, refresh token and retry.
			if status == 401 && attempt == 0 {
				// Force refresh and update cache since current token is invalid
				token = self.authenticator.refresh_and_cache(&self.http).await?;

				continue;
			}

			let txt = self.handle_response(resp).await?;

			return Ok(serde_json::from_str::<T>(&txt)?);
		}

		unreachable!("loop must always return within 2 attempts; qed")
	}

	/// Handles HTTP response status codes and extracts response body text.
	async fn handle_response(&self, response: Response) -> Result<String> {
		let status = response.status();
		let txt = response.text().await?;

		if status == 401 {
			Err(Error::Unauthorized)?;
		} else if status == 429 {
			Err(Error::RateLimit)?;
		} else if !status.is_success() {
			if let Ok(e) = serde_json::from_str::<ApiError>(&txt) {
				Err(e)?;
			}

			Err(Error::any(format!("{status}: {txt}")))?;
		}

		Ok(txt)
	}

	// async fn get<T>(&self, url: &str) -> Result<T>
	// where
	// 	T: for<'de> Deserialize<'de>,
	// {
	// 	self.execute_request(|token| {
	// 		self.http
	// 			.get(url)
	// 			.header(AUTHORIZATION, format!("Bearer {token}"))
	// 			.header(CONTENT_TYPE, "application/json")
	// 	})
	// 	.await
	// }

	/// Sends POST requests with JSON body to API endpoints.
	async fn post<B, T>(&self, url: &str, body: &B) -> Result<T>
	where
		B: Serialize,
		T: for<'de> Deserialize<'de>,
	{
		self.execute_request(|bearer| {
			self.http
				.post(url)
				.header(AUTHORIZATION, format!("Bearer {bearer}"))
				.header(CONTENT_TYPE, "application/json")
				.json(body)
		})
		.await
	}

	// async fn put<B, T>(&self, url: &str, body: &B) -> Result<T>
	// where
	// 	B: Serialize,
	// 	T: for<'de> Deserialize<'de>,
	// {
	// 	self.execute_request(|token| {
	// 		self.http
	// 			.put(url)
	// 			.header(AUTHORIZATION, format!("Bearer {token}"))
	// 			.header(CONTENT_TYPE, "application/json")
	// 			.json(body)
	// 	})
	// 	.await
	// }

	// async fn delete<T>(&self, url: &str) -> Result<T>
	// where
	// 	T: for<'de> Deserialize<'de>,
	// {
	// 	self.execute_request(|token| {
	// 		self.http
	// 			.delete(url)
	// 			.header(AUTHORIZATION, format!("Bearer {token}"))
	// 			.header(CONTENT_TYPE, "application/json")
	// 	})
	// 	.await
	// }
}

/// Response wrapper that can contain either successful data or API error information.
#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum ApiResponse<T> {
	/// Successful response containing the requested data.
	Ok(T),
	/// Error response containing API error details.
	Err(ApiError),
}

#[derive(Debug, Deserialize)]
/// API error response structure containing error details from X/Twitter API.
pub struct ApiError {
	/// Detailed description of the error that occurred.
	pub detail: String,
	/// HTTP status code associated with the error.
	pub status: u32,
	/// Brief title or category of the error.
	pub title: String,
	/// URI reference identifying the error type.
	pub r#type: String,
}
impl Display for ApiError {
	fn fmt(&self, f: &mut Formatter) -> FmtResult {
		write!(f, "{} ({}): {}", self.title, self.status, self.detail)
	}
}
impl ErrorT for ApiError {
	fn source(&self) -> Option<&(dyn 'static + ErrorT)> {
		None
	}
}
