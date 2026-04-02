use rust_decimal::prelude::*;
use uuid::Uuid;
use crate::engine::order::{Side, OrderType};
use serde::{Serialize, Deserialize};

/// Marker trait to group all commands together
pub trait Command {}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SubmitOrder {
    side: Side, 
    order_type: OrderType, 
    price: Option<Decimal>, 
    quantity: Decimal,
}

impl SubmitOrder {
    pub fn new(side: Side, order_type: OrderType, price: Option<Decimal>, quantity: Decimal) -> SubmitOrder {
        SubmitOrder { side, order_type, price, quantity }
    }
}

impl Command for SubmitOrder {}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CancelOrder {
    uuid: Uuid,
}

impl CancelOrder {
    pub fn new(uuid: Uuid) -> CancelOrder {
        CancelOrder { uuid }
    }
}

impl Command for CancelOrder {}