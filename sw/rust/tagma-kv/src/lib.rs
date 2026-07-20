pub mod coord_gen;

use tagma_core::{Coord, CoordPath, CoordSpace2, DynCoordSpace};

// Re-exports from the coord_gen module.
pub use coord_gen::{ByteFold, ByteWise, CharWise, CoordGen, FixedKey, GenError, Prefix, DefaultDynamic};

// ---------------------------------------------------------------------------
// String → CoordPath conversion (zero hash, zero collision)
// ---------------------------------------------------------------------------

/// Converts a string key to a `Coord` vector by mapping each UTF-8 byte
/// directly to one `Coord`. Since byte values (0..255) are always within
/// the valid Coord range (0..11172), this mapping is injective and
/// collision-free. No hash function is used.
///
/// Delegates to [`ByteWise`].
///
/// Returns `None` for empty strings.
pub fn string_to_coord_path(s: &str) -> Option<Vec<Coord>> {
    ByteWise.generate(s).ok()
}

/// Packs a short string (≤4 bytes) into a `CoordPath<2>` for accelerated
/// lookup in CoordSpace2. Each Coord stores 2 bytes as a u16.
///
/// Returns `None` if the string exceeds 4 bytes.
pub fn string_to_short_path(s: &str) -> Option<CoordPath<2>> {
    let bytes = s.as_bytes();
    if bytes.len() > 4 {
        return None;
    }
    let mut buf = [0u8; 4];
    buf[..bytes.len()].copy_from_slice(bytes);
    let v0 = u16::from_le_bytes([buf[0], buf[1]]);
    let v1 = u16::from_le_bytes([buf[2], buf[3]]);
    Some(CoordPath::new([
        Coord::new(v0 % 11172).expect("short path coord 0"),
        Coord::new(v1 % 11172).expect("short path coord 1"),
    ]))
}

// ---------------------------------------------------------------------------
// Wrapper:  Box<[u8]> vs Vec<u8>
// ---------------------------------------------------------------------------
// CoordSpace and CoordSpace2 use alloc_zeroed, which requires the None
// discriminant to be the all-zero bit pattern.  Option<Vec<u8>> uses
// 0x8000... as its None niche (the pointer's high bit), making it
// incompatible with zeroed memory.  Option<Box<[u8]>> uses the null
// pointer (all zeros) as None, which IS compatible.  We store values
// internally as Box<[u8]> and convert to/from Vec<u8> at the API.

fn vec_to_box(v: Vec<u8>) -> Box<[u8]> {
    v.into_boxed_slice()
}

fn box_to_vec(v: &[u8]) -> Vec<u8> {
    v.to_vec()
}

fn box_to_vec_owned(v: Box<[u8]>) -> Vec<u8> {
    v.into_vec()
}

// ---------------------------------------------------------------------------
// DynCoordSpace uses heap-allocated nodes (no alloc_zeroed), so Vec<u8>
// works fine internally.  But for consistency with the short path we use
// Box<[u8]> everywhere.
// ---------------------------------------------------------------------------

/// A hash-free, collision-free string key-value store backed by Tagma's
/// coordinate space.
///
/// Two storage tiers:
/// - **Default**: variable-length byte-wise mapping into `DynCoordSpace`.
///   Supports any non-empty string key. Lookup cost is O(len(key)).
/// - **Short**: fixed 4-byte packing into `CoordSpace2` (119 MB, ~0.39 ns).
///   For keys ≤4 bytes only.
///
/// No hash functions are used anywhere in this crate.
pub struct CoordKV {
    dyn_space: DynCoordSpace<Box<[u8]>>,
    short_space: CoordSpace2<Box<[u8]>>,
    len: usize,
}

impl CoordKV {
    /// Creates an empty store.
    pub fn new() -> Self {
        CoordKV {
            dyn_space: DynCoordSpace::new(),
            short_space: CoordSpace2::new(),
            len: 0,
        }
    }

    /// Returns the number of stored entries.
    pub fn len(&self) -> usize {
        self.len
    }

    /// Returns `true` if the store is empty.
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Inserts a key-value pair.
    ///
    /// Empty string keys are ignored (cannot be represented as CoordPath).
    pub fn insert(&mut self, key: &str, value: Vec<u8>) {
        if key.is_empty() {
            return;
        }
        let boxed = vec_to_box(value.clone());
        let path = string_to_coord_path(key).unwrap();
        if self.dyn_space.place(&path, boxed).is_none() {
            self.len += 1;
        }
        if let Some(short_path) = string_to_short_path(key) {
            self.short_space.place_path(&short_path, vec_to_box(value));
        }
    }

    /// Retrieves a value by key. Uses the injective byte-wise path
    /// (collision-free).
    ///
    /// The short path (`get_short`) is an additional optimization for
    /// ≤4-byte keys that may have modulo collisions and is not checked
    /// by `get` — correctness always uses the injective path.
    pub fn get(&self, key: &str) -> Option<Vec<u8>> {
        if key.is_empty() {
            return None;
        }
        string_to_coord_path(key)
            .and_then(|path| self.dyn_space.at(&path))
            .map(|v| box_to_vec(v.as_ref()))
    }

    /// Removes a key-value pair. Returns the value if present.
    pub fn remove(&mut self, key: &str) -> Option<Vec<u8>> {
        if key.is_empty() {
            return None;
        }
        let removed_short = string_to_short_path(key)
            .and_then(|p| self.short_space.vacate_path(&p))
            .map(box_to_vec_owned);
        let removed_dyn = string_to_coord_path(key)
            .and_then(|path| self.dyn_space.vacate(&path))
            .map(box_to_vec_owned);
        if removed_short.is_some() || removed_dyn.is_some() {
            self.len = self.len.saturating_sub(1);
        }
        removed_short.or(removed_dyn)
    }

    /// Short-key optimized insert (≤4 bytes). Returns error if key is too long.
    pub fn insert_short(&mut self, key: &str, value: Vec<u8>) -> Result<(), &'static str> {
        if key.len() > 4 {
            return Err("tagma-kv: key exceeds 4 bytes for short insert");
        }
        if !key.is_empty() {
            let dyn_path = string_to_coord_path(key).unwrap();
            self.dyn_space.place(&dyn_path, vec_to_box(value.clone()));
        }
        let path = string_to_short_path(key).unwrap();
        if self
            .short_space
            .place_path(&path, vec_to_box(value))
            .is_none()
        {
            self.len += 1;
        }
        Ok(())
    }

    /// Short-key optimized get (≤4 bytes). Returns None if key is too long
    /// or not found.
    pub fn get_short(&self, key: &str) -> Option<Vec<u8>> {
        if key.len() > 4 {
            return None;
        }
        string_to_short_path(key)
            .and_then(|p| self.short_space.at_path(&p))
            .map(|v| box_to_vec(v.as_ref()))
    }

    /// Removes all entries. Retains allocations.
    pub fn clear(&mut self) {
        self.dyn_space.clear();
        self.short_space.clear();
        self.len = 0;
    }
}

impl Default for CoordKV {
    fn default() -> Self {
        Self::new()
    }
}
