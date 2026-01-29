use crate::types::TokenIds;
use alloy::primitives::{B256, U256};
use anyhow::{Context, Result};
use serde::Deserialize;

#[derive(Deserialize)]
struct MarketResponse {
    markets: Vec<IndividualMarket>,
}

#[derive(Deserialize)]
struct IndividualMarket {
    #[serde(rename = "clobTokenIds")]
    clob_token_ids: String,
    #[serde(rename = "conditionId")]
    condition_id: B256,
}
pub async fn get_token_id(slug: &String) -> Result<(TokenIds, B256)> {
    let url = format!("https://gamma-api.polymarket.com/events/slug/{}", slug);
    let response = reqwest::get(url)
        .await
        .context("failed to fetch clob token ids")?
        .text()
        .await?;
    let market_response: MarketResponse = serde_json::from_str(&response)
        .with_context(|| format!("Invalid market_logic response for: {}", slug))?;
    let market: &IndividualMarket = &market_response.markets[0];
    let token_vec: Vec<U256> =
        serde_json::from_str(&market.clob_token_ids).context("failed to parse token vec json")?;
    let market_identifier: B256 = market.condition_id;

    let token_ids = TokenIds {
        buy_token: token_vec[0],
        sell_token: token_vec[1],
    };
    Ok((token_ids, market_identifier))
}
