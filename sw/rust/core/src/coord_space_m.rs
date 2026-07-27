use crate::coord_path::CoordPath;
use core::ptr::{self, NonNull};

// ---------------------------------------------------------------------------
// CoordSpaceM: mmap-backed dense CoordSpace (N >= 3)
// ---------------------------------------------------------------------------

/// Slot count for a mmap-backed dense CoordSpace at depth N.
///
/// Only N=3 ($11{,}172^3$) fits in `usize` on 64-bit without overflow.
/// N >= 6 exceeds `usize` (1.94e24 > u64::MAX) and is not supported.
/// Use `CoordSpaceN` (sparse tree) for depths >= 4.
const fn coord_slots(n: usize) -> usize {
    let sq = 11172usize.wrapping_mul(11172); // 124,813,584
    match n {
        3 => sq.wrapping_mul(11172), // = 11172^3, fits in 64-bit
        _ => 0,                      // unsupported depth; new() will panic
    }
}

/// A dense CoordSpace backed by an anonymous mmap allocation.
///
/// Like `CoordSpace` and `CoordSpace2`, this is true Tagma — no hashing, no tree,
/// collision-free O(1) addressing. Unlike the heap-allocated variants, the backing
/// memory is allocated via `mmap` with `MAP_ANONYMOUS | MAP_NORESERVE`, so the
/// virtual address space is reserved but physical pages are committed lazily on
/// first access. CoordSpaceM (N >= 3) enables Tagma at address spaces exceeding
/// single-node heap limits.
///
/// # Panics
///
/// `new()` panics if `mmap` fails (out of virtual address space, system limit).
///
/// # Feature gate
///
/// Requires the `mmap` feature (Unix only).
#[derive(Debug)]
pub struct CoordSpaceM<const N: usize, V> {
    ptr: NonNull<Option<V>>,
    len: usize,
}

impl<const N: usize, V> CoordSpaceM<N, V> {
    const SLOT_COUNT: usize = coord_slots(N);

    /// Returns the allocation size in bytes.
    fn alloc_size() -> usize {
        Self::SLOT_COUNT.saturating_mul(core::mem::size_of::<Option<V>>())
    }

    /// Creates an empty mmap-backed space.
    ///
    /// Reserves `SLOT_COUNT * size_of::<Option<V>>()` bytes of virtual address
    /// space via anonymous mmap. Physical pages are allocated by the kernel on
    /// first write (page fault). Overcommit is enabled via `MAP_NORESERVE`.
    #[inline]
    pub fn new() -> Self {
        let size = Self::alloc_size();
        if size == 0 || size == usize::MAX {
            panic!("CoordSpaceM: unsupported allocation size for N={}", N);
        }
        let ptr = unsafe {
            libc::mmap(
                ptr::null_mut(),
                size,
                libc::PROT_READ | libc::PROT_WRITE,
                libc::MAP_PRIVATE | libc::MAP_ANONYMOUS | libc::MAP_NORESERVE,
                -1,
                0,
            )
        };
        if ptr == libc::MAP_FAILED {
            panic!("CoordSpaceM: mmap(N={}) failed for {} bytes", N, size,);
        }
        CoordSpaceM {
            ptr: NonNull::new(ptr as *mut Option<V>).unwrap(),
            len: 0,
        }
    }

    /// Returns the number of occupied slots.
    #[inline]
    pub fn len(&self) -> usize {
        self.len
    }

    /// Returns `true` if no slots are occupied.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Returns a reference to the value at `path`, or `None`.
    #[inline]
    pub fn at_path(&self, path: &CoordPath<N>) -> Option<&V> {
        let idx = linear_index::<N>(path);
        debug_assert!(
            idx < Self::SLOT_COUNT,
            "CoordSpaceM at_path: index {} out of bounds (max {})",
            idx,
            Self::SLOT_COUNT - 1
        );
        unsafe { (*self.ptr.as_ptr().add(idx)).as_ref() }
    }

    /// Places `value` at `path`. Returns the previous value if any.
    #[inline]
    pub fn place_path(&mut self, path: &CoordPath<N>, value: V) -> Option<V> {
        let idx = linear_index::<N>(path);
        debug_assert!(
            idx < Self::SLOT_COUNT,
            "CoordSpaceM place_path: index {} out of bounds (max {})",
            idx,
            Self::SLOT_COUNT - 1
        );
        let slot = unsafe { &mut *self.ptr.as_ptr().add(idx) };
        let prev = slot.take();
        *slot = Some(value);
        if prev.is_none() {
            self.len += 1;
        }
        prev
    }

    /// Removes the value at `path`. Returns it if present.
    #[inline]
    pub fn vacate_path(&mut self, path: &CoordPath<N>) -> Option<V> {
        let idx = linear_index::<N>(path);
        debug_assert!(
            idx < Self::SLOT_COUNT,
            "CoordSpaceM vacate_path: index {} out of bounds (max {})",
            idx,
            Self::SLOT_COUNT - 1
        );
        let slot = unsafe { &mut *self.ptr.as_ptr().add(idx) };
        let prev = slot.take();
        if prev.is_some() {
            self.len -= 1;
        }
        prev
    }

