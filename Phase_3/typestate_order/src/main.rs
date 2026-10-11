use typestate_order::{Cancelled, Delivered, Draft, Order, OrderItem, Paid, Shipped};

fn main() {
    println!("=== Rust Typestate Pattern: Order Flow Demo ===\n");

    // 1. ZST Zero-Cost Proof
    println!("1. Zero-Cost Memory Verification:");
    println!("   size_of::<Draft>()     = {} bytes", std::mem::size_of::<Draft>());
    println!("   size_of::<Paid>()      = {} bytes", std::mem::size_of::<Paid>());
    println!("   size_of::<Shipped>()   = {} bytes", std::mem::size_of::<Shipped>());
    println!("   size_of::<Delivered>() = {} bytes", std::mem::size_of::<Delivered>());
    println!("   size_of::<Cancelled>() = {} bytes", std::mem::size_of::<Cancelled>());
    println!(
        "   size_of::<Order<Draft>>() == size_of::<Order<Shipped>>(): {}\n",
        std::mem::size_of::<Order<Draft>>() == std::mem::size_of::<Order<Shipped>>()
    );

    // 2. Happy Path Workflow
    println!("2. Happy Path Flow: Draft -> Paid -> Shipped -> Delivered");
    let order = Order::new(1001)
        .add_item(OrderItem {
            name: "Rust Programming Book".into(),
            price_cents: 4999,
        })
        .add_item(OrderItem {
            name: "Mechanical Keyboard".into(),
            price_cents: 12900,
        });

    println!(
        "   [Draft] Order #{}: {} items, total: ${:.2}",
        order.id(),
        order.items().len(),
        order.total_cents() as f64 / 100.0
    );

    // Transition: Draft -> Paid
    let paid_order = order.pay().expect("Payment succeeded");
    println!("   [Paid] Order #{} payment authorized.", paid_order.id());

    // Transition: Paid -> Shipped
    let shipped_order = paid_order.ship("DHL-EXPRESS-987654321");
    println!(
        "   [Shipped] Order #{} dispatched with tracking: {}",
        shipped_order.id(),
        shipped_order.tracking_number()
    );

    // Transition: Shipped -> Delivered
    let delivered_order = shipped_order.deliver();
    println!(
        "   [Delivered] Order #{} marked delivered! Tracking: {}\n",
        delivered_order.id(),
        delivered_order.tracking_number()
    );

    // 3. Cancellation Alternative Path
    println!("3. Cancellation Flow (Paid -> Cancelled):");
    let order2 = Order::new(1002)
        .add_item(OrderItem {
            name: "Noise-Cancelling Headphones".into(),
            price_cents: 29900,
        })
        .pay()
        .expect("Payment succeeded");

    let cancelled_order = order2.cancel("Customer requested refund before dispatch");
    println!(
        "   [Cancelled] Order #{} cancelled. Reason: {}\n",
        cancelled_order.id(),
        cancelled_order.cancellation_reason()
    );

    println!("All typestate transitions completed safely with compile-time guarantees!");
}
