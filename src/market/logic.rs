use crate::infra::config::MarketConfig;
use crate::infra::get_token_id;
use crate::market::market_types::{
    NewPrices, OpenOrder, OpenOrderStatus, Order, OrderRequest, Spreads,
};
use crate::types::TokenIds;
use crate::websockets::ws_types::{OrderFill, OrderSide, PlacedOrder};
use alloy::primitives::{B256, U256};
use anyhow::Result;
use polymarket_client_sdk::types::Decimal;

pub struct CheckOrderResult {
    place: Option<Order>,
    cancel: Option<String>,
}

pub struct Market {
    pub token_ids: TokenIds,
    pub bid_order: Option<OpenOrder>,
    pub ask_order: Option<OpenOrder>,
    pub exposure: Decimal,
    pub config: MarketConfig,
}

impl Market {
    pub async fn new(config: MarketConfig) -> Result<(Self, B256)> {
        let (token_ids, market_identifier) = get_token_id(&config.slug).await?;

        let market = Self {
            token_ids,
            bid_order: None,
            ask_order: None,
            config,
            exposure: Decimal::from(0),
        };
        Ok((market, market_identifier))
    }

    fn get_spreads(&self) -> Spreads {
        let ask = if self.exposure > self.config.exposure {
            Decimal::from(0)
        } else {
            self.config.spread
        };

        let bid = if self.exposure < -self.config.exposure {
            Decimal::from(0)
        } else {
            self.config.spread
        };

        Spreads { bid, ask }
    }

    pub fn price_update(&mut self, new_prices: NewPrices) -> Option<Vec<OrderRequest>> {
        let mut order_requests = Vec::new();
        let spreads = self.get_spreads();
        let desired_bid_price = new_prices.best_bid - spreads.bid;
        let desired_ask_price = Decimal::from(1) - new_prices.best_ask - spreads.ask;
        let size = self.config.order_size;

        let check_bid_result = Self::check_order(
            &mut self.bid_order,
            desired_bid_price,
            size,
            self.token_ids.buy_token,
        );
        let check_ask_result = Self::check_order(
            &mut self.ask_order,
            desired_ask_price,
            size,
            self.token_ids.sell_token,
        );

        match check_ask_result {
            Some(ask_result) => {
                match ask_result.cancel {
                    Some(order_id) => order_requests.push(OrderRequest::CancelOrder(order_id)),
                    None => {}
                }
                match ask_result.place {
                    Some(order) => order_requests.push(OrderRequest::PlaceOrder(order)),
                    None => {}
                }
            }
            None => {}
        }

        match check_bid_result {
            Some(bid_result) => {
                match bid_result.cancel {
                    Some(order_id) => {
                        order_requests.push(OrderRequest::CancelOrder(order_id));
                    }
                    None => {}
                }
                match bid_result.place {
                    Some(order) => order_requests.push(OrderRequest::PlaceOrder(order)),
                    None => {}
                }
            }
            None => {}
        }

        if order_requests.is_empty() {
            None
        } else {
            Some(order_requests)
        }
    }

    pub fn check_order(
        open_order: &mut Option<OpenOrder>,
        desired_price: Decimal,
        size: Decimal,
        token_id: U256,
    ) -> Option<CheckOrderResult> {
        let placed_order = match &open_order {
            Some(order) => match &order.status {
                OpenOrderStatus::Pending => None,
                OpenOrderStatus::Placed(order_id) => {
                    if order.price != desired_price {
                        let new_order = Order::new(desired_price, size, token_id);
                        dbg!("placing new order: {}", &new_order);
                        Some(CheckOrderResult {
                            place: Some(new_order),
                            cancel: Some(order_id.clone()),
                        })
                    } else {
                        None
                    }
                }
            },
            None => {
                let new_order = Order::new(desired_price, size, token_id);
                Some(CheckOrderResult {
                    place: Some(new_order),
                    cancel: None,
                })
            }
        };
        if placed_order.is_some() {
            *open_order = Some(OpenOrder::default(desired_price, OpenOrderStatus::Pending));
        }
        placed_order
    }

    pub fn check_placed_order(&mut self, placed_order: PlacedOrder) -> Option<String> {
        let mut cancel_order_id: Option<String> = None;

        let open_order = if placed_order.token_id == self.token_ids.buy_token {
            &mut self.bid_order
        } else { &mut self.ask_order };

        match open_order {
            Some(order) => {
                match &order.status {
                    OpenOrderStatus::Pending => {}
                    OpenOrderStatus::Placed(order_id) => {
                        cancel_order_id = Some(order_id.clone());
                    }
                }
                if order.price == placed_order.price {
                    order.status = OpenOrderStatus::Placed(placed_order.order_id);
                } else {
                    *open_order = Self::create_placed_order(placed_order);
                }
                cancel_order_id
            }
            None => {
                *open_order = Self::create_placed_order(placed_order);
                None
            }
        }
    }
    fn check_order_id(open_order: &Option<OpenOrder>, order_id: &String) -> bool {
        if let Some(order) = open_order {
            match &order.status {
                OpenOrderStatus::Pending => false,
                OpenOrderStatus::Placed(placed_order_id) => {
                    if order_id == placed_order_id {
                        return true;
                    } else {
                        false
                    }
                }
            }
        } else {
            false
        }
    }
    fn get_order_side_from_id(&self, order_id: &String) -> Option<OrderSide> {
        if Self::check_order_id(&self.bid_order, order_id) {
            Some(OrderSide::Buy)
        } else if Self::check_order_id(&self.ask_order, order_id) {
            Some(OrderSide::Sell)
        } else {
            None
        }
    }
    fn create_placed_order(placed_order: PlacedOrder) -> Option<OpenOrder> {
        Some(OpenOrder::default(
            placed_order.price,
            OpenOrderStatus::Placed(placed_order.order_id),
        ))
    }
    pub fn order_canceled(&mut self, order_id: String) {
        let order = self.get_order_side_from_id(&order_id);
        match order {
            Some(order_side) => match order_side {
                OrderSide::Buy => {
                    self.bid_order = None;
                }
                OrderSide::Sell => {
                    self.ask_order = None;
                }
            },
            None => {}
        }
    }
    pub fn order_filled(&mut self, fill: OrderFill) {
        // update exposure even if the order isnt assigned to the struct
        let order = self.get_order_side_from_id(&fill.order_id);
        match order {
            Some(order_side) => match order_side {
                OrderSide::Buy => {
                    if let Some(order) = &mut self.bid_order {
                        order.matched += Decimal::from(fill.amount);
                        self.exposure += Decimal::from(fill.amount);
                        if order.matched >= self.config.order_size {
                            self.bid_order = None;
                        }
                    }
                }
                OrderSide::Sell => {
                    if let Some(order) = &mut self.ask_order {
                        order.matched += Decimal::from(fill.amount);
                        self.exposure -= Decimal::from(fill.amount);
                        if order.matched >= self.config.order_size {
                            self.ask_order = None;
                        }
                    }
                }
            },
            None => {}
        }
    }
}
