//! Client
//!
//! API client, constructed through [`Builder`] via [`Client::builder()`].
//!
//! # Examples
//! #### Configure [`Client`] and ping API
//! An API key can be gotten from <https://docs.freestuffbot.xyz>.
//! ```no_run
//! # use freestuffapi::Client;
//! # #[tokio::main]
//! # async fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let api_key = "secret api key";
//! let client = Client::builder()
//!     .key(&api_key)
//!     .build()?;
//! client.ping().await?;
//! #     Ok(())
//! # }
//! ```

// @TODO: escape input! like category and game ids

use crate::api::*;
use reqwest::{header, Client as RClient, Method, Response, StatusCode, Url};
use serde::Deserialize;
use std::collections::HashMap;
use thiserror::Error;

type APIError = String;
type GameId = u64;

/// Builder errors
#[derive(Error, Debug)]
pub enum BuilderError {
    /// Failed to convert a URL
    #[error("Failed to convert into a valid URL")]
    URLConversion,

    /// No API key was specified
    #[error("No API key was set")]
    NoAPIKey,
}

/// Client builder
pub struct Builder {
    api_domain: Url,
    api_key: Option<String>,
}

impl Builder {
    /// Construct a new Builder.
    ///
    /// # Example
    /// ```no_run
    /// # use freestuffapi::Client;
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let api_key = "blah blah";
    ///     let client = Client::builder()
    ///         .key(&api_key)
    ///         .build()?;
    /// #     Ok(())
    /// # }
    /// ```
    pub fn new() -> Self {
        Self {
            api_domain: "https://api.freestuffbot.xyz"
                .parse()
                .expect("Failed to parse default API base URL"),
            api_key: None,
        }
    }

    /// Set API domain. Defaults to <api.freestuffbot.xyz>
    pub fn api_domain<U: TryInto<Url>>(mut self, domain: U) -> Result<Builder, BuilderError> {
        self.api_domain = domain.try_into().map_err(|_| BuilderError::URLConversion)?;
        Ok(self)
    }

    /// Set API key. Required to run, as there is no public API.
    ///
    /// An API key can be gotten by donating, see: <https://docs.freestuffbot.xyz>
    pub fn key(mut self, key: &str) -> Self {
        self.api_key = Some(key.to_string());
        self
    }

    /// Consume Builder and construct Client
    pub fn build(self) -> Result<Client, BuilderError> {
        let api_key = self.api_key.ok_or(BuilderError::NoAPIKey)?;

        let http_client = RClient::builder()
            .https_only(true)
            .build()
            .expect("failed to build http client");

        Ok(Client {
            api_domain: self.api_domain,
            api_key,
            http_client,
        })
    }
}

type ClientResult<T> = Result<T, ClientError>;

/// Client errors
#[derive(Error, Debug)]
pub enum ClientError {
    /// HTTP error
    #[error("HTTP error")]
    HTTP(reqwest::Error),

    /// Invalid response from API
    #[error("Invalid response from API")]
    InvalidResponse,

    /// API error
    #[error("API error: {0}")]
    API(APIError),

    /// Ratelimited by API
    #[error("Too many requests")]
    Ratelimited,
}

/// Client
pub struct Client {
    api_domain: Url,
    api_key: String,
    http_client: RClient,
}

impl Client {
    /// Construct a new Client Builder
    ///
    /// Alias to Builder::new
    pub fn builder() -> Builder {
        Builder::new()
    }

    /// Build API endpoint URL
    fn api_endpoint(&self, endpoint: &str) -> Url {
        self.api_domain
            .join(endpoint)
            .expect("Failed to construct API endpoint URL")
    }

    /// Send authorized requests to API
    async fn send_request(
        &self,
        endpoint: &str,
        _parameters: Option<()>,
    ) -> ClientResult<Response> {
        let url = self.api_endpoint(endpoint);
        let request = self
            .http_client
            .request(Method::GET, url)
            .header(header::AUTHORIZATION, format!("Basic {}", self.api_key))
            .build()
            .map_err(ClientError::HTTP)?;

        self.http_client
            .execute(request)
            .await
            .map_err(ClientError::HTTP)
            .and_then(|response| match response.status() {
                status if status.is_success() => Ok(response),
                StatusCode::TOO_MANY_REQUESTS => Err(ClientError::Ratelimited),
                _ => Err(ClientError::InvalidResponse),
            })
    }

    /// Pings via API and returns if success
    pub async fn ping(&self) -> ClientResult<bool> {
        Ok(self
            .send_request("/v1/ping", None)
            .await?
            .status()
            .is_success())
    }

    /// Fetch list of game IDs in category.
    ///
    /// Valid categories are `all`, `approved`, and `free`.
    pub async fn game_list(&self, category: &str) -> ClientResult<Vec<GameId>> {
        let path = format!("/v1/games/{category}");
        self.send_request(&path, None)
            .await?
            .json::<ApiResponse<Vec<GameId>>>()
            .await
            .map_err(ClientError::HTTP)?
            .into_data()
            .map_err(ClientError::API)
    }

    /// Fetch info about games
    ///
    /// Limited to five games per request.
    ///
    /// # Example
    /// ```no_run
    /// # use freestuffapi::Client;
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = Client::builder().key("secret api key").build()?;
    /// for game in client.game_details(&[1234, 5678, 1020, 2030]).await?.into_values() {
    ///     println!("{}: {}", game.title, game.description.unwrap_or_default());
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn game_details(&self, games: &[GameId]) -> ClientResult<HashMap<String, GameInfo>> {
        if games.len() == 0 {
            return Ok(HashMap::new());
        }

        let ids = games
            .iter()
            .map(|id| id.to_string())
            .reduce(|acc, id| format!("{acc}+{id}"))
            .expect("at least one id must be specified");

        let path = format!("/v1/game/{ids}/info");
        self.send_request(&path, None)
            .await?
            .json::<ApiResponse<HashMap<String, GameInfo>>>()
            .await
            .map_err(ClientError::HTTP)?
            .into_data()
            .map_err(ClientError::API)
    }

    /// Fetch info about a single game.
    ///
    /// Helper function for [`game_details`]
    ///
    /// [`game_details`]: Self::game_details
    pub async fn game_detail(&self, game: GameId) -> ClientResult<GameInfo> {
        self.game_details(&[game])
            .await
            .and_then(|map| map.into_values().next().ok_or(ClientError::InvalidResponse))
    }
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct ApiResponse<Data> {
    success: bool,
    error: Option<String>,
    message: Option<String>,
    data: Data,
}

impl<Data> ApiResponse<Data> {
    /// Consumes self and returns response data
    pub fn into_data(self) -> Result<Data, APIError> {
        self.message.map(Err).unwrap_or(Ok(self.data))
    }
}
