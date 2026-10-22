use std::alloc::{GlobalAlloc, Layout, System};
use std::sync::atomic::{AtomicUsize, Ordering};

struct CountingAllocator;
static LARGE_ALLOCATIONS: AtomicUsize = AtomicUsize::new(0);
const DECLARED_LENGTH: usize = 16 * 1024 * 1024;

unsafe impl GlobalAlloc for CountingAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        if layout.size() >= DECLARED_LENGTH {
            LARGE_ALLOCATIONS.fetch_add(1, Ordering::Relaxed);
        }
        unsafe { System.alloc(layout) }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        unsafe { System.dealloc(ptr, layout) }
    }
}

#[global_allocator]
static ALLOCATOR: CountingAllocator = CountingAllocator;

#[test]
fn bounded_string_rejects_length_before_allocating() {
    let mut bytes = vec![0, 1, 0, 0];
    bytes.extend_from_slice(&(DECLARED_LENGTH as u32).to_le_bytes());
    let before = LARGE_ALLOCATIONS.load(Ordering::Relaxed);
    let result = cdr::deserialize_from::<_, String, _>(bytes.as_slice(), cdr::Bounded(8));
    assert!(matches!(result, Err(cdr::Error::SizeLimit)));
    assert_eq!(LARGE_ALLOCATIONS.load(Ordering::Relaxed), before);
    assert!(matches!(
        cdr::deserialize::<String>(&bytes),
        Err(cdr::Error::SizeLimit)
    ));
    assert_eq!(LARGE_ALLOCATIONS.load(Ordering::Relaxed), before);
}

#[test]
fn string_requires_a_length_including_a_null_terminator() {
    for bytes in [
        vec![0, 1, 0, 0, 0, 0, 0, 0],
        vec![0, 1, 0, 0, 2, 0, 0, 0, b'a', b'b'],
    ] {
        assert!(cdr::deserialize::<String>(&bytes).is_err());
    }
    let bytes = cdr::serialize::<_, _, cdr::CdrLe>("相机/café", cdr::Infinite).unwrap();
    assert_eq!(cdr::deserialize::<String>(&bytes).unwrap(), "相机/café");
}
