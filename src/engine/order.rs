use crate::{engine::instrument::InstrumentId, util::clock::HlcTimestamp};
use rust_decimal::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Represents which side of the order book an order belongs to.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum Side {
    Bid,
    Ask,
}

/// Represents the type of order that a given order belongs to.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum OrderType {
    Limit,
    Market,
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
    order_id: Uuid,
    instrument_id: InstrumentId,
}

impl Order {
    pub fn new(
        instrument_id: InstrumentId,
        side: Side,
        order_type: OrderType,
        price: Option<Decimal>,
        quantity: Decimal,
        timestamp: HlcTimestamp,
        order_id: Uuid,
    ) -> Order {
        Order {
            instrument_id,
            side,
            order_type,
            price,
            quantity,
            timestamp,
            order_id,
        }
    }

    pub fn get_instrument_id(&self) -> InstrumentId {
        self.instrument_id.clone()
    }

    pub fn get_side(&self) -> Side {
        self.side
    }

    pub fn get_order_type(&self) -> OrderType {
        self.order_type
    }

    pub fn get_price(&self) -> Option<Decimal> {
        self.price
    }

    pub fn get_quantity(&self) -> Decimal {
        self.quantity
    }

    pub fn get_order_id(&self) -> Uuid {
        self.order_id
    }

    pub fn get_timestamp(&self) -> HlcTimestamp {
        self.timestamp
    }
}
