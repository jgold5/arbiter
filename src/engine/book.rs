use std::{
    cmp::Reverse,
    collections::{BTreeMap, HashMap, VecDeque},
};

use rust_decimal::Decimal;
use uuid::Uuid;

use crate::engine::{
    event::EventPayload,
    order::{self, Order, OrderType, Side},
};

type BidLadder = BTreeMap<Reverse<Decimal>, VecDeque<Order>>;
type AskLadder = BTreeMap<Decimal, VecDeque<Order>>;
type OrderIndex = HashMap<Uuid, (Side, Decimal)>;

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
    bids: BidLadder,
    asks: AskLadder,
    order_index: OrderIndex,
}

/// Errors that can occur when applying an event to the order book.
#[derive(Debug, PartialEq)]
pub enum OrderBookError {
    /// No resting order with the given ID exists in the book.
    OrderNotFound,
    /// An order with the given ID is already present in the book.
    DuplicateOrder,
    /// Market orders cannot rest in the book and must be matched immediately upon arrival.
    InvalidOrderType,
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
        OrderBook {
            bids,
            asks,
            order_index,
        }
    }

    pub fn get_bids(&self) -> &BidLadder {
        &self.bids
    }

    pub fn get_asks(&self) -> &AskLadder {
        &self.asks
    }

    pub fn order_exists(&self, order_id: Uuid) -> bool {
        self.order_index.contains_key(&order_id)
    }

    /// Applies an event to the order book, updating its state accordingly.
    /// - [`EventPayload::OrderSubmitted`] - adds the order to the appropriate price level
    /// - [`EventPayload::OrderFilled`] - reduces or removes the filled order(s) from the book
    /// - [`EventPayload::OrderCancelled`] - removes the order from the book
    pub fn apply(&mut self, event: &EventPayload) -> Result<(), OrderBookError> {
        match event {
            EventPayload::OrderSubmitted(submitted) => {
                let order = submitted.get_order();
                match order.get_order_type() {
                    OrderType::Market => Err(OrderBookError::InvalidOrderType),
                    OrderType::Limit => match order.get_side() {
                        Side::Bid => {
                            if self.order_index.contains_key(&order.get_order_id()) {
                                Err(OrderBookError::DuplicateOrder)
                            } else {
                                self.bids
                                    .entry(Reverse(order.get_price().unwrap()))
                                    .or_insert_with(VecDeque::new)
                                    .push_back(order.to_owned());
                                self.order_index.insert(
                                    order.get_order_id(),
                                    (Side::Bid, order.get_price().unwrap()),
                                );
                                Ok(())
                            }
                        }
                        Side::Ask => {
                            if self.order_index.contains_key(&order.get_order_id()) {
                                Err(OrderBookError::DuplicateOrder)
                            } else {
                                self.asks
                                    .entry(order.get_price().unwrap())
                                    .or_insert_with(VecDeque::new)
                                    .push_back(order.to_owned());
                                self.order_index.insert(
                                    order.get_order_id(),
                                    (Side::Ask, order.get_price().unwrap()),
                                );
                                Ok(())
                            }
                        }
                    },
                }
            }
            EventPayload::OrderFilled(filled) => {
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

    fn update_order(
        &mut self,
        order_id: Uuid,
        side: Side,
        price: Decimal,
        filled_quantity: Decimal,
    ) {
        match side {
            Side::Bid => {
                let all_orders_at_price = self.bids.get_mut(&Reverse(price)).unwrap();
                let i = all_orders_at_price
                    .iter()
                    .position(|o| o.get_order_id() == order_id)
                    .unwrap();
                let order_to_update = all_orders_at_price.remove(i).unwrap();
                let curr_qty = order_to_update.get_quantity();
                if filled_quantity >= curr_qty {
                    self.order_index.remove(&order_id);
                    if all_orders_at_price.is_empty() {
                        self.bids.remove(&Reverse(price));
                    }
                } else {
                    let new_qty = curr_qty - filled_quantity;
                    let updated_order = Order::new(
                        order_to_update.get_instrument_id(),
                        Side::Bid,
                        OrderType::Limit,
                        order_to_update.get_price(),
                        new_qty,
                        order_to_update.get_timestamp(),
                        order_to_update.get_order_id(),
                    );
                    all_orders_at_price.push_front(updated_order);
                }
            }
            Side::Ask => {
                let all_orders_at_price = self.asks.get_mut(&price).unwrap();
                let i = all_orders_at_price
                    .iter()
                    .position(|o| o.get_order_id() == order_id)
                    .unwrap();
                let order_to_update = all_orders_at_price.remove(i).unwrap();
                let curr_qty = order_to_update.get_quantity();
                if filled_quantity >= curr_qty {
                    self.order_index.remove(&order_id);
                    if all_orders_at_price.is_empty() {
                        self.asks.remove(&price);
                    }
                } else {
                    let new_qty = curr_qty - filled_quantity;
                    let updated_order = Order::new(
                        order_to_update.get_instrument_id(),
                        Side::Ask,
                        OrderType::Limit,
                        order_to_update.get_price(),
                        new_qty,
                        order_to_update.get_timestamp(),
                        order_to_update.get_order_id(),
                    );
                    all_orders_at_price.push_front(updated_order);
                }
            }
        }
    }

    fn remove_order(&mut self, order_id: Uuid, side: Side, price: Decimal) {
        match side {
            Side::Bid => {
                let all_orders_at_price = self.bids.get_mut(&Reverse(price)).unwrap();
                all_orders_at_price.retain(|o| o.get_order_id() != order_id);
                if all_orders_at_price.is_empty() {
                    self.bids.remove(&Reverse(price));
                }
                self.order_index.remove(&order_id);
            }
            Side::Ask => {
                let all_orders_at_price = self.asks.get_mut(&price).unwrap();
                all_orders_at_price.retain(|o| o.get_order_id() != order_id);
                if all_orders_at_price.is_empty() {
                    self.asks.remove(&price);
                }
                self.order_index.remove(&order_id);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        engine::{
            event::{OrderCancelled, OrderFilled, OrderSubmitted},
            instrument::InstrumentId,
        },
        util::clock::{HybridLogicalClock, TestClock},
    };

    fn make_order(
        side: Side,
        order_type: OrderType,
        price: Option<Decimal>,
        quantity: Decimal,
    ) -> Order {
        let test_clock = TestClock::new(0);
        let mut hcl = HybridLogicalClock::new(test_clock);
        Order::new(
            InstrumentId::new("AAPL".to_string()).unwrap(),
            side,
            order_type,
            price,
            quantity,
            hcl.tick(),
            Uuid::new_v4(),
        )
    }

    #[test]
    fn test_submit_limit_order() {
        let mut order_book = OrderBook::new();
        let order = make_order(
            Side::Ask,
            OrderType::Limit,
            Some(Decimal::new(100, 0)),
            Decimal::new(100, 0),
        );
        let id = order.get_order_id();
        let res = order_book.apply(&EventPayload::OrderSubmitted(OrderSubmitted::new(order)));
        assert!(res.is_ok());
        assert!(order_book.asks.contains_key(&Decimal::new(100, 0)));
        assert_eq!(order_book.get_asks().len(), 1);
        assert_eq!(order_book.get_bids().len(), 0);
        assert_eq!(order_book.order_exists(id), true);
    }

    #[test]
    fn test_cancel_limit_order() {
        let mut order_book = OrderBook::new();
        let order = make_order(
            Side::Ask,
            OrderType::Limit,
            Some(Decimal::new(100, 0)),
            Decimal::new(100, 0),
        );
        let id = order.get_order_id();
        let _ = order_book.apply(&EventPayload::OrderSubmitted(OrderSubmitted::new(order)));
        let res = order_book.apply(&&EventPayload::OrderCancelled(OrderCancelled::new(id)));
        assert!(res.is_ok());
        assert!(!order_book.asks.contains_key(&Decimal::new(100, 0)));
        assert_eq!(order_book.get_asks().len(), 0);
        assert_eq!(order_book.get_bids().len(), 0);
        assert_eq!(order_book.order_exists(id), false);
    }

    #[test]
    fn test_fill_limit_order() {
        let mut order_book = OrderBook::new();
        let order_1 = make_order(
            Side::Ask,
            OrderType::Limit,
            Some(Decimal::new(100, 0)),
            Decimal::new(100, 0),
        );
        let order_2 = make_order(
            Side::Bid,
            OrderType::Limit,
            Some(Decimal::new(100, 0)),
            Decimal::new(50, 0),
        );
        let qty = order_2.get_quantity();
        let price = order_2.get_price().unwrap();
        let id_1 = order_1.get_order_id();
        let id_2 = order_2.get_order_id();
        let _ = order_book.apply(&EventPayload::OrderSubmitted(OrderSubmitted::new(order_1)));
        let _ = order_book.apply(&EventPayload::OrderSubmitted(OrderSubmitted::new(order_2)));
        let res = order_book.apply(&EventPayload::OrderFilled(OrderFilled::new(
            id_1, id_2, qty, price,
        )));
        assert!(res.is_ok());
        assert!(order_book.asks.contains_key(&Decimal::new(100, 0)));
        assert_eq!(order_book.get_asks().len(), 1);
        assert_eq!(order_book.get_bids().len(), 0);
        assert_eq!(order_book.order_exists(id_1), true);
        assert_eq!(order_book.order_exists(id_2), false);
        let resting_order = order_book
            .get_asks()
            .get(&Decimal::new(100, 0))
            .unwrap()
            .front()
            .unwrap();
        assert_eq!(resting_order.get_quantity(), Decimal::new(50, 0));
    }

    #[test]
    fn test_submit_market_order() {
        let mut order_book = OrderBook::new();
        let order = make_order(
            Side::Ask,
            OrderType::Market,
            Some(Decimal::new(100, 0)),
            Decimal::new(100, 0),
        );
        let res = order_book.apply(&EventPayload::OrderSubmitted(OrderSubmitted::new(order)));
        assert_eq!(res, Err(OrderBookError::InvalidOrderType));
    }

    #[test]
    fn test_cancel_nonexistent_order() {
        let mut order_book = OrderBook::new();
        let order = make_order(
            Side::Ask,
            OrderType::Limit,
            Some(Decimal::new(100, 0)),
            Decimal::new(100, 0),
        );
        let id = order.get_order_id();
        let res = order_book.apply(&EventPayload::OrderCancelled(OrderCancelled::new(id)));
        assert_eq!(res, Err(OrderBookError::OrderNotFound));
    }

    #[test]
    fn test_submit_duplicate_order() {
        let mut order_book = OrderBook::new();
        let order = make_order(
            Side::Ask,
            OrderType::Limit,
            Some(Decimal::new(100, 0)),
            Decimal::new(100, 0),
        );
        let dup = order.clone();
        let _ = order_book.apply(&EventPayload::OrderSubmitted(OrderSubmitted::new(order)));
        let res = order_book.apply(&EventPayload::OrderSubmitted(OrderSubmitted::new(dup)));
        assert_eq!(res, Err(OrderBookError::DuplicateOrder));
    }
}
