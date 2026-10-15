use array_vec::{ArrayVec, CapacityError};
use std::cell::Cell;
use std::rc::Rc;

#[test]
fn test_basic_push_pop_and_indexing() {
    let mut vec: ArrayVec<i32, 4> = ArrayVec::new();
    assert_eq!(vec.len(), 0);
    assert_eq!(vec.capacity(), 4);
    assert!(vec.is_empty());
    assert!(!vec.is_full());

    vec.push(10).unwrap();
    vec.push(20).unwrap();
    vec.push(30).unwrap();

    assert_eq!(vec.len(), 3);
    assert_eq!(vec[0], 10);
    assert_eq!(vec[1], 20);
    assert_eq!(vec[2], 30);
    assert_eq!(&vec[..], &[10, 20, 30]);

    // Mutation via DerefMut
    vec[1] = 25;
    assert_eq!(vec[1], 25);

    assert_eq!(vec.pop(), Some(30));
    assert_eq!(vec.pop(), Some(25));
    assert_eq!(vec.pop(), Some(10));
    assert_eq!(vec.pop(), None);
    assert!(vec.is_empty());
}

#[test]
fn test_capacity_overflow() {
    let mut vec: ArrayVec<String, 2> = ArrayVec::new();
    vec.push("hello".to_string()).unwrap();
    vec.push("world".to_string()).unwrap();
    assert!(vec.is_full());

    let err = vec.push("overflow".to_string()).unwrap_err();
    assert_eq!(err, CapacityError("overflow".to_string()));
    assert_eq!(vec.len(), 2);
    assert_eq!(vec[0], "hello");
    assert_eq!(vec[1], "world");
}

#[test]
fn test_zero_capacity() {
    let mut vec: ArrayVec<i32, 0> = ArrayVec::new();
    assert_eq!(vec.capacity(), 0);
    assert!(vec.is_empty());
    assert!(vec.is_full());
    assert_eq!(vec.pop(), None);
    assert_eq!(vec.push(42), Err(CapacityError(42)));
}

#[derive(Debug)]
struct Explicit {
    id: u64,
}

#[test]
fn test_non_default_non_copy_elements() {
    let mut vec: ArrayVec<Explicit, 3> = ArrayVec::new();
    vec.push(Explicit { id: 1 }).unwrap();
    vec.push(Explicit { id: 2 }).unwrap();
    assert_eq!(vec.len(), 2);
    assert_eq!(vec[0].id, 1);
    assert_eq!(vec[1].id, 2);
}

#[derive(Debug)]
struct DropDetector {
    drop_count: Rc<Cell<usize>>,
}

impl DropDetector {
    fn new(drop_count: Rc<Cell<usize>>) -> Self {
        Self { drop_count }
    }
}

impl Drop for DropDetector {
    fn drop(&mut self) {
        self.drop_count.set(self.drop_count.get() + 1);
    }
}

#[test]
fn test_drop_when_vec_goes_out_of_scope() {
    let drops = Rc::new(Cell::new(0));

    {
        let mut vec: ArrayVec<DropDetector, 4> = ArrayVec::new();
        vec.push(DropDetector::new(drops.clone())).unwrap();
        vec.push(DropDetector::new(drops.clone())).unwrap();
        vec.push(DropDetector::new(drops.clone())).unwrap();
        assert_eq!(drops.get(), 0);
    } // vec dropped here

    assert_eq!(drops.get(), 3);
}

#[test]
fn test_drop_on_pop_and_clear() {
    let drops = Rc::new(Cell::new(0));

    let mut vec: ArrayVec<DropDetector, 4> = ArrayVec::new();
    vec.push(DropDetector::new(drops.clone())).unwrap();
    vec.push(DropDetector::new(drops.clone())).unwrap();
    vec.push(DropDetector::new(drops.clone())).unwrap();

    let popped = vec.pop().unwrap();
    assert_eq!(drops.get(), 0); // Not dropped yet, held by `popped`
    drop(popped);
    assert_eq!(drops.get(), 1);

    vec.clear();
    assert_eq!(drops.get(), 3); // Remaining 2 elements dropped by clear()
    assert_eq!(vec.len(), 0);
}

#[test]
fn test_into_iter_full_consumption() {
    let mut vec: ArrayVec<String, 3> = ArrayVec::new();
    vec.push("a".to_string()).unwrap();
    vec.push("b".to_string()).unwrap();

    let collected: Vec<String> = vec.into_iter().collect();
    assert_eq!(collected, vec!["a".to_string(), "b".to_string()]);
}

#[test]
fn test_into_iter_partial_consumption_drop() {
    let drops = Rc::new(Cell::new(0));

    let mut vec: ArrayVec<DropDetector, 4> = ArrayVec::new();
    vec.push(DropDetector::new(drops.clone())).unwrap();
    vec.push(DropDetector::new(drops.clone())).unwrap();
    vec.push(DropDetector::new(drops.clone())).unwrap();

    let mut iter = vec.into_iter();
    let first = iter.next().unwrap();
    assert_eq!(drops.get(), 0);
    drop(first);
    assert_eq!(drops.get(), 1);

    drop(iter); // Dropping iterator should drop remaining 2 elements
    assert_eq!(drops.get(), 3);
}

#[test]
fn test_slice_operations_deref() {
    let mut vec: ArrayVec<i32, 5> = ArrayVec::new();
    vec.push(5).unwrap();
    vec.push(1).unwrap();
    vec.push(4).unwrap();
    vec.push(2).unwrap();

    vec.sort_unstable();
    assert_eq!(&vec[..], &[1, 2, 4, 5]);

    let found = vec.binary_search(&4);
    assert_eq!(found, Ok(2));
}
