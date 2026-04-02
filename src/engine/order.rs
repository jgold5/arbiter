use serde::{Serialize, Deserialize};
use rust_decimal::prelude::*;
use uuid::Uuid;

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
    /// the epoch timestamp of order creation in nanoseconds
    timestamp: u64,
    quantity: Decimal,
    uuid: Uuid,
}

impl Order {
    pub fn new(side: Side, order_type: OrderType, price: Option<Decimal>, quantity: Decimal) {
        // timstamp = 

        // Order {side: side, order_type: order_type, price: price: timestamp: todo, quantity: quantity, uuid: todo}

    }
}