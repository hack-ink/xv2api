<div align="center">

# XV2API

### X/Twitter V2 API Library

[![License](https://img.shields.io/badge/License-GPLv3-blue.svg)](https://www.gnu.org/licenses/gpl-3.0)
[![Docs](https://img.shields.io/docsrs/xv2api)](https://docs.rs/xv2api)
[![Checks](https://github.com/hack-ink/xv2api/actions/workflows/rust.yml/badge.svg?branch=main)](https://github.com/hack-ink/xv2api/actions/workflows/rust.yml)
[![Release](https://github.com/hack-ink/xv2api/actions/workflows/release.yml/badge.svg)](https://github.com/hack-ink/xv2api/actions/workflows/release.yml)
[![GitHub tag (latest by date)](https://img.shields.io/github/v/tag/hack-ink/xv2api)](https://github.com/hack-ink/xv2api/tags)
[![GitHub last commit](https://img.shields.io/github/last-commit/hack-ink/xv2api?color=red&style=plastic)](https://github.com/hack-ink/xv2api)
[![GitHub code lines](https://tokei.rs/b1/github/hack-ink/xv2api)](https://github.com/hack-ink/xv2api)
</div>

## Feature Highlights

### 🚀 Core Features

- **OAuth 2.0 Authentication**: Secure authentication with automatic token refresh and caching
- **Tweet Posting**: Create and publish tweets via X/Twitter V2 API
- **Rate Limiting**: Built-in rate limit handling and error management
- **Async/Await Support**: Fully asynchronous API built with Tokio
- **Environment Configuration**: Easy setup using environment variables
- **Token Management**: Automatic bearer token refresh with optional refresh token persistence

### 🛡️ Security & Reliability

- **Automatic Token Refresh**: Seamlessly handles token expiration without manual intervention
- **Error Handling**: Comprehensive error types for different API scenarios
- **PKCE Flow Support**: Enhanced security with Proof Key for Code Exchange
- **TLS Security**: All requests use secure HTTPS connections with rustls

### 🎯 Developer Experience

- **Simple API**: Clean, intuitive interface for common X/Twitter operations
- **Type Safety**: Full Rust type safety with serde serialization/deserialization
- **Comprehensive Documentation**: Well-documented code with examples
- **Environment-based Configuration**: Secure credential management

## Usage

### Configuration

#### Environment Variables

Set up the required environment variables for X/Twitter API access:

```sh
# Required: Your X/Twitter App Client ID and Secret
export X_CLIENT_ID="your_client_id_here"
export X_CLIENT_SECRET="your_client_secret_here"

# Optional: Refresh token to avoid re-authentication
export X_REFRESH_TOKEN="your_refresh_token_here"
```

#### Basic Example

```rust
use xv2api::{Api, tweets::ApiTweet};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize API client from environment variables.
    let api = Api::from_env();

    // Post a tweet.
    let resp = api.tweet("Hello from XV2API! 🦀".to_string()).await?;

	println!("Response: {resp:?}");
    println!("Tweet posted successfully!");

    Ok(())
}
```

## Development

### Architecture

XV2API follows a modular architecture designed for extensibility and maintainability:

```text
xv2api/
├── src/
│   ├── lib.rs          # Main API client and core functionality
│   ├── auth.rs         # OAuth 2.0 authentication module
│   ├── tweets.rs       # Tweet-related API endpoints
│   └── error.rs        # Error types and handling
├── Cargo.toml          # Project configuration and dependencies
└── README.md           # This documentation
```

#### Core Components

- **`Api`**: Main client struct handling HTTP requests and authentication
- **`Authenticator`**: OAuth 2.0 flow management with token caching
- **`ApiTweet`**: Trait defining tweet-related operations
- **`Error`**: Comprehensive error handling for various failure scenarios

#### Key Design Principles

1. **Async-First**: Built with Tokio for high-performance async operations
2. **Type Safety**: Leverages Rust's type system for compile-time guarantees
3. **Error Transparency**: Clear error types for different failure modes
4. **Token Management**: Automatic token refresh with minimal user intervention
5. **Modular Design**: Easy to extend with new API endpoints

## Support Me

If you find this project helpful and would like to support its development, you can buy me a coffee!

Your support is greatly appreciated and motivates me to keep improving this project.

- **Fiat**
  - [Ko-fi](https://ko-fi.com/hack_ink)
  - [爱发电](https://afdian.com/a/hack_ink)
- **Crypto**
  - **Bitcoin**
    - `bc1pedlrf67ss52md29qqkzr2avma6ghyrt4jx9ecp9457qsl75x247sqcp43c`
  - **Ethereum**
    - `0x3e25247CfF03F99a7D83b28F207112234feE73a6`
  - **Polkadot**
    - `156HGo9setPcU2qhFMVWLkcmtCEGySLwNqa3DaEiYSWtte4Y`

Thank you for your support!

## Appreciation

We would like to extend our heartfelt gratitude to the following projects and contributors:

- The Rust community for their continuous support and development of the Rust ecosystem.

## Additional Acknowledgements

- [OAuth2 crate](https://crates.io/crates/oauth2) for providing excellent OAuth 2.0 implementation
- X/Twitter Developer Community for API documentation and support

<div align="right">

### License

<sup>Licensed under [GPL-3.0](LICENSE).</sup>
</div>
