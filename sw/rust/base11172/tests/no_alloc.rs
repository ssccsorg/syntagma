//! The numeric digits are derived without the heap.
//!
//! The record layer derives an id per record from a digest it already holds, so
//! an allocation on this path is charged to every record written and to every
//! label resolved. This counts allocations rather than timing them, because a
//! count is the same answer on every machine and a build that adds one fails
//! here.
//!
//! The counter is process-wide, so this file holds one test: a second test
//! running beside it on another thread would be counted too.

use std::alloc::{GlobalAlloc, Layout, System};
use std::sync::atomic::{AtomicUsize, Ordering};

struct Counting;

static ALLOCATIONS: AtomicUsize = AtomicUsize::new(0);

unsafe impl GlobalAlloc for Counting {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        ALLOCATIONS.fetch_add(1, Ordering::Relaxed);
        unsafe { System.alloc(layout) }
    }

    unsafe fn dealloc(&self, pointer: *mut u8, layout: Layout) {
        unsafe { System.dealloc(pointer, layout) };
    }

    unsafe fn realloc(&self, pointer: *mut u8, layout: Layout, size: usize) -> *mut u8 {
        ALLOCATIONS.fetch_add(1, Ordering::Relaxed);
        unsafe { System.realloc(pointer, layout, size) }
    }
}

#[global_allocator]
static ALLOCATOR: Counting = Counting;

#[test]
fn the_digits_are_derived_without_the_heap() {
    let digest = [0xA5u8; 32];
    let before = ALLOCATIONS.load(Ordering::Relaxed);
    let digits = std::hint::black_box(base11172::digits_be::<20, 32>(&digest));
    let after = ALLOCATIONS.load(Ordering::Relaxed);
    assert_eq!(
        after, before,
        "digits_be allocated, and the record layer derives an id per record"
    );
    let digits = digits.expect("a 256-bit value fits in 20 digits");
    assert!(
        digits.iter().any(|coord| coord.index() != 0),
        "a digest that is not zero produced only zero digits"
    );

    // The refusal path is on the same function, so it is measured too: a
    // caller that passes a value that does not fit pays the same path.
    let too_wide = [0xFFu8; 2];
    let before = ALLOCATIONS.load(Ordering::Relaxed);
    let refused = std::hint::black_box(base11172::digits_be::<1, 2>(&too_wide));
    let after = ALLOCATIONS.load(Ordering::Relaxed);
    assert_eq!(
        after, before,
        "digits_be allocated on the refusal path as well"
    );
    assert!(refused.is_none(), "a two-byte value does not fit one digit");
}
