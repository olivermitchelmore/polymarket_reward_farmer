use alloy::primitives::U256;

pub struct TokenIds {
    pub buy_token: U256,
    pub sell_token: U256,
}

impl TokenIds {
    pub fn new(buy_token: U256, sell_token: U256) -> Self {
        Self {
            buy_token,
            sell_token,
        }
    }
}