    /// Removes all values. Retains the mmap allocation.
    ///
    /// On Linux, uses `madvise(MADV_DONTNEED)` to discard pages
    /// immediately. On other platforms, re-mmaps over the region with
    /// `MAP_FIXED` to zero it.
    pub fn clear(&mut self) {
        let ptr = self.ptr.as_ptr() as *mut libc::c_void;
        let size = Self::alloc_size();
        if size > 0 && size < usize::MAX {
            unsafe {
                #[cfg(target_os = "linux")]
                libc::madvise(ptr, size, libc::MADV_DONTNEED);
                #[cfg(not(target_os = "linux"))]
                {
                    let ret = libc::mmap(
                        ptr,
                        size,
                        libc::PROT_READ | libc::PROT_WRITE,
                        libc::MAP_PRIVATE | libc::MAP_ANONYMOUS | libc::MAP_FIXED,
                        -1,
                        0,
                    );
                    assert!(
                        ret != libc::MAP_FAILED,
                        "CoordSpaceM: MAP_FIXED remap failed on clear"
                    );
                }
            }
        }
        self.len = 0;
    }
}

impl<const N: usize, V: Clone> Clone for CoordSpaceM<N, V> {
    fn clone(&self) -> Self {
        let size = Self::alloc_size();
        if size == 0 || size >= usize::MAX >> 1 {
            // Cannot clone an mmap region that exceeds reasonable bounds.
            // N=12 and N=19 saturate to usize::MAX; cloning them would
            // try to memcpy the entire virtual address space. For those
            // depths, cloning is not supported.
            panic!(
                "CoordSpaceM: clone not supported for N={} (size={})",
                N, size
            );
        }
        // Allocate a new mmap via raw mmap (not Self::new, because we
        // need the pointer without wrapping it in CoordSpaceM yet).
        let ptr = unsafe {
            libc::mmap(
                ptr::null_mut(),
                size,
                libc::PROT_READ | libc::PROT_WRITE,
                libc::MAP_PRIVATE | libc::MAP_ANONYMOUS | libc::MAP_NORESERVE,
                -1,
                0,
            )
        };
        if ptr == libc::MAP_FAILED {
            panic!(
                "CoordSpaceM: mmap failed during clone (N={}, size={})",
                N, size
            );
        }
        unsafe {
            libc::memcpy(ptr, self.ptr.as_ptr() as *const libc::c_void, size);
        }
        CoordSpaceM {
            ptr: NonNull::new(ptr as *mut Option<V>).unwrap(),
            len: self.len,
        }
    }
}

impl<const N: usize, V> Drop for CoordSpaceM<N, V> {
    fn drop(&mut self) {
        let size = Self::alloc_size();
        if size > 0 && size < usize::MAX {
            unsafe {
                libc::munmap(self.ptr.as_ptr() as *mut libc::c_void, size);
            }
        }
    }
}

impl<const N: usize, V: PartialEq> PartialEq for CoordSpaceM<N, V> {
    fn eq(&self, other: &Self) -> bool {
        if self.len != other.len {
            return false;
        }
        if self.len == 0 {
            return true;
        }
        // Compare occupied slots by walking the slot array.
        // This is correct but O(N) in slot count. For CoordSpaceM,
        // equality checks are typically used in tests with few entries.
        let size = Self::alloc_size();
        if size >= usize::MAX >> 1 {
            return self.len == other.len;
        }
        let n = size / core::mem::size_of::<Option<V>>();
        let a = unsafe { core::slice::from_raw_parts(self.ptr.as_ptr(), n) };
        let b = unsafe { core::slice::from_raw_parts(other.ptr.as_ptr(), n) };
        a == b
    }
}
impl<const N: usize, V: PartialEq> Eq for CoordSpaceM<N, V> {}

impl<const N: usize, V> Default for CoordSpaceM<N, V> {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// Type alias
// ---------------------------------------------------------------------------

/// 3-character: mmap-backed dense, true Tagma at N=3 scale.
///
/// This is the only supported depth for `CoordSpaceM`. At N>=6 the slot
/// count $11172^N$ exceeds `usize` on 64-bit platforms, making dense
/// linear addressing impossible. Use `CoordSpaceN[N]` (sparse tree) for
/// depths >= 4.
pub type CoordSpaceM3<V> = CoordSpaceM<3, V>;

// The following depths are intentionally omitted:
//   CoordSpaceM6  — 11172^6 overflows usize (1.94e24 > u64::MAX)
//   CoordSpaceM12 — 11172^12 overflows usize
//   CoordSpaceM19 — 11172^19 overflows usize
// For these depths, the CoordSpace family provides tree-based fallbacks
// (CoordSpaceN6, CoordSpaceN12, CoordSpaceN19) via the alloc feature.

// ---------------------------------------------------------------------------
// Helper
// ---------------------------------------------------------------------------

/// Linear index into the flat mmap array for a CoordPath of depth N.
/// Delegates to the shared implementation in coord_space_dense.
#[inline]
fn linear_index<const N: usize>(path: &CoordPath<N>) -> usize {
    crate::coord_space_dense::linear_index::<N>(path)
}
