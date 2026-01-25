use crate::websockets::ws_types::{
    ChannelData, ChannelMessage, OrderFill, OrderSide, PlacedOrder, UserData,
};
use alloy::primitives::{Address, B256, U256};
use futures::StreamExt;
use polymarket_client_sdk::auth::{Credentials, Uuid};
use polymarket_client_sdk::clob::types::Side;
use polymarket_client_sdk::clob::ws::types::response::OrderMessageType;
use polymarket_client_sdk::clob::ws::{Client, WsMessage};

pub async fn connect_to_user_ws(
    tx: crossfire::MAsyncTx<ChannelMessage>,
    credentials: Credentials,
    address: Address,
) {
    let client = Client::default()
        .authenticate(credentials, address)
        .unwrap();
    let markets = Vec::new();
    let mut stream = std::pin::pin!(client.subscribe_user_events(markets).unwrap());

    while let Some(event) = stream.next().await {
        match event {
            Ok(WsMessage::Order(order)) => {
                println!("{:?}", order);
                let msg_type = order.msg_type.unwrap();
                let market_id = order.market;
                let order_id = order.id;

                let user_data: UserData = match &msg_type {
                    OrderMessageType::Placement => {
                        let price = order.price;
                        let side = match order.side {
                            Side::Buy => OrderSide::Buy,
                            Side::Sell => OrderSide::Sell,
                            _ => panic!("Unknown order side"),
                        };

                        let placed_order = PlacedOrder {
                            order_id,
                            price,
                            side,
                        };
                        UserData::Placed(placed_order)
                    }
                    OrderMessageType::Update => {
                        let amount = order.size_matched.unwrap();
                        let order_fill = OrderFill { order_id, amount };
                        UserData::Filled(order_fill)
                    }
                    OrderMessageType::Cancellation => UserData::Cancelled(order_id),
                    _ => panic!("Unknown message type"),
                };
                let channel_data = ChannelData::UserData(user_data);
                let channel_message = ChannelMessage {
                    market_id,
                    channel_data,
                };
                tx.send(channel_message).await.unwrap();
            }
            Ok(WsMessage::Trade(trade)) => {
                println!("{:?}", trade);
            }
            Ok(other) => {
                println!("other received {:?} ", other);
            }
            Err(e) => {
                eprintln!("Error in user websocket message: {e}");
            }
        }
    }
}
