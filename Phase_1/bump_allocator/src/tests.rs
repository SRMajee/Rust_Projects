use crate::BumpAllocator;
use std::alloc::Layout;
use std::thread;

#[test]
fn test_alloc_val_primitives() {
    let arena = BumpAllocator::new(1024);

    let val_u32 = arena.alloc_val(42u32).expect("alloc u32 failed");
    assert_eq!(*val_u32, 42);

    let val_u64 = arena.alloc_val(100_000_000_000u64).expect("alloc u64 failed");
    assert_eq!(*val_u64, 100_000_000_000);

    *val_u32 += 10;
    assert_eq!(*val_u32, 52);
}

#[test]
fn test_alignment_requirements() {
    let arena = BumpAllocator::new(2048);

    #[repr(align(64))]
    struct CacheAligned {
        #[allow(dead_code)]
        data: [u8; 64],
    }

    // Allocate an unaligned 1-byte value first
    let _byte = arena.alloc_val(7u8).unwrap();

    // Allocate 64-byte aligned structure
    let aligned = arena.alloc_val(CacheAligned { data: [1; 64] }).unwrap();
    let addr = aligned as *const CacheAligned as usize;
    assert_eq!(addr % 64, 0, "Address {:x} is not 64-byte aligned", addr);

    // Allocate 8-byte aligned value
    let u64_val = arena.alloc_val(123456789u64).unwrap();
    let addr_u64 = u64_val as *const u64 as usize;
    assert_eq!(addr_u64 % 8, 0, "Address {:x} is not 8-byte aligned", addr_u64);
}

#[test]
fn test_out_of_memory() {
    let arena = BumpAllocator::new(64);

    let layout = Layout::from_size_align(32, 8).unwrap();
    let p1 = arena.alloc_raw(layout);
    assert!(p1.is_some());

    let p2 = arena.alloc_raw(layout);
    assert!(p2.is_some());

    // Arena is now full (64 / 64 bytes)
    let p3 = arena.alloc_raw(layout);
    assert!(p3.is_none());
}

#[test]
fn test_bulk_reset() {
    let mut arena = BumpAllocator::new(128);

    let layout = Layout::from_size_align(64, 8).unwrap();
    assert!(arena.alloc_raw(layout).is_some());
    assert!(arena.alloc_raw(layout).is_some());
    assert!(arena.alloc_raw(layout).is_none()); // Full

    // Reset clears offset
    arena.reset();
    assert_eq!(arena.allocated_bytes(), 0);

    // Can allocate again
    assert!(arena.alloc_raw(layout).is_some());
}

#[test]
fn test_zero_sized_types() {
    let arena = BumpAllocator::new(128);

    struct Empty;
    let empty = arena.alloc_val(Empty).unwrap();
    assert_eq!(arena.allocated_bytes(), 0); // No bytes bumped for ZST
    let _ = empty;
}

#[test]
fn test_multithreaded_concurrent_allocations() {
    let arena = std::sync::Arc::new(BumpAllocator::new(1024 * 1024));
    let mut handles = Vec::new();

    for thread_id in 0..8 {
        let arena_clone = arena.clone();
        handles.push(thread::spawn(move || {
            for i in 0..100 {
                let val = arena_clone.alloc_val((thread_id * 1000 + i) as u64).unwrap();
                assert_eq!(*val, (thread_id * 1000 + i) as u64);
            }
        }));
    }

    for h in handles {
        h.join().unwrap();
    }
}
