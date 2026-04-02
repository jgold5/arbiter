use serde::{Serialize, Deserialize};
use rust_decimal::prelude::*;
use uuid::Uuid;
use crate::util::clock::HlcTimestamp;

/// Represents which side of the order book an order belongs to.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum Side {
    Bid,
    Ask
}

/// Represents the type of order that a given order belongs to.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum OrderType {
    Limit,
    Market 
}

/// The struct representation of an order
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Order {
    side: Side,
    order_type: OrderType,
    /// the price of the order or [`None`] for [`OrderType::Market`] orders
    price: Option<Decimal>,
    /// Timestamp using a Hybrid Logical Clock
    timestamp: HlcTimestamp,
    quantity: Decimal,
    uuid: Uuid,
}

impl Order {
    pub fn new(side: Side, order_type: OrderType, price: Option<Decimal>, quantity: Decimal, timestamp: HlcTimestamp, uuid: Uuid) -> Order {
        Order {side, order_type, price, quantity, timestamp, uuid}
    }
}