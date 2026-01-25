use alloy::primitives::U256;
use polymarket_client_sdk::types::Decimal;

#[derive(Debug)]
pub struct Order {
    pub price: Decimal,
    pub size: Decimal,
    pub token_id: U256,
}

impl Order {
    pub fn new(price: Decimal, size: Decimal, token_id: U256) -> Self {
        Self {
            price,
            size,
            token_id,
        }
    }
}
#[derive(Debug)]
pub enum OrderRequest {
    PlaceOrder(Order),
    CancelOrder(String),
}

pub enum OpenOrderStatus {
    Pending,
    Placed(String),
}

pub struct OpenOrder {
    pub price: Decimal,
    pub status: OpenOrderStatus,
    pub matched: Decimal,
}

pub struct Spreads {
    pub bid: Decimal,
    pub ask: Decimal,
}

impl OpenOrder {
    pub fn default(price: Decimal, status: OpenOrderStatus) -> Self {
        Self {
            price,
            status,
            matched: Decimal::from(0),
        }
    }
}
pub struct NewPrices {
    pub best_bid: Decimal,
    pub best_ask: Decimal,
}
