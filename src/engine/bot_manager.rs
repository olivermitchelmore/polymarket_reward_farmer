use crate::infra::{ConfigParams, SigningUtils};
use crate::market::Market;

use crate::market::market_types::{NewPrices, Order, OrderRequest};
use crate::websockets::ws_types::{ChannelData, ChannelMessage, UserData};
use crate::websockets::{connect_to_market_ws, connect_to_user_ws};
use ahash::AHashMap;
use alloy::primitives::B256;
use crossfire::{Rx, mpsc};
use futures::future::join_all;
use polymarket_client_sdk::clob::types::Side;
use polymarket_client_sdk::types::Decimal;

pub struct BotManager {
    markets: AHashMap<B256, Market>,
    signing_utils: SigningUtils,
}

impl BotManager {
    pub async fn new() -> Self {
        let config_params = ConfigParams::new().expect("Failed to get config params");
        let signing_utils =
            SigningUtils::new_client(&config_params.private_key, &config_params.funder_address)
                .await
                .expect("Failed to create signing utils");
        let markets = Self::get_markets().await;
        Self {
            markets,
            signing_utils,
        }
    }
    pub fn run(mut self) {
        let rx = self.start_websockets();
        while let Ok(message) = rx.recv() {
            if let Some(market) = self.markets.get_mut(&message.market_id) {
                match message.channel_data {
                    ChannelData::MarketData(market_data) => {
                        let new_prices = NewPrices {
                            best_bid: market_data.best_bid,
                            best_ask: market_data.best_ask,
                        };
                        let orders = market.price_update(new_prices);
                        match orders {
                            Some(order_requests) => {
                                for order_request in order_requests {
                                    match order_request {
                                        OrderRequest::PlaceOrder(order) => self.place_order(order),
                                        OrderRequest::CancelOrder(order_id) => {
                                            self.cancel_order(order_id)
                                        }
                                    }
                                }
                            }
                            None => {}
                        }
                    }
                    ChannelData::UserData(user_data) => match user_data {
                        UserData::Placed(placed_order) => {
                            market.placed_order_update(placed_order);
                        }
                        UserData::Update(order_update) => {
                            market.order_update(order_update);
                        }
                        UserData::Cancelled(order_id) => {
                            market.canceled_order_update(order_id);
                        }
                    },
                }
            }
        }
    }

    // pub fn get_initial_prices(&self) {
    //     for (market_id, market) in &self.markets {
    //         let price_request = PriceRequest::builder()
    //             .token_id(market.token_ids.buy_token)
    //             .side(Side::Buy)
    //             .build();
    //         PriceResponse
    //     }
    // }

    // pub fn get_price(price_request: PriceRequest)

    pub fn start_websockets(&self) -> Rx<ChannelMessage> {
        let mut asset_ids = Vec::new();

        for (_, market) in &self.markets {
            asset_ids.push(market.token_ids.buy_token);
            println!("{}", market.token_ids.buy_token);
        }

        let credentials = self.signing_utils.client.credentials().clone();
        let funder_address = self.signing_utils.funder_address;

        let (tx, mut rx) = mpsc::bounded_async(5);
        let market_sender = tx.clone();
        tokio::spawn(async move { connect_to_market_ws(market_sender, asset_ids.clone()).await });
        tokio::spawn(
            async move { connect_to_user_ws(tx.clone(), credentials, funder_address).await },
        );
        rx.into_blocking()
    }
    pub async fn get_markets() -> AHashMap<B256, Market> {
        let config_params = ConfigParams::new().expect("Failed to load config");
        let mut futures = Vec::new();
        let mut markets = AHashMap::new();

        for market_config in config_params.market_configs.markets {
            futures.push(Market::new(market_config));
        }
        let assigned_market_results = join_all(futures).await;
        for assigned_market_result in assigned_market_results {
            match assigned_market_result {
                Ok((market, market_id)) => {
                    println!(
                        "market created: {}, {:?} {:?}",
                        market_id, market.token_ids.buy_token, market.token_ids.sell_token
                    );
                    markets.insert(market_id, market);
                }
                Err(e) => eprintln!("{e}"),
            }
        }
        markets
    }
    pub fn cancel_order(&self, order_id: String) {
        let client = self.signing_utils.client.clone();

        tokio::spawn(async move {
            let response = client.cancel_order(&order_id).await;
        });
    }
    pub fn place_order(&self, order: Order) {
        let client = self.signing_utils.client.clone();
        let signer = self.signing_utils.signer.clone();
        let price = Decimal::from(order.price);

        tokio::spawn(async move {
            let order = client
                .limit_order()
                .token_id(order.token_id)
                .size(order.size)
                .price(price)
                .side(Side::Buy)
                .build()
                .await
                .unwrap();
            let signed_order = client.sign(&signer, order).await.unwrap();
            let future = client.post_order(signed_order).await;
            match future {
                Ok(_) => {},
                Err(e) => eprintln!("{e}"),
            }
        });
    }
}
