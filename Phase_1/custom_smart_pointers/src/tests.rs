use super::*;
use std::cell::RefCell;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

/// Drop detector to verify that Drop executes precisely as intended.
struct DropSpy {
    drop_count: Arc<AtomicUsize>,
}

impl DropSpy {
    fn new(counter: Arc<AtomicUsize>) -> Self {
        Self {
            drop_count: counter,
        }
    }
}

impl Drop for DropSpy {
    fn drop(&mut self) {
        self.drop_count.fetch_add(1, Ordering::SeqCst);
    }
}

#[test]
fn test_my_box_basic() {
    let b = MyBox::new(42);
    assert_eq!(*b, 42);
}

#[test]
fn test_my_box_deref_mut() {
    let mut b = MyBox::new(String::from("Hello"));
    b.push_str(", world!");
    assert_eq!(*b, "Hello, world!");
}

#[test]
fn test_my_box_drop_execution() {
    let drop_counter = Arc::new(AtomicUsize::new(0));
    {
        let _b = MyBox::new(DropSpy::new(drop_counter.clone()));
        assert_eq!(drop_counter.load(Ordering::SeqCst), 0);
    }
    // _b exited scope, drop should have been called exactly once
    assert_eq!(drop_counter.load(Ordering::SeqCst), 1);
}

#[test]
fn test_my_box_zst() {
    // Zero-sized type test (must not attempt zero-size alloc)
    struct Zst;
    let b = MyBox::new(Zst);
    let _ = *b;
    drop(b);
}

#[test]
fn test_my_box_into_inner() {
    let drop_counter = Arc::new(AtomicUsize::new(0));
    let b = MyBox::new(DropSpy::new(drop_counter.clone()));
    let inner = MyBox::into_inner(b);
    assert_eq!(drop_counter.load(Ordering::SeqCst), 0);
    drop(inner);
    assert_eq!(drop_counter.load(Ordering::SeqCst), 1);
}

#[test]
fn test_my_rc_basic() {
    let rc1 = MyRc::new(100);
    assert_eq!(*rc1, 100);
    assert_eq!(MyRc::strong_count(&rc1), 1);

    let rc2 = rc1.clone();
    assert_eq!(*rc2, 100);
    assert_eq!(MyRc::strong_count(&rc1), 2);
    assert_eq!(MyRc::strong_count(&rc2), 2);
}

#[test]
fn test_my_rc_drop_execution() {
    let drop_counter = Arc::new(AtomicUsize::new(0));
    {
        let rc1 = MyRc::new(DropSpy::new(drop_counter.clone()));
        assert_eq!(drop_counter.load(Ordering::SeqCst), 0);
        assert_eq!(MyRc::strong_count(&rc1), 1);

        {
            let _rc2 = rc1.clone();
            assert_eq!(MyRc::strong_count(&rc1), 2);
            assert_eq!(drop_counter.load(Ordering::SeqCst), 0);
        }
        // rc2 dropped, rc1 still alive -> drop_count should still be 0
        assert_eq!(MyRc::strong_count(&rc1), 1);
        assert_eq!(drop_counter.load(Ordering::SeqCst), 0);
    }
    // rc1 dropped -> drop_count should now be 1
    assert_eq!(drop_counter.load(Ordering::SeqCst), 1);
}

#[test]
fn test_my_rc_interior_mutability() {
    // Shared pointer with interior mutability pattern (RefCell)
    let shared = MyRc::new(RefCell::new(vec![1, 2, 3]));
    let clone = shared.clone();

    clone.borrow_mut().push(4);

    assert_eq!(*shared.borrow(), vec![1, 2, 3, 4]);
}

#[test]
fn test_my_rc_zst() {
    // Rc over ZST still has a control block
    struct Empty;
    let rc1 = MyRc::new(Empty);
    let rc2 = rc1.clone();
    assert_eq!(MyRc::strong_count(&rc1), 2);
    drop(rc1);
    assert_eq!(MyRc::strong_count(&rc2), 1);
}
