use std::{cmp::Reverse, collections::{BTreeMap, HashMap, VecDeque}};

use rust_decimal::Decimal;
use uuid::Uuid;

use crate::engine::{event::{EventPayload}, order::{Order, OrderType, Side}};

/// The live state of all resting orders for a single instrument.
///  
/// The order book maintains two sorted price ladders -- bids (descending) and asks (ascending) --
/// each backed by a FIFO queue of orders at every price level. A secondary index maps order IDs to
/// their side and price level, enabling O(1) lookups for fills and cancellations.
/// 
/// The book is a projection: it does not accept orders directly but is updated by applying [`EventPayload`]
/// variants emitted by the matching engine.
#[derive(Debug, PartialEq)]
pub struct OrderBook {
    bids: BTreeMap<Reverse<Decimal>, VecDeque<Order>>,
    asks: BTreeMap<Decimal, VecDeque<Order>>,
    order_index: HashMap<Uuid, (Side, Decimal)>

}

/// Errors that can occur when applying an event to the order book.
#[derive(Debug, PartialEq)]
pub enum OrderBookError {
    /// No resting order with the given ID exists in the book.
    OrderNotFound,
    /// An order with the given ID is already present in the book.
    DuplicateOrder,
    /// Market orders cannot rest in the book and must be matched immediately upon arrival.
    InvalidOrderType
}

impl Default for OrderBook {
    fn default() -> Self {
        Self::new()
    }
}

impl OrderBook {

    /// Initializes a new order book with no resting orders. 
    pub fn new() -> OrderBook {
        let bids = BTreeMap::new();
        let asks = BTreeMap::new();
        let order_index = HashMap::new();
        OrderBook { bids, asks, order_index }
    }

    /// Applies an event to the order book, updating its state accordingly.
    /// - [`EventPayload::OrderSubmitted`] - adds the order to the appropriate price level
    /// - [`EventPayload::OrderFilled`] - reduces or removes the filled order(s) from the book
    /// - [`EventPayload::OrderCancelled`] - removes the order from the book
    pub fn apply(&mut self, event: &EventPayload) -> Result<(), OrderBookError> {
        match event {
            EventPayload::OrderSubmitted(submitted)  => {
                let order = submitted.get_order();
                match order.get_order_type() {
                    OrderType::Market => Err(OrderBookError::InvalidOrderType),
                    OrderType::Limit => {
                        match order.get_side() {
                            Side::Bid => {
                                if self.order_index.contains_key(&order.get_order_id()) {
                                    Err(OrderBookError::DuplicateOrder)
                                } else {
                                    self.bids.entry(Reverse(order.get_price().unwrap())).or_insert_with(VecDeque::new).push_back(order.to_owned());
                                    self.order_index.insert(order.get_order_id(), (Side::Bid, order.get_price().unwrap()));
                                    Ok(())
                                }
                            }
                            Side::Ask => {
                                if self.order_index.contains_key(&order.get_order_id()) {
                                    Err(OrderBookError::DuplicateOrder)
                                } else {
                                    self.asks.entry(order.get_price().unwrap()).or_insert_with(VecDeque::new).push_back(order.to_owned());
                                    self.order_index.insert(order.get_order_id(), (Side::Ask, order.get_price().unwrap()));
                                    Ok(())
                                }
                            }
                        }
                    }
                }
            }
            EventPayload::OrderFilled(filled)=> {
                let order_id = filled.get_order_id();
                let matched_order_id = filled.get_matched_order_id();
                let order_entry = self.order_index.get(&order_id).cloned();
                let matched_order_entry = self.order_index.get(&matched_order_id).cloned();
                if let Some((side, price)) = order_entry {
                    self.update_order(order_id, side, price, filled.get_quantity());
                }
                if let Some((side, price)) = matched_order_entry { 
                    self.update_order(matched_order_id, side, price, filled.get_quantity());
                }
                Ok(())
            }
            EventPayload::OrderCancelled(cancelled) => {
                let order_id = cancelled.get_order_id();
                let order_entry = self.order_index.get(&order_id);
                if order_entry.is_none() {
                    Err(OrderBookError::OrderNotFound)
                } else {
                    let (side, price) = order_entry.unwrap().clone();
                    self.remove_order(order_id, side, price);
                    Ok(())
                }
            }
        }
    }

    fn update_order(&mut self, order_id: Uuid, side: Side, price: Decimal, filled_quantity: Decimal) {
        match side {
            Side::Bid => {
                let all_orders_at_price  = self.bids.get_mut(&Reverse(price)).unwrap();
                let i = all_orders_at_price.iter().position(|o| o.get_order_id() == order_id).unwrap();
                let order_to_update = all_orders_at_price.remove(i).unwrap();
                let curr_qty = order_to_update.get_quantity();
                if filled_quantity >= curr_qty {
                    self.order_index.remove(&order_id);
                    if all_orders_at_price.is_empty() {
                        self.bids.remove(&Reverse(price));
                    }
                } else {
                    let new_qty = curr_qty - filled_quantity;
                    let updated_order = Order::new(order_to_update.get_instrument_id(), Side::Bid, OrderType::Limit, order_to_update.get_price(), new_qty, order_to_update.get_timestamp(), order_to_update.get_order_id());
                    all_orders_at_price.push_front(updated_order);
                }
            }
            Side::Ask => {
                let all_orders_at_price  = self.asks.get_mut(&price).unwrap();
                let i = all_orders_at_price.iter().position(|o| o.get_order_id() == order_id).unwrap();
                let order_to_update = all_orders_at_price.remove(i).unwrap();
                let curr_qty = order_to_update.get_quantity();
                if filled_quantity >= curr_qty {
                    self.order_index.remove(&order_id);
                    if all_orders_at_price.is_empty() {
                        self.asks.remove(&price);
                    }
                } else {
                    let new_qty = curr_qty - filled_quantity;
                    let updated_order = Order::new(order_to_update.get_instrument_id(), Side::Ask, OrderType::Limit, order_to_update.get_price(), new_qty, order_to_update.get_timestamp(), order_to_update.get_order_id());
                    all_orders_at_price.push_front(updated_order);
                }
            }
        }
    }

    fn remove_order(&mut self, order_id: Uuid, side: Side, price: Decimal) {
        match side {
            Side::Bid => {
                let all_orders_at_price  = self.bids.get_mut(&Reverse(price)).unwrap();
                all_orders_at_price.retain(|o| o.get_order_id() != order_id);
                if all_orders_at_price.is_empty() {
                    self.bids.remove(&Reverse(price));
                }
                self.order_index.remove(&order_id);
            }
            Side::Ask => {
                let all_orders_at_price  = self.asks.get_mut(&price).unwrap();
                all_orders_at_price.retain(|o| o.get_order_id() != order_id);
                if all_orders_at_price.is_empty() {
                    self.asks.remove(&price);
                }
                self.order_index.remove(&order_id);
            }
        }
    }

}