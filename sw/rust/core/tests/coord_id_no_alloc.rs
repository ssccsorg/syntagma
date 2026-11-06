// The native pair, counted rather than timed.
//
// `CoordId` prints one character an axis and reads that text back. Both are on
// the record path: an id is printed to be stored and read back to be addressed,
// so neither may touch the heap. The reader is the one that could: an earlier
// version collected the characters into a Vec before validating them.
//
// The counter is process-wide, so this file holds one test.

use std::alloc::{GlobalAlloc, Layout, System};
use std::sync::atomic::{AtomicUsize, Ordering};

use tagma_core::CoordId;

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

/// A buffer on the stack, so printing is measured without the string it would
/// otherwise build.
struct StackText {
    bytes: [u8; 60],
    len: usize,
}

impl core::fmt::Write for StackText {
    fn write_str(&mut self, text: &str) -> core::fmt::Result {
        let end = self.len.checked_add(text.len()).ok_or(core::fmt::Error)?;
        if end > self.bytes.len() {
            return Err(core::fmt::Error);
        }
        self.bytes[self.len..end].copy_from_slice(text.as_bytes());
        self.len = end;
        Ok(())
    }
}

#[test]
fn the_canonical_text_is_read_and_written_without_the_heap() {
    let text: String = (0..20u16)
        .map(|index| char::from_u32(0xAC00 + u32::from(index)).expect("a coordinate"))
        .collect();

    let before = ALLOCATIONS.load(Ordering::Relaxed);
    let id = std::hint::black_box(CoordId::<20>::from_string(&text));
    let after = ALLOCATIONS.load(Ordering::Relaxed);
    assert_eq!(
        after, before,
        "from_string allocated; an id is read per key"
    );

    let id = id.expect("a canonical id");

    let mut out = StackText {
        bytes: [0; 60],
        len: 0,
    };
    let before = ALLOCATIONS.load(Ordering::Relaxed);
    core::fmt::write(&mut out, format_args!("{id}")).expect("20 characters fit 60 bytes");
    let after = ALLOCATIONS.load(Ordering::Relaxed);
    assert_eq!(
        after, before,
        "Display allocated; an id is printed per record"
    );

    let printed = core::str::from_utf8(&out.bytes[..out.len]).expect("the writer stored UTF-8");
    assert_eq!(printed, text, "the printed text is not the text read back");

    // The counter is the instrument, so it is checked with an allocation that is
    // meant to be counted. Without this a build where the hook stopped being
    // applied would pass every measurement above.
    let before = ALLOCATIONS.load(Ordering::Relaxed);
    let probe = std::hint::black_box(String::from("probe"));
    let after = ALLOCATIONS.load(Ordering::Relaxed);
    assert!(after > before, "the counting allocator is not measuring");
    drop(probe);
}
