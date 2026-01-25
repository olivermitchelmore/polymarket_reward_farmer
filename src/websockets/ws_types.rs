use alloy::primitives::B256;
use polymarket_client_sdk::types::Decimal;

#[derive(Debug)]
pub enum OrderSide {
    Buy,
    Sell,
}

#[derive(Debug)]
pub enum UserData {
    Placed(PlacedOrder),
    Cancelled(String),
    Filled(OrderFill),
}

#[derive(Debug)]
pub struct OrderFill {
    pub order_id: String,
    pub amount: Decimal,
}
#[derive(Debug)]
pub struct PlacedOrder {
    pub order_id: String,
    pub price: Decimal,
    pub side: OrderSide,
}

#[derive(Debug)]
pub struct MarketData {
    pub best_bid: Decimal,
    pub best_ask: Decimal,
}

#[derive(Debug)]
pub struct ChannelMessage {
    pub market_id: B256,
    pub channel_data: ChannelData,
}
#[derive(Debug)]
pub enum ChannelData {
    UserData(UserData),
    MarketData(MarketData),
}
