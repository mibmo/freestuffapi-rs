//! Raw API types.

use serde::{Deserialize, Deserializer};
use std::collections::HashMap;

/// Service status
#[derive(Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ServiceStatus {
    /// Everything is alright
    Ok,
    /// Service is experiencing partial disconnects / issues
    Partial,
    /// Sent on server startup
    Rebooting,
    /// Sent on error that requires human interaction
    Fatal,
}

/// Game info
#[derive(Debug, Deserialize)]
pub struct GameInfo {
    /// Game URLs
    pub urls: Urls,
    /// Proxy URL. Deprecated: use urls.default
    pub url: String,
    /// Direct URL. Deprecated: use urls.org
    pub org_url: String,
    /// Game title
    pub title: String,
    /// Prices before discount
    #[serde(deserialize_with = "object_empty_as_none")]
    pub org_price: Option<Price>,
    /// Prices with discount applied
    #[serde(deserialize_with = "object_empty_as_none")]
    pub price: Option<Price>,
    /// Thumbnail URLs
    #[serde(deserialize_with = "object_empty_as_none")]
    pub thumbnail: Option<Thumbnail>,
    /// Product kind
    pub kind: ProductKind,
    /// Tags
    pub tags: Vec<String>,
    /// Description
    pub description: Option<String>,
    /// Rating
    pub rating: Option<f32>,
    /// Notice from Freestuff API
    pub notice: Option<String>,
    /// Lasts until
    pub until: Option<f32>,
    /// Store
    pub store: Store,
    /// Flags that describe product
    pub flags: GameFlags,
    #[serde(rename = "type")]
    /// Announcement type
    pub game_type: AnnouncementType,
    //store_meta:
    /// Localized versions of product info
    pub localized: Option<HashMap<String, LocalizedGameInfo>>,
}

/// Game URLs
#[derive(Debug, Deserialize)]
pub struct Urls {
    /// Recommended URL
    pub default: String,
    /// Opens in browser
    pub browser: String,
    /// Opens in related desktop client (i.e. steam://)
    pub client: Option<String>,
    /// Original URL
    pub org: String,
}

/// Game prices
#[derive(Debug, Deserialize)]
pub struct Price {
    /// Euro price
    pub euro: Option<f64>,
    /// USD price
    pub dollar: Option<f64>,
}

/// Thumbnail URLs
#[derive(Debug, Deserialize)]
pub struct Thumbnail {
    /// Original thumbnail image
    pub org: String,
    /// Proxied and properly cropped thumbnail image
    pub blank: String,
    /// Proxied image with all available extra info
    pub full: String,
    /// Proxied image with game tags above thumbnail
    pub tags: String,
}

/// Localized game info
#[allow(missing_docs)]
#[derive(Debug, Deserialize)]
pub struct LocalizedGameInfo {
    /// Localized language name
    pub lang_name: String,
    /// Language name in English
    pub lang_name_en: String,
    /// Language flag emoji
    pub lang_flag_emoji: String,
    pub platform: String,
    pub claim_long: String,
    pub claim_short: String,
    pub free: String,
    pub header: String,
    pub footer: String,
    pub org_price_eur: String,
    pub org_price_usd: String,
    pub until: String,
    pub until_alt: String,
    pub flags: Vec<String>,
}

/// Game store
#[derive(Debug, Deserialize)]
#[serde(field_identifier, rename_all = "lowercase")]
pub enum Store {
    /// Steam
    Steam,
    /// Epic Games store
    Epic,
    /// Humble Bundle
    Humble,
    /// Good Old Games
    Gog,
    /// EA Origin
    Origin,
    /// Ubisoft UPlay
    Uplay,
    /// Twitch store
    Twitch,
    /// itch.io
    Itch,
    /// Discord Store
    Discord,
    /// Apple App Store
    Apple,
    /// Google Play Store
    Google,
    /// Nintendo Switch
    Switch,
    /// Playstation
    Ps,
    /// Microsoft Xbox
    Xbox,
    /// Other store
    Other(String),
}

/// Type of announcement
#[derive(Debug, Deserialize)]
#[serde(field_identifier, rename_all = "lowercase")]
pub enum AnnouncementType {
    /// Free to keep
    Free,
    /// Playable during weekend
    Weekend,
    /// Discount on game
    Discount,
    /// Advertisement
    Ad,
    /// Unknown announcement type
    Unknown(String),
}

/// Game flags
#[derive(Debug, Deserialize)]
pub struct GameFlags(u8);

impl GameFlags {
    /// Get raw bitflag number
    pub fn inner(&self) -> u8 {
        self.0
    }

    fn bit(&self, bit: usize) -> bool {
        self.0 >> bit & 1 == 1
    }

    /// Low quality game
    pub fn trash(&self) -> bool {
        self.bit(0)
    }

    /// Third party key provider
    pub fn thirdparty(&self) -> bool {
        self.bit(1)
    }
}

/// Type of product
#[derive(Debug, Deserialize)]
#[serde(field_identifier, rename_all = "lowercase")]
pub enum ProductKind {
    /// Game
    Game,
    /// Downloadable Content
    DLC,
    /// Software
    Software,
    /// Art
    Art,
    /// Original soundtrack
    OST,
    /// Book
    Book,
    /// Other products
    Other(String),
}

/// Deserialize empty objects as [`Option::None`]
///
/// Used with serde fields' deserialize_with
fn object_empty_as_none<'de, D, T>(deserializer: D) -> Result<Option<T>, D::Error>
where
    D: Deserializer<'de>,
    for<'a> T: Deserialize<'a>,
{
    #[derive(Deserialize, Debug)]
    #[serde(deny_unknown_fields)]
    struct Empty {}

    #[derive(Deserialize, Debug)]
    #[serde(untagged)]
    enum Aux<T> {
        T(T),
        Empty(Empty),
        Null,
    }

    match Deserialize::deserialize(deserializer)? {
        Aux::T(t) => Ok(Some(t)),
        Aux::Empty(_) | Aux::Null => Ok(None),
    }
}
