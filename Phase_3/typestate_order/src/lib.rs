use std::marker::PhantomData;

/// Zero-sized type state markers.
#[derive(Debug, PartialEq, Eq)]
pub struct Draft;
#[derive(Debug, PartialEq, Eq)]
pub struct Paid;
#[derive(Debug, PartialEq, Eq)]
pub struct Shipped;
#[derive(Debug, PartialEq, Eq)]
pub struct Delivered;
#[derive(Debug, PartialEq, Eq)]
pub struct Cancelled;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OrderItem {
    pub name: String,
    pub price_cents: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OrderError {
    EmptyOrder,
}

/// Type-safe Order state machine enforced at compile time.
///
/// # Compile-Time Invariants
/// Invalid transitions fail at compile time:
/// ```compile_fail
/// use typestate_order::Order;
/// Order::new(1).ship("TRK123"); // Draft cannot ship
/// ```
/// ```compile_fail
/// use typestate_order::{Order, OrderItem};
/// let paid = Order::new(1).add_item(OrderItem { name: "Pen".into(), price_cents: 100 }).pay().unwrap();
/// paid.add_item(OrderItem { name: "Book".into(), price_cents: 500 }); // Paid cannot add items
/// ```
/// ```compile_fail
/// use typestate_order::{Order, OrderItem};
/// let order = Order::new(1).add_item(OrderItem { name: "Pen".into(), price_cents: 100 });
/// let _paid = order.pay().unwrap();
/// order.cancel("stale"); // Use-after-move error!
/// ```
#[derive(Debug, PartialEq, Eq)]
pub struct Order<State> {
    id: u64,
    items: Vec<OrderItem>,
    total_cents: u64,
    tracking_number: Option<String>,
    cancellation_reason: Option<String>,
    _state: PhantomData<State>,
}

impl<S> Order<S> {
    pub fn id(&self) -> u64 { self.id }
    pub fn items(&self) -> &[OrderItem] { &self.items }
    pub fn total_cents(&self) -> u64 { self.total_cents }
}

impl Order<Draft> {
    pub fn new(id: u64) -> Self {
        Self {
            id,
            items: Vec::new(),
            total_cents: 0,
            tracking_number: None,
            cancellation_reason: None,
            _state: PhantomData,
        }
    }

    pub fn add_item(mut self, item: OrderItem) -> Self {
        self.total_cents += item.price_cents;
        self.items.push(item);
        self
    }

    pub fn pay(self) -> Result<Order<Paid>, OrderError> {
        if self.items.is_empty() {
            return Err(OrderError::EmptyOrder);
        }
        Ok(Order {
            id: self.id,
            items: self.items,
            total_cents: self.total_cents,
            tracking_number: None,
            cancellation_reason: None,
            _state: PhantomData,
        })
    }

    pub fn cancel(self, reason: impl Into<String>) -> Order<Cancelled> {
        Order {
            id: self.id,
            items: self.items,
            total_cents: self.total_cents,
            tracking_number: None,
            cancellation_reason: Some(reason.into()),
            _state: PhantomData,
        }
    }
}

impl Order<Paid> {
    pub fn ship(self, tracking_number: impl Into<String>) -> Order<Shipped> {
        Order {
            id: self.id,
            items: self.items,
            total_cents: self.total_cents,
            tracking_number: Some(tracking_number.into()),
            cancellation_reason: None,
            _state: PhantomData,
        }
    }

    pub fn cancel(self, reason: impl Into<String>) -> Order<Cancelled> {
        Order {
            id: self.id,
            items: self.items,
            total_cents: self.total_cents,
            tracking_number: None,
            cancellation_reason: Some(reason.into()),
            _state: PhantomData,
        }
    }
}

impl Order<Shipped> {
    pub fn tracking_number(&self) -> &str {
        self.tracking_number.as_deref().unwrap_or_default()
    }

    pub fn deliver(self) -> Order<Delivered> {
        Order {
            id: self.id,
            items: self.items,
            total_cents: self.total_cents,
            tracking_number: self.tracking_number,
            cancellation_reason: None,
            _state: PhantomData,
        }
    }
}

impl Order<Delivered> {
    pub fn tracking_number(&self) -> &str {
        self.tracking_number.as_deref().unwrap_or_default()
    }
}

impl Order<Cancelled> {
    pub fn cancellation_reason(&self) -> &str {
        self.cancellation_reason.as_deref().unwrap_or_default()
    }
}
