use crate::websockets::ws_types::MarketData;
use crate::websockets::ws_types::{ChannelData, ChannelMessage};
use alloy::primitives::U256;
use futures::StreamExt;
use polymarket_client_sdk::clob::ws::Client;

pub async fn connect_to_market_ws(tx: crossfire::MAsyncTx<ChannelMessage>, asset_ids: Vec<U256>) {
    let client = Client::default();
    let stream_result = client.subscribe_prices(asset_ids.clone());

    match stream_result {
        Ok(stream) => {
            let mut stream = Box::pin(stream);
            while let Some(price_change) = stream.next().await {
                match price_change {
                    Ok(price) => {
                        for price_change in &price.price_changes {
                            let asset_id = price_change.asset_id;
                            if !asset_ids.contains(&asset_id) {
                                continue;
                            }
                            let best_bid = price_change.best_bid.unwrap();
                            let best_ask = price_change.best_ask.unwrap();
                            let market_id = price.market;

                            let market_data = MarketData { best_bid, best_ask };
                            let channel_message = ChannelMessage {
                                market_id,
                                channel_data: ChannelData::MarketData(market_data),
                            };
                            tx.send(channel_message).await.unwrap();
                        }
                    }
                    Err(e) => {
                        eprintln!("Error getting price change: {:?}", e);
                    }
                }
            }
        }
        Err(e) => {
            eprintln!("Error connecting to market: {:?}", e);
        }
    }
}
