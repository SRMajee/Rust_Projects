use bump_allocator::BumpAllocator;

// Backing buffer for static global allocator (1 MB)
#[repr(align(16))]
struct AlignedBuffer([u8; 1024 * 1024]);
static mut ARENA_BUFFER: AlignedBuffer = AlignedBuffer([0; 1024 * 1024]);

#[global_allocator]
static GLOBAL: BumpAllocator = unsafe {
    BumpAllocator::from_raw_parts(&raw mut ARENA_BUFFER.0 as *mut u8, 1024 * 1024)
};

#[test]
fn test_global_allocator_with_box_and_vec() {
    // Allocations here will be routed directly through BumpAllocator::alloc!
    let boxed_num = Box::new(9999u64);
    assert_eq!(*boxed_num, 9999);

    let mut vec = Vec::new();
    for i in 0..100 {
        vec.push(i * 2);
    }

    assert_eq!(vec.len(), 100);
    assert_eq!(vec[50], 100);

    let s = String::from("Hello from BumpAllocator GlobalAlloc!");
    assert!(s.starts_with("Hello"));
}
