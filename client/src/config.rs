use std::{fmt, fs::File, io::BufReader, path::Path, str::FromStr};

use eyre::{eyre, Result, WrapErr};
use iroha_config::derive::Configurable;
use iroha_crypto::{PrivateKey, PublicKey};
use iroha_data_model::{prelude::*, transaction};
use iroha_logger::Configuration as LoggerConfiguration;
use serde::{Deserialize, Serialize};
use small::SmallStr;

const DEFAULT_TORII_TELEMETRY_URL: &str = "127.0.0.1:8180";
const DEFAULT_TRANSACTION_TIME_TO_LIVE_MS: u64 = 100_000;
const DEFAULT_TRANSACTION_STATUS_TIMEOUT_MS: u64 = 10_000;
const DEFAULT_ADD_TRANSACTION_NONCE: bool = false;

/// Wrapper over `SmallStr` to provide basic auth login checking
#[derive(Clone, Serialize, Debug)]
pub struct WebLogin(SmallStr);

impl WebLogin {
    /// Construct new `WebLogin`
    ///
    /// # Errors
    /// Fails if `login` contains `:` character
    pub fn new(login: &str) -> Result<Self> {
        Self::from_str(login)
    }
}

impl FromStr for WebLogin {
    type Err = eyre::ErrReport;
    fn from_str(login: &str) -> Result<Self> {
        if login.contains(':') {
            return Err(eyre!("WebLogin cannot contain `:` character"));
        }

        Ok(Self(SmallStr::from_str(login)))
    }
}

impl fmt::Display for WebLogin {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

/// Deserializing `WebLogin` with `FromStr` implementation
impl<'de> Deserialize<'de> for WebLogin {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        FromStr::from_str(&s).map_err(serde::de::Error::custom)
    }
}

/// Basic Authentication credentials
#[derive(Clone, Deserialize, Serialize, Debug, Configurable)]
pub struct BasicAuth {
    /// Login for Basic Authentication
    pub web_login: WebLogin,
    /// Password for Basic Authentication
    pub password: SmallStr,
}

/// `Configuration` provides an ability to define client parameters such as `TORII_URL`.
// TODO: design macro to load config from env.
#[derive(Clone, Deserialize, Serialize, Debug, Configurable)]
#[serde(rename_all = "UPPERCASE")]
#[serde(default)]
#[config(env_prefix = "IROHA_")]
pub struct Configuration {
    /// Public key of the user account.
    #[config(serde_as_str)]
    pub public_key: PublicKey,
    /// Private key of the user account.
    pub private_key: PrivateKey,
    /// User account id.
    pub account_id: AccountId,
    /// Basic Authentication credentials
    pub basic_auth: Option<BasicAuth>,
    /// Torii URL.
    pub torii_api_url: SmallStr,
    /// Status URL.
    pub torii_telemetry_url: SmallStr,
    /// Proposed transaction TTL in milliseconds.
    pub transaction_time_to_live_ms: u64,
    /// Transaction status wait timeout in milliseconds.
    pub transaction_status_timeout_ms: u64,
    /// Limits to which transactions must adhere to
    pub transaction_limits: TransactionLimits,
    /// If `true` add nonce, which make different hashes for transactions which occur repeatedly and simultaneously
    pub add_transaction_nonce: bool,
    /// `Logger` configuration.
    #[config(inner)]
    pub logger_configuration: LoggerConfiguration,
}

impl Default for Configuration {
    fn default() -> Self {
        Self {
            public_key: PublicKey::default(),
            private_key: PrivateKey::default(),
            account_id: AccountId::test("", ""),
            basic_auth: None,
            torii_api_url: small::SmallStr::from_str(uri::DEFAULT_API_URL),
            torii_telemetry_url: small::SmallStr::from_str(DEFAULT_TORII_TELEMETRY_URL),
            transaction_time_to_live_ms: DEFAULT_TRANSACTION_TIME_TO_LIVE_MS,
            transaction_status_timeout_ms: DEFAULT_TRANSACTION_STATUS_TIMEOUT_MS,
            transaction_limits: TransactionLimits {
                max_instruction_number: transaction::DEFAULT_MAX_INSTRUCTION_NUMBER,
                max_wasm_size_bytes: transaction::DEFAULT_MAX_WASM_SIZE_BYTES,
            },
            add_transaction_nonce: DEFAULT_ADD_TRANSACTION_NONCE,
            logger_configuration: LoggerConfiguration::default(),
        }
    }
}

impl Configuration {
    /// This method will build `Configuration` from a json *pretty* formatted file (without `:` in
    /// key names).
    ///
    /// # Panics
    /// If configuration file present, but has incorrect format.
    ///
    /// # Errors
    /// If system  fails to find a file or read it's content.
    pub fn from_path<P: AsRef<Path> + fmt::Debug>(path: P) -> Result<Configuration> {
        let file = File::open(path).wrap_err("Failed to open the config file")?;
        let reader = BufReader::new(file);
        serde_json::from_reader(reader).wrap_err("Failed to deserialize json from reader")
    }
}