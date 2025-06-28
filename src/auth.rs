//! X/Twitter OAuth 2.0 Authenticator

// std
use std::{env, fs, io, sync::Arc};
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

#[derive(Debug, Clone)]
pub struct Authenticator {
	oauth_client: OauthClient,
	bearer_token: Arc<RwLock<Option<String>>>,
	refresh_token: Option<String>,
}
impl Authenticator {
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
			bearer_token: Arc::new(RwLock::new(env::var("X_BEARER_TOKEN").ok())),
			refresh_token: env::var("X_REFRESH_TOKEN").ok(),
		}
	}

	pub async fn request_bearer(&self, http: &Client) -> Result<String> {
		// First, try to use the existing bearer token if available.
		if let Some(bearer) = &*self.bearer_token.read().await {
			return Ok(bearer.to_owned());
		}
		// If no bearer token, try to refresh using refresh token.
		if let Ok(bearer) = self.refresh_bearer_token(http).await {
			return Ok(bearer);
		}

		// No saved tokens or refresh failed, start interactive flow.
		self.interactive_flow(http).await
	}

	pub async fn refresh_bearer_token(&self, http: &Client) -> Result<String> {
		let refresh_token = self
			.oauth_client
			.exchange_refresh_token(&RefreshToken::new(
				self.refresh_token.clone().ok_or(Error::OauthRequired)?,
			))
			.request_async(http)
			.await?;
		let bearer_token = refresh_token.access_token().secret().to_owned();

		self.update_tokens(
			&bearer_token,
			refresh_token.refresh_token().map(|r| r.secret().as_str()),
		)?;

		Ok(bearer_token)
	}

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

		println!("\n=== Zoauth 2.0 authorization ===");
		println!("open this url in your browser and paste the returned code:\n{auth_url}\n");

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

		self.update_tokens(
			&bearer_token,
			refresh_token.refresh_token().map(|r| r.secret().as_str()),
		)?;

		println!("✅ obtained bearer token {bearer_token}");

		if let Some(refresh_token) = refresh_token.refresh_token() {
			println!("x_refresh_token={}", refresh_token.secret());
		}

		Ok(bearer_token)
	}

	fn update_tokens(&self, bearer_token: &str, refresh_token: Option<&str>) -> Result<()> {
		let path = ".env";
		let mut lines: Vec<String> =
			fs::read_to_string(path).unwrap_or_default().lines().map(|l| l.to_string()).collect();
		let mut ensure = |key: &str, val: &str| {
			if let Some(idx) = lines.iter().position(|l| l.starts_with(&format!("export {key}="))) {
				lines[idx] = format!("export {key}={val}");
			} else {
				lines.push(format!("export {key}={val}"));
			}
		};

		ensure("X_BEARER_TOKEN", bearer_token);

		if let Some(r) = refresh_token {
			ensure("X_REFRESH_TOKEN", r);
		}

		fs::write(path, lines.join("\n") + "\n")?;

		Ok(())
	}

	pub async fn authenticate(&self, http: &Client) -> Result<String> {
		// Check if we have a cached token first.
		if let Some(bearer) = &*self.bearer_token.read().await {
			return Ok(bearer.to_owned());
		}

		// No cached token, get a new one.
		let bearer = self.request_bearer(http).await?;

		// Cache the token.
		{
			let mut cached = self.bearer_token.write().await;

			*cached = Some(bearer.clone());
		}

		Ok(bearer)
	}

	pub async fn clear_cached_token(&self) {
		self.bearer_token.write().await.take();
	}
}
