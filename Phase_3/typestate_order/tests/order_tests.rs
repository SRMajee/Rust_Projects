use typestate_order::{
    Cancelled, Delivered, Draft, Order, OrderError, OrderItem, Paid, Shipped,
};

#[test]
fn test_full_order_lifecycle() {
    let order = Order::new(1)
        .add_item(OrderItem {
            name: "Mechanical Keyboard".into(),
            price_cents: 12000,
        })
        .add_item(OrderItem {
            name: "Mousepad".into(),
            price_cents: 2500,
        });

    assert_eq!(order.id(), 1);
    assert_eq!(order.items().len(), 2);
    assert_eq!(order.total_cents(), 14500);

    let paid = order.pay().expect("payment should succeed for non-empty order");
    assert_eq!(paid.id(), 1);
    assert_eq!(paid.total_cents(), 14500);

    let shipped = paid.ship("FEDEX-12345");
    assert_eq!(shipped.tracking_number(), "FEDEX-12345");

    let delivered = shipped.deliver();
    assert_eq!(delivered.tracking_number(), "FEDEX-12345");
}

#[test]
fn test_empty_order_cannot_be_paid() {
    let order = Order::new(2);
    assert_eq!(order.items().len(), 0);
    assert_eq!(order.total_cents(), 0);

    let result = order.pay();
    assert_eq!(result, Err(OrderError::EmptyOrder));
}

#[test]
fn test_draft_cancellation() {
    let order = Order::new(3).add_item(OrderItem {
        name: "Desk Lamp".into(),
        price_cents: 3500,
    });

    let cancelled = order.cancel("Buyer changed their mind");
    assert_eq!(cancelled.id(), 3);
    assert_eq!(cancelled.cancellation_reason(), "Buyer changed their mind");
    assert_eq!(cancelled.total_cents(), 3500);
}

#[test]
fn test_paid_cancellation_and_refund() {
    let order = Order::new(4)
        .add_item(OrderItem {
            name: "USB Cable".into(),
            price_cents: 900,
        })
        .pay()
        .expect("payment should succeed");

    let cancelled = order.cancel("Out of stock at fulfillment warehouse");
    assert_eq!(cancelled.id(), 4);
    assert_eq!(
        cancelled.cancellation_reason(),
        "Out of stock at fulfillment warehouse"
    );
}

#[test]
fn test_zero_sized_types_guarantee() {
    assert_eq!(std::mem::size_of::<Draft>(), 0);
    assert_eq!(std::mem::size_of::<Paid>(), 0);
    assert_eq!(std::mem::size_of::<Shipped>(), 0);
    assert_eq!(std::mem::size_of::<Delivered>(), 0);
    assert_eq!(std::mem::size_of::<Cancelled>(), 0);
}

#[test]
fn test_phantom_data_zero_cost_abstraction() {
    // Memory layout size of Order<S> must be invariant of the state parameter S
    let draft_sz = std::mem::size_of::<Order<Draft>>();
    let paid_sz = std::mem::size_of::<Order<Paid>>();
    let shipped_sz = std::mem::size_of::<Order<Shipped>>();
    let delivered_sz = std::mem::size_of::<Order<Delivered>>();
    let cancelled_sz = std::mem::size_of::<Order<Cancelled>>();

    assert_eq!(draft_sz, paid_sz);
    assert_eq!(paid_sz, shipped_sz);
    assert_eq!(shipped_sz, delivered_sz);
    assert_eq!(delivered_sz, cancelled_sz);
}
