//! X/Twitter OAuth 2.0 Authenticator

// std
use std::{env, io, sync::Arc};
// crates.io
use oauth2::{
	AuthUrl, AuthorizationCode, ClientId, ClientSecret, CsrfToken, EmptyExtraTokenFields,
	EndpointNotSet, EndpointSet, PkceCodeChallenge, RedirectUrl, RefreshToken,
	RevocationErrorResponseType, Scope, StandardErrorResponse, StandardRevocableToken,
	StandardTokenIntrospectionResponse, StandardTokenResponse, TokenResponse, TokenUrl,
	basic::{BasicClient, BasicErrorResponseType, BasicTokenType},
};
use reqwest::Client;
use tokio::sync::RwLock;
// self
use crate::prelude::*;

type OauthClient = oauth2::Client<
	StandardErrorResponse<BasicErrorResponseType>,
	StandardTokenResponse<EmptyExtraTokenFields, BasicTokenType>,
	StandardTokenIntrospectionResponse<EmptyExtraTokenFields, BasicTokenType>,
	StandardRevocableToken,
	StandardErrorResponse<RevocationErrorResponseType>,
	EndpointSet,
	EndpointNotSet,
	EndpointNotSet,
	EndpointNotSet,
	EndpointSet,
>;

/// OAuth 2.0 authenticator for X/Twitter API with token caching and refresh capabilities.
#[derive(Debug, Clone)]
pub struct Authenticator {
	/// The configured OAuth 2.0 client for X/Twitter authentication.
	oauth_client: OauthClient,
	/// Optional refresh token loaded from environment variables.
	refresh_token: Option<String>,
	/// Cached bearer token protected by async read-write lock.
	bearer_token: Arc<RwLock<Option<String>>>,
}
impl Authenticator {
	/// Creates a new authenticator with client credentials and X/Twitter OAuth endpoints.
	pub fn new(id: String, secret: String) -> Self {
		let oauth_client = BasicClient::new(ClientId::new(id))
			.set_client_secret(ClientSecret::new(secret))
			.set_auth_uri(
				AuthUrl::new("https://x.com/i/oauth2/authorize".into())
					.expect("url must be valid; qed"),
			)
			.set_token_uri(
				TokenUrl::new("https://api.x.com/2/oauth2/token".into())
					.expect("url must be valid; qed"),
			)
			.set_redirect_uri(
				RedirectUrl::new("http://localhost:8080/callback".into())
					.expect("url must be valid; qed"),
			);

		Self {
			oauth_client,
			refresh_token: env::var("X_REFRESH_TOKEN").ok(),
			bearer_token: Default::default(),
		}
	}

	/// Obtains a bearer token by attempting refresh first, then falling back to interactive flow.
	pub async fn request_bearer(&self, http: &Client) -> Result<String> {
		// Always try to refresh using refresh token first when program starts.
		if let Ok(bearer) = self.refresh_bearer_token(http).await {
			return Ok(bearer);
		}

		// No refresh token or refresh failed, start interactive flow.
		self.interactive_flow(http).await
	}

	/// Refreshes the bearer token using the stored refresh token.
	pub async fn refresh_bearer_token(&self, http: &Client) -> Result<String> {
		let refresh_token = self
			.oauth_client
			.exchange_refresh_token(&RefreshToken::new(
				self.refresh_token.clone().ok_or(Error::OauthRequired)?,
			))
			.request_async(http)
			.await?;
		let bearer_token = refresh_token.access_token().secret().to_owned();

		// Log the new refresh token if available, let user decide where to store it.
		if let Some(new_refresh_token) = refresh_token.refresh_token() {
			tracing::info!("🔄 new refresh token available: {}", new_refresh_token.secret());
			tracing::info!("💡 consider updating your X_REFRESH_TOKEN environment variable");
		}

		tracing::info!("✅ successfully refreshed bearer token");

		Ok(bearer_token)
	}

	/// Performs interactive OAuth flow requiring user to authorize in browser and enter code.
	pub async fn interactive_flow(&self, http: &Client) -> Result<String> {
		let (pkce_challenge, pkce_verifier) = PkceCodeChallenge::new_random_sha256();
		let (auth_url, _csrf) = self
			.oauth_client
			.authorize_url(CsrfToken::new_random)
			.add_scope(Scope::new("tweet.read".into()))
			.add_scope(Scope::new("tweet.write".into()))
			.add_scope(Scope::new("users.read".into()))
			.add_scope(Scope::new("offline.access".into()))
			.set_pkce_challenge(pkce_challenge)
			.url();

		tracing::info!("=== oauth 2.0 authorization ===");
		tracing::info!("open this url in your browser and paste the returned code: {auth_url}");

		let mut code = String::new();

		io::stdin().read_line(&mut code)?;

		let code = code.trim();

		if code.is_empty() {
			Err(Error::any("authorization code cannot be empty"))?;
		}

		let refresh_token = self
			.oauth_client
			.exchange_code(AuthorizationCode::new(code.to_owned()))
			.set_pkce_verifier(pkce_verifier)
			.request_async(http)
			.await?;
		let bearer_token = refresh_token.access_token().secret().to_owned();

		tracing::info!("✅ successfully obtained bearer token");

		if let Some(refresh_token) = refresh_token.refresh_token() {
			tracing::info!("🔑 refresh token: {}", refresh_token.secret());
			tracing::info!(
				"💡 save this refresh token to your X_REFRESH_TOKEN environment variable for future use"
			);
		}

		Ok(bearer_token)
	}

	/// Returns cached bearer token or triggers authentication flow if none exists.
	pub async fn authenticate(&self, http: &Client) -> Result<String> {
		// Check if we have a cached token first.
		if let Some(bearer) = &*self.bearer_token.read().await {
			return Ok(bearer.to_owned());
		}

		self.refresh_and_cache(http).await
	}

	/// Refreshes and caches a new bearer token with write lock protection.
	pub async fn refresh_and_cache(&self, http: &Client) -> Result<String> {
		// Acquire write lock to prevent multiple simultaneous token requests.
		let mut cached = self.bearer_token.write().await;
		let bearer = self.request_bearer(http).await?;

		*cached = Some(bearer.clone());

		Ok(bearer)
	}
}
