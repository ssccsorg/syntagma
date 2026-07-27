use crate::coord_path::CoordPath;
use alloc::alloc::{alloc_zeroed, Layout};
use alloc::boxed::Box;
use core::ptr;
use core::slice;

// ---------------------------------------------------------------------------
// Dense CoordSpace family via macro
// ---------------------------------------------------------------------------

/// Slot count for a dense CoordSpace at depth N.
///
/// For N=1:  11,172
/// For N=2:  124,813,584
/// For N=3:  1,394,417,360,448  (mmap required; beyond single heap alloc)
/// For N=4+: overflow; not representable in usize on 64-bit.
const fn coord_slots(n: usize) -> usize {
    match n {
        1 => 11172,
        2 => 11172 * 11172,
        3 => 11172 * 11172 * 11172,
        _ => 0, // unreachable; guarded by call sites
    }
}

/// Defines a dense CoordSpace type with a fixed depth N.
///
/// Generated API: `new`, `at_path`, `place_path`, `vacate_path`,
/// `clear`, `len`, `is_empty`, `PartialEq`, `Clone`, `Debug`.
///
/// # Allocation
///
/// A single `alloc_zeroed` call on construction. For N=2 with V=()
/// this is 119 MB; for V=u32 it is 952 MB. The caller must ensure
/// the allocation size is acceptable for their deployment.
///
/// # Safety
///
/// Relies on `Option<V>` having an all-zero `None` bit pattern
/// (same invariant as `CoordSpace`).
macro_rules! define_dense_coord_space {
    ($name:ident, $n:expr) => {
        const SLOT_COUNT: usize = coord_slots($n);

        #[doc = concat!(
            "Dense, zeroed, heap-allocated CoordSpace for N=", stringify!($n), ".\n\n",
            "All ", stringify!($n), " character(s): ", stringify!(SLOT_COUNT), " slots.\n",
            "Backed by a single `alloc_zeroed` call \u{2014} true Tagma, no hashing, no tree.\n\n",
            "# Panics\n\n",
            "Panics if `SLOT_COUNT * size_of::<Option<V>>()` overflows `usize`."
        )]
        pub struct $name<V> {
            slots: Box<[Option<V>]>,
            len: usize,
        }

        impl<V> $name<V> {
            /// Creates an empty dense space with all slots zeroed (None).
            ///
            /// Allocation size: `SLOT_COUNT * size_of::<Option<V>>()` bytes.
            /// Initialized via `alloc_zeroed` \u{2014} pages are lazily committed by the OS.
            #[inline]
            pub fn new() -> Self {
                let slot_bytes = core::mem::size_of::<Option<V>>()
                    .checked_mul(SLOT_COUNT)
                    .expect("overflow in CoordSpace dense allocation size");
                let layout = Layout::from_size_align(slot_bytes, core::mem::align_of::<Option<V>>())
                    .expect("invalid Layout for CoordSpace dense array");
                let ptr = unsafe { alloc_zeroed(layout) as *mut Option<V> };
                assert!(!ptr.is_null(), "CoordSpace dense allocation failed");
                let slots = unsafe { Box::from_raw(slice::from_raw_parts_mut(ptr, SLOT_COUNT)) };
                $name { slots, len: 0 }
            }

            #[inline]
            pub fn len(&self) -> usize {
                self.len
            }

            #[inline]
            pub fn is_empty(&self) -> bool {
                self.len == 0
            }

            pub fn at_path(&self, path: &CoordPath<$n>) -> Option<&V> {
                let idx = linear_index::<$n>(path);
                debug_assert!(idx < SLOT_COUNT, "CoordSpaceN at_path: index {} out of bounds (max {})", idx, SLOT_COUNT - 1);
                // SAFETY: linear_index returns a value < SLOT_COUNT for valid CoordPaths.
                unsafe { (*self.slots.as_ptr().add(idx)).as_ref() }
            }

            pub fn place_path(&mut self, path: &CoordPath<$n>, value: V) -> Option<V> {
                let idx = linear_index::<$n>(path);
                debug_assert!(idx < SLOT_COUNT, "CoordSpaceN place_path: index {} out of bounds (max {})", idx, SLOT_COUNT - 1);
                let slot = unsafe { &mut *self.slots.as_mut_ptr().add(idx) };
                let prev = slot.take();
                *slot = Some(value);
                if prev.is_none() {
                    self.len += 1;
                }
                prev
            }

            pub fn vacate_path(&mut self, path: &CoordPath<$n>) -> Option<V> {
                let idx = linear_index::<$n>(path);
                debug_assert!(idx < SLOT_COUNT, "CoordSpaceN vacate_path: index {} out of bounds (max {})", idx, SLOT_COUNT - 1);
                let slot = unsafe { &mut *self.slots.as_mut_ptr().add(idx) };
                let prev = slot.take();
                if prev.is_some() {
                    self.len -= 1;
                }
                prev
            }

            pub fn clear(&mut self) {
                let ptr = self.slots.as_mut_ptr() as *mut u8;
                let bytes = (SLOT_COUNT as u64)
                    .saturating_mul(core::mem::size_of::<Option<V>>() as u64);
                unsafe { ptr::write_bytes(ptr, 0, bytes as usize) };
                self.len = 0;
            }
        }

        impl<V> Default for $name<V> {
            #[inline]
            fn default() -> Self {
                Self::new()
            }
        }

        impl<V: core::fmt::Debug> core::fmt::Debug for $name<V> {
            fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                f.debug_struct(stringify!($name))
                    .field("N", &($n))
                    .field("len", &self.len)
                    .field("capacity", &SLOT_COUNT)
                    .finish()
            }
        }

        impl<V: PartialEq> PartialEq for $name<V> {
            fn eq(&self, other: &Self) -> bool {
                self.slots == other.slots
            }
        }
        impl<V: PartialEq> Eq for $name<V> {}

        impl<V: Clone> Clone for $name<V> {
            fn clone(&self) -> Self {
                $name {
                    slots: self.slots.clone(),
                    len: self.len,
                }
            }
        }

        impl<V> FromIterator<(CoordPath<$n>, V)> for $name<V> {
            fn from_iter<I: IntoIterator<Item = (CoordPath<$n>, V)>>(iter: I) -> Self {
                let mut space = Self::new();
                for (path, value) in iter {
                    space.place_path(&path, value);
                }
                space
            }
        }
    };
}

// ---------------------------------------------------------------------------
// Concrete types
// ---------------------------------------------------------------------------

define_dense_coord_space!(CoordSpace2, 2);

// (N=3+ via mmap deferred — issue #28)

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Linear index into the flat dense array for a CoordPath of depth N.
///
/// For N=1: `c0`
/// For N=2: `c0 * 11172 + c1`
/// For N=3: `(c0 * 11172 + c1) * 11172 + c2`
#[inline]
pub(crate) fn linear_index<const N: usize>(path: &CoordPath<N>) -> usize {
    let mut idx = 0usize;
    let mut i = 0;
    while i < N {
        idx = idx
            .wrapping_mul(11172)
            .wrapping_add(path.coords()[i].index() as usize);
        i += 1;
    }
    idx
}
