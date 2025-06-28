#![allow(missing_docs)]

// std
use std::borrow::Cow;

pub type Result<T, E = Error> = std::result::Result<T, E>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
	#[error("{0}")]
	Any(Cow<'static, str>),

	#[error(transparent)]
	Io(#[from] std::io::Error),

	#[error(transparent)]
	Oauth2(
		#[from]
		oauth2::RequestTokenError<
			oauth2::HttpClientError<reqwest::Error>,
			oauth2::StandardErrorResponse<oauth2::basic::BasicErrorResponseType>,
		>,
	),
	#[error(transparent)]
	Reqwest(#[from] reqwest::Error),
	#[error(transparent)]
	SerdeJson(#[from] serde_json::Error),

	#[error(transparent)]
	Api(#[from] crate::ApiError),
	#[error("authentication failed")]
	AuthenticationFailed,
	#[error("oauth required")]
	OauthRequired,
	#[error("rate limit exceeded")]
	RateLimit,
	#[error("unauthorized")]
	Unauthorized,
}
impl Error {
	pub fn any<T>(any: T) -> Self
	where
		T: Into<Cow<'static, str>>,
	{
		Self::Any(any.into())
	}
}
