use anyhow::{Context, Result};
use dotenv::dotenv;
use polymarket_client_sdk::types::Decimal;
use serde::Deserialize;
use std::env;
use std::fs;

#[derive(Deserialize, Debug)]
pub struct MarketConfigs {
    pub markets: Vec<MarketConfig>,
}

#[derive(Deserialize, Debug)]
pub struct MarketConfig {
    pub slug: String,
    pub order_size: Decimal,
    pub spread: Decimal,
    pub max_exposure: Decimal,
}

#[derive(Debug)]
pub struct ConfigParams {
    pub funder_address: String,
    pub private_key: String,
    pub market_configs: MarketConfigs,
}

impl ConfigParams {
    pub fn new() -> Result<Self> {
        let (funder_address, private_key) = Self::load_env_vars().expect("Failed to load env vars");
        let market_configs =
            MarketConfigs::load_config().context("Failed to load market_logic configs")?;
        Ok(Self {
            funder_address,
            private_key,
            market_configs,
        })
    }
    fn load_env_vars() -> Result<(String, String)> {
        dotenv().context("Failed to read .env")?;
        let private_key = env::var("PRIVATE_KEY").context("Failed to read PRIVATE_KEY_PATH")?;
        let funder_address = env::var("FUNDER_ADDRESS").context("Failed to get funder address")?;
        Ok((funder_address, private_key))
    }
}

impl MarketConfigs {
    fn load_config() -> Result<Self> {
        let content = fs::read_to_string("config.toml").context("Failed to read config.toml")?;
        let content_toml = toml::from_str(&content).context("Failed to parse config.toml")?;

        Ok(content_toml)
    }
}
