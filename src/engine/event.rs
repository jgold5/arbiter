use crate::engine::order::Order;
use crate::util::clock::HlcTimestamp;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Marker trait to group all events together
pub trait Event {}

/// Any event to represent any order submitted on the exchange
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct OrderSubmitted {order: Order}

impl OrderSubmitted {
    pub fn new(order: Order) -> OrderSubmitted {
        OrderSubmitted { order }
    }
}

impl Event for OrderSubmitted {}

/// An order filled event representing partially or fully filled orders. Contains the id of the order,
/// the id of the order that it was filled against, the quantity, and the price
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct OrderFilled {
    order_id: Uuid,
    matched_order_id: Uuid,
    quantity: Decimal,
    price: Decimal, 
}

impl OrderFilled {
    pub fn new(order_id: Uuid, matched_order_id: Uuid, quantity: Decimal, price: Decimal) -> OrderFilled {
        OrderFilled { order_id, matched_order_id, quantity, price }
    }
}

impl Event for OrderFilled {}

/// An event to indicate the cancellation of an order
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct OrderCancelled {uuid: Uuid}

impl OrderCancelled {
    pub fn new(uuid: Uuid) -> OrderCancelled {
        OrderCancelled { uuid }
    }
}

impl Event for OrderCancelled {}


/// An enum representing all types of events
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub enum EventPayload {
    OrderSubmitted(OrderSubmitted),
    OrderFilled(OrderFilled),
    OrderCancelled(OrderCancelled)
}

/// An envelope containing  the type and data of an event
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct EventEnvelope {
    id: Uuid,
    timestamp: HlcTimestamp,
    payload: EventPayload
}

impl EventEnvelope {

    pub fn new(id: Uuid, timestamp: HlcTimestamp, payload: EventPayload) -> EventEnvelope {
        EventEnvelope {id, timestamp, payload}
    }
}