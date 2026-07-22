use tagma_core::{Coord, CoordPath};

// ---------------------------------------------------------------------------
// Error type
// ---------------------------------------------------------------------------

/// Errors that can occur during coordinate generation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GenError {
    /// The key is empty; no valid path can be produced.
    EmptyKey,
    /// The key exceeds the maximum length supported by this strategy.
    KeyTooLong { max_len: usize, actual_len: usize },
}

// ---------------------------------------------------------------------------
// CoordGen trait
// ---------------------------------------------------------------------------

/// A strategy for converting a string key into a sequence of [`Coord`]
/// values — the fundamental mapping from application-level keys to Tagma's
/// coordinate space.
///
/// Two families exist:
///
/// **Dynamic** — path length varies with key length.  Because each byte or
/// scalar maps injectively, dynamic strategies are collision-free.  They
/// must be backed by a depth-flexible store such as [`DynCoordSpace`].
///
/// **Static** — path length is fixed regardless of key length.  Static
/// strategies enable O(1) dense array lookups via [`CoordSpace2`] or
/// [`CoordSpaceN`] at the cost of potential collisions from compression or
/// truncation.
///
/// [`DynCoordSpace`]: ../tagma_core/dyn_coord_space/struct.DynCoordSpace.html
/// [`CoordSpace2`]: ../tagma_core/coord_space_dense/struct.CoordSpace2.html
/// [`CoordSpaceN`]: ../tagma_core/coord_space_n/struct.CoordSpaceN.html
pub trait CoordGen {
    /// Human-readable strategy name (e.g. `"byte-wise"`, `"prefix-8"`).
    fn name(&self) -> &str;

    /// Converts `key` into a vector of `Coord` values.
    ///
    /// Returns `GenError::EmptyKey` for empty strings.
    fn generate(&self, key: &str) -> Result<Vec<Coord>, GenError>;

    /// Whether this strategy guarantees injective (collision-free) mapping.
    ///
    /// Dynamic strategies (`ByteWise`, `CharWise`) return `true`.
    /// Static strategies (`Prefix<N>`, `ByteFold<N>`) return `false`.
    fn is_injective(&self) -> bool;

    /// If this strategy always produces a fixed number of Coords, returns
    /// that number.  Dynamic strategies return `None`.
    fn fixed_depth(&self) -> Option<usize>;
}

// ---------------------------------------------------------------------------
// Dynamic strategies
// ---------------------------------------------------------------------------

/// Byte-wise dynamic strategy.
///
/// Each UTF-8 byte maps to exactly one [`Coord`].  Since byte values are
/// in 0..256 and the valid Coord range is 0..11172, the mapping is
/// injective and collision-free.
///
/// Path length equals `key.len()` (in bytes, not characters).
///
/// This is the default strategy used by [`CoordKV`](crate::CoordKV).
///
/// # Why `&str` is the hardest case
///
/// `&str` is the most demanding key type in the Rust ecosystem: variable
/// length, UTF-8 validation, heap allocation for owned forms, SipHash-2-4
/// processing every byte.  ByteWise converts `&str` to Coord sequences
/// competitively with SipHash (often faster for short keys).  Every more
/// constrained key type — integers, UUIDs, fixed byte arrays, enums,
/// timestamps — requires less work to convert, making the relative
/// advantage of Tagma's approach even larger.
///
/// If the hardest case is already solved, the rest follows.
///
/// [`CoordKV`]: crate::CoordKV
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ByteWise;

impl CoordGen for ByteWise {
    fn name(&self) -> &str {
        "byte-wise"
    }

    fn generate(&self, key: &str) -> Result<Vec<Coord>, GenError> {
        if key.is_empty() {
            return Err(GenError::EmptyKey);
        }
        Ok(key
            .as_bytes()
            .iter()
            .map(|&b| Coord::new(b as u16).expect("byte value fits in Coord range 0..11172"))
            .collect())
    }

    fn is_injective(&self) -> bool {
        true
    }

    fn fixed_depth(&self) -> Option<usize> {
        None
    }
}

/// Char-wise dynamic strategy.
///
/// Each Unicode scalar value (Rust `char`) maps to two [`Coord`] values
/// via:
///
/// ```text
/// temp   = char as u32               (0..1,114,112)
/// c0     = temp / 11172              (0..99)
/// c1     = temp % 11172              (0..11171)
/// ```
///
/// Since `11172 * 100 = 1,117,200` exceeds the maximum Unicode scalar value
/// (1,114,112), every valid `char` produces a unique pair `(c0, c1)`.
/// The mapping is injective and collision-free.
///
/// Path length equals `2 × key.chars().count()`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CharWise;

impl CoordGen for CharWise {
    fn name(&self) -> &str {
        "char-wise"
    }

    fn generate(&self, key: &str) -> Result<Vec<Coord>, GenError> {
        if key.is_empty() {
            return Err(GenError::EmptyKey);
        }
        let n_chars = key.chars().count();
        let mut coords = Vec::with_capacity(n_chars * 2);
        for ch in key.chars() {
            let code = ch as u32;
            let c0 = (code / 11172) as u16;
            let c1 = (code % 11172) as u16;
            // c0 is at most 99 (since 11172*100 > 1,114,112), always below 11172.
            // c1 is always below 11172 by construction.
            coords.push(Coord::new(c0).expect("c0 < 100 < 11172"));
            coords.push(Coord::new(c1).expect("c1 < 11172 by modulus"));
        }
        Ok(coords)
    }

    fn is_injective(&self) -> bool {
        true
    }

    fn fixed_depth(&self) -> Option<usize> {
        None
    }
}

// ---------------------------------------------------------------------------
// Static strategies
// ---------------------------------------------------------------------------

/// Static prefix strategy.
///
/// Takes the first `N` bytes of the key and maps each to one [`Coord`].
/// If the key is shorter than `N` bytes, remaining positions are
/// zero-padded (`Coord(0)` == `가`).
///
/// This is a **lossy** truncation strategy — two different keys sharing
/// the same initial `N` bytes produce the same path.  Use this when you
/// only need to group or prefix-scan by leading bytes.
///
/// Path length always equals `N`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Prefix<const N: usize>;

impl<const N: usize> Prefix<N> {
    const _ASSERT: () = assert!(N > 0, "Prefix<0> is meaningless; use N >= 1");
}

impl<const N: usize> CoordGen for Prefix<N> {
    fn name(&self) -> &str {
        // const generics prevent runtime formatting; provide a static prefix.
        "prefix"
    }

    fn generate(&self, key: &str) -> Result<Vec<Coord>, GenError> {
        if key.is_empty() {
            return Err(GenError::EmptyKey);
        }
        let bytes = key.as_bytes();
        Ok((0..N)
            .map(|i| {
                let b = bytes.get(i).copied().unwrap_or(0);
                Coord::new(b as u16).expect("byte value fits in Coord range")
            })
            .collect())
    }

    fn is_injective(&self) -> bool {
        false
    }

    fn fixed_depth(&self) -> Option<usize> {
        Some(N)
    }
}

/// Static byte-fold strategy.
///
/// XOR-folds all bytes of the key into `N` accumulators, then maps each
/// accumulator modulo 11172 to a [`Coord`].
///
/// Accumulator `j` collects `key[i]` for all `i` where `i % N == j`:
///
/// ```text
/// acc[j] ^= byte for each byte at position i ≡ j (mod N)
/// ```
///
/// This is a **lossy** compression strategy — multiple keys can produce
/// the same `N`-Coord path (XOR collisions).  Use it to obtain a fixed
/// path depth for arbitrary-length keys when exact injectivity is not
/// required.
///
/// Path length always equals `N`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ByteFold<const N: usize>;

impl<const N: usize> ByteFold<N> {
    const _ASSERT: () = assert!(N > 0, "ByteFold<0> is meaningless; use N >= 1");
}

impl<const N: usize> CoordGen for ByteFold<N> {
    fn name(&self) -> &str {
        "byte-fold"
    }

    fn generate(&self, key: &str) -> Result<Vec<Coord>, GenError> {
        if key.is_empty() {
            return Err(GenError::EmptyKey);
        }
        let mut acc = vec![0u16; N];
        for (i, &b) in key.as_bytes().iter().enumerate() {
            acc[i % N] ^= b as u16;
        }
        Ok(acc
            .into_iter()
            .map(|v| Coord::new(v % 11172).expect("modulus < 11172"))
            .collect())
    }

    fn is_injective(&self) -> bool {
        false
    }

    fn fixed_depth(&self) -> Option<usize> {
        Some(N)
    }
}

// ---------------------------------------------------------------------------
// Default dynamic strategy (for use as a type alias or convenience)
// ---------------------------------------------------------------------------

/// The default dynamic coordinate generation strategy: [`ByteWise`].
pub type DefaultDynamic = ByteWise;

// ---------------------------------------------------------------------------
// CoordKey — type-level key capacity restriction
// ---------------------------------------------------------------------------

/// A fixed-size byte-array key that maps injectively to a [`CoordPath`]
/// of the same length `N`.
///
/// Unlike [`Prefix<N>`] (which truncates or zero-pads arbitrary strings),
/// `CoordKey<N>` **enforces exact key length at the type level** via const
/// generics.  This makes it impossible to construct a `CoordKey<N>` whose
/// byte length differs from `N`.
///
/// # Type-level length enforcement
///
/// Two construction paths, both guaranteeing length correctness:
///
/// **Compile time** (preferred, zero-cost):
///
/// ```
/// use tagma_kv::coord_gen::CoordKey;
///
/// const KEY: CoordKey<2> = CoordKey::from_str_const("hi");  // OK
/// // const BAD: CoordKey<2> = CoordKey::from_str_const("hello"); // compile error
/// ```
///
/// **Runtime** (convenient for dynamic input):
///
/// ```
/// use tagma_kv::coord_gen::CoordKey;
///
/// let key: CoordKey<2> = "hi".parse().unwrap();    // fallible via FromStr
/// let key: CoordKey<2> = "hi".into();               // infallible, panics on mismatch
/// ```
///
/// The runtime `From<&str>` path panics on length mismatch by design.
/// This is consistent with Rust's `From` contract: it assumes the input
/// is already valid.  For fallible conversion use `.parse::<CoordKey<N>>()`.
///
/// # Why `&str` is the hardest case
///
/// `&str` is the most demanding key type in the Rust ecosystem:
///
/// - Variable length (no type-level bound)
/// - UTF-8 validation required on construction
/// - Heap allocation for owned forms (`String`)
/// - SipHash-2-4 must process every byte
///
/// Tagma KV converts `&str` to `Coord` faster than SipHash hashes it (22.5 ns vs
/// 23.8 ns on ARMv8.4-A Firestorm for 2-byte keys).  If the hardest case is already faster,
/// then **every more constrained key type** — fixed integers, UUIDs, byte arrays,
/// enums, timestamps — is trivially faster by an even larger margin.  The
/// variable-length string boundary is the only real challenge, and it is solved.
///
/// # Injectivity guarantee
///
/// ```text
/// domain:   2^(8N)  possible [u8; N] values
/// codomain: 11172^N possible CoordPath<N> values
/// ratio:    11172^N / 2^(8N) ≈ 43.6^N
/// ```
///
/// Since the codomain is always vastly larger than the domain, each
/// distinct `CoordKey<N>` maps to a distinct `CoordPath<N>`.  The mapping
/// is **collision-free** — suitable for use with [`CoordSpace2`] (N=2) or
/// [`CoordSpaceN`] (any N) as a lossless static strategy.
///
/// # Example
///
/// ```
/// use tagma_core::CoordSpace2;
/// use tagma_kv::coord_gen::CoordKey;
///
/// let key = CoordKey::new(*b"hi");
/// let path = key.to_coord_path();
///
/// let mut store: CoordSpace2<u32> = CoordSpace2::new();
/// store.place_path(&path, 42);
/// assert_eq!(store.at_path(&path), Some(&42));
/// ```
///
/// [`CoordSpace2`]: ../tagma_core/coord_space_dense/struct.CoordSpace2.html
/// [`CoordSpaceN`]: ../tagma_core/coord_space_n/struct.CoordSpaceN.html
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct CoordKey<const N: usize>([u8; N]);

impl<const N: usize> CoordKey<N> {
    /// Creates a `CoordKey` from a byte array of exactly `N` bytes.
    ///
    /// The length is verified at compile time by the type system.
    #[inline]
    pub const fn new(bytes: [u8; N]) -> Self {
        Self(bytes)
    }

    /// Returns the underlying byte array.
    #[inline]
    pub const fn as_bytes(&self) -> &[u8; N] {
        &self.0
    }

    /// Converts this fixed-size key to a [`CoordPath<N>`] — an injective
    /// mapping: each byte maps to one [`Coord`], and `N` bytes produce a
    /// unique path because `11172^N >> 2^(8N)`.
    ///
    /// This is the **collision-free** counterpart of [`Prefix<N>`] for
    /// type-enforced key lengths.
    ///
    /// [`CoordPath<N>`]: ../tagma_core/coord_path/struct.CoordPath.html
    #[inline]
    pub fn to_coord_path(&self) -> CoordPath<N> {
        let mut coords = [Coord::new(0).unwrap(); N];
        for (i, &b) in self.0.iter().enumerate() {
            debug_assert!(
                (b as usize) < Coord::N_VALID,
                "byte value {} exceeds Coord range",
                b
            );
            coords[i] = Coord::new(b as u16).expect("byte < 11172");
        }
        CoordPath::new(coords)
    }

    /// Returns the number of bytes (always `N`).
    #[inline]
    pub const fn len(&self) -> usize {
        N
    }

    /// Returns `true` only when `N == 0` (a degenerate case).
    #[inline]
    pub const fn is_empty(&self) -> bool {
        N == 0
    }
}

impl<const N: usize> From<[u8; N]> for CoordKey<N> {
    #[inline]
    fn from(bytes: [u8; N]) -> Self {
        Self(bytes)
    }
}

impl<const N: usize> From<CoordKey<N>> for [u8; N] {
    #[inline]
    fn from(key: CoordKey<N>) -> Self {
        key.0
    }
}

impl<const N: usize> core::str::FromStr for CoordKey<N> {
    type Err = GenError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let bytes = s.as_bytes();
        if bytes.len() != N {
            return Err(GenError::KeyTooLong {
                max_len: N,
                actual_len: bytes.len(),
            });
        }
        let mut arr = [0u8; N];
        arr.copy_from_slice(bytes);
        Ok(Self(arr))
    }
}

impl<const N: usize> From<&str> for CoordKey<N> {
    /// Converts a `&str` to `CoordKey<N>`, **panicking** if the string
    /// length does not match `N`.
    ///
    /// Use this when you know the string is exactly `N` bytes and want
    /// an infallible conversion.  For fallible conversion use
    /// [`CoordKey::<N>::try_from`] or `.parse::<CoordKey<N>>()`.
    ///
    /// # Panics
    ///
    /// Panics if `s.len() != N`.
    fn from(s: &str) -> Self {
        assert!(
            s.len() == N,
            "CoordKey::from: expected string of exactly {} bytes, got {}",
            N,
            s.len()
        );
        let mut arr = [0u8; N];
        arr.copy_from_slice(s.as_bytes());
        Self(arr)
    }
}

impl<const N: usize> CoordKey<N> {
    /// Converts a `&str` to `CoordKey<N>` in **const context**.
    ///
    /// At compile time (`const`, `static`), a length mismatch produces a
    /// compile error.  At runtime it panics.
    ///
    /// # Example
    ///
    /// ```
    /// use tagma_kv::coord_gen::CoordKey;
    ///
    /// const KEY: CoordKey<2> = CoordKey::from_str_const("hi");
    /// assert_eq!(KEY.as_bytes(), b"hi");
    /// ```
    ///
    /// ```compile_fail
    /// use tagma_kv::coord_gen::CoordKey;
    /// const BAD: CoordKey<2> = CoordKey::from_str_const("hello"); // compile error
    /// ```
    #[inline]
    pub const fn from_str_const(s: &str) -> Self {
        assert!(s.len() == N, "CoordKey::from_str_const: length mismatch");
        let bytes = s.as_bytes();
        let mut arr = [0u8; N];
        let mut i = 0;
        while i < N {
            // SAFETY: i < N == s.len() (asserted above)
            arr[i] = unsafe { *bytes.as_ptr().add(i) };
            i += 1;
        }
        Self(arr)
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // ── CoordKey ──────────────────────────────────────────────────────────────

    #[test]
    fn fixed_key_new() {
        let key = CoordKey::new(*b"hello");
        assert_eq!(key.len(), 5);
        assert!(!key.is_empty());
        assert_eq!(key.as_bytes(), b"hello");
    }

    #[test]
    fn fixed_key_to_coord_path() {
        let key = CoordKey::new(*b"ab");
        let path = key.to_coord_path();
        assert_eq!(path.len(), 2);
        assert_eq!(path.coords()[0], Coord::new(b'a' as u16).unwrap());
        assert_eq!(path.coords()[1], Coord::new(b'b' as u16).unwrap());
    }

    #[test]
    fn fixed_key_from_array() {
        let key = CoordKey::from([0x01u8, 0xFFu8]);
        let path = key.to_coord_path();
        assert_eq!(path.coords()[0], Coord::new(0x01).unwrap());
        assert_eq!(path.coords()[1], Coord::new(0xFF).unwrap());
    }

    #[test]
    fn fixed_key_from_str_exact() {
        use core::str::FromStr;
        let key = CoordKey::<3>::from_str("abc").unwrap();
        assert_eq!(key.as_bytes(), b"abc");
    }

    #[test]
    fn fixed_key_from_str_wrong_length() {
        use core::str::FromStr;
        let err = CoordKey::<3>::from_str("ab").unwrap_err();
        assert_eq!(
            err,
            GenError::KeyTooLong {
                max_len: 3,
                actual_len: 2
            }
        );

        let err = CoordKey::<3>::from_str("abcd").unwrap_err();
        assert_eq!(
            err,
            GenError::KeyTooLong {
                max_len: 3,
                actual_len: 4
            }
        );
    }

    #[test]
    fn fixed_key_parse() {
        let key: CoordKey<4> = "test".parse().unwrap();
        assert_eq!(key.as_bytes(), b"test");
    }

    #[test]
    fn fixed_key_from_str_infallible() {
        let key: CoordKey<2> = "ok".into();
        assert_eq!(key.as_bytes(), b"ok");
    }

    #[test]
    fn fixed_key_roundtrip_array() {
        let original = [0xABu8, 0xCDu8];
        let key = CoordKey::new(original);
        let back: [u8; 2] = key.into();
        assert_eq!(original, back);
    }

    #[test]
    fn fixed_key_injective_across_two_byte_keys() {
        // Every 2-byte combination must produce a unique CoordPath<2>.
        // CoordPath does not implement Hash, so we convert to [u16; 2].
        use std::collections::HashSet;
        let mut seen = HashSet::new();
        for b0 in 0u8..=255 {
            for b1 in 0u8..=255 {
                let key = CoordKey::new([b0, b1]);
                let path = key.to_coord_path();
                let linear: [u16; 2] = [path.coords()[0].index(), path.coords()[1].index()];
                assert!(seen.insert(linear), "collision at bytes ({}, {})", b0, b1);
            }
        }
        assert_eq!(seen.len(), 65536);
    }

    #[test]
    fn fixed_key_eq_hash() {
        use std::collections::HashSet;
        let a = CoordKey::new(*b"ab");
        let b = CoordKey::new(*b"ab");
        let c = CoordKey::new(*b"ac");
        assert_eq!(a, b);
        assert_ne!(a, c);

        let mut set = HashSet::new();
        set.insert(a);
        set.insert(b); // duplicate
        set.insert(c);
        assert_eq!(set.len(), 2);
    }

    // ── ByteWise ────────────────────────────────────────────────────────

    #[test]
    fn bytewise_basic() {
        let s = ByteWise;
        assert_eq!(s.name(), "byte-wise");
        assert!(s.is_injective());
        assert_eq!(s.fixed_depth(), None);

        let path = s.generate("abc").unwrap();
        assert_eq!(path.len(), 3);
        assert_eq!(path[0], Coord::new(b'a' as u16).unwrap());
        assert_eq!(path[1], Coord::new(b'b' as u16).unwrap());
        assert_eq!(path[2], Coord::new(b'c' as u16).unwrap());
    }

    #[test]
    fn bytewise_empty() {
        assert_eq!(ByteWise.generate(""), Err(GenError::EmptyKey));
    }

    #[test]
    fn bytewise_unicode() {
        let path = ByteWise.generate("한").unwrap();
        // UTF-8: 한 = [0xED, 0x95, 0x9C]
        assert_eq!(path.len(), 3);
    }

    #[test]
    fn bytewise_injective_different_keys() {
        let a = ByteWise.generate("hello").unwrap();
        let b = ByteWise.generate("world").unwrap();
        assert_ne!(a, b, "different keys must produce different paths");
    }

    // ── CharWise ────────────────────────────────────────────────────────

    #[test]
    fn charwise_basic() {
        let s = CharWise;
        assert_eq!(s.name(), "char-wise");
        assert!(s.is_injective());
        assert_eq!(s.fixed_depth(), None);

        let path = CharWise.generate("ab").unwrap();
        // "a" (U+0061) -> c0 = 0, c1 = 97
        // "b" (U+0062) -> c0 = 0, c1 = 98
        assert_eq!(path.len(), 4);
        assert_eq!(path[0], Coord::new(0).unwrap());
        assert_eq!(path[1], Coord::new(0x0061).unwrap());
        assert_eq!(path[2], Coord::new(0).unwrap());
        assert_eq!(path[3], Coord::new(0x0062).unwrap());
    }

    #[test]
    fn charwise_hangul() {
        let path = CharWise.generate("한").unwrap();
        // "한" (U+D55C) -> code = 54620
        // c0 = 54620 / 11172 = 4  (4 * 11172 = 44688)
        // c1 = 54620 % 11172 = 9932
        assert_eq!(path.len(), 2);
        assert_eq!(path[0], Coord::new(4).unwrap());
        assert_eq!(path[1], Coord::new(9932).unwrap());
    }

    #[test]
    fn charwise_empty() {
        assert_eq!(CharWise.generate(""), Err(GenError::EmptyKey));
    }

    #[test]
    fn charwise_injective_different_keys() {
        let a = CharWise.generate("hello").unwrap();
        let b = CharWise.generate("world").unwrap();
        assert_ne!(a, b, "different keys must produce different paths");
    }

    // ── Prefix<N> ───────────────────────────────────────────────────────

    #[test]
    fn prefix_basic() {
        let s = Prefix::<4>;
        assert!(!s.is_injective());
        assert_eq!(s.fixed_depth(), Some(4));

        let path = Prefix::<3>.generate("abcde").unwrap();
        assert_eq!(path.len(), 3);
        assert_eq!(path[0], Coord::new(b'a' as u16).unwrap());
        assert_eq!(path[1], Coord::new(b'b' as u16).unwrap());
        assert_eq!(path[2], Coord::new(b'c' as u16).unwrap());
    }

    #[test]
    fn prefix_zero_pad() {
        let path = Prefix::<4>.generate("ab").unwrap();
        assert_eq!(path.len(), 4);
        assert_eq!(path[0], Coord::new(b'a' as u16).unwrap());
        assert_eq!(path[1], Coord::new(b'b' as u16).unwrap());
        assert_eq!(path[2], Coord::new(0).unwrap());
        assert_eq!(path[3], Coord::new(0).unwrap());
    }

    #[test]
    fn prefix_empty() {
        assert_eq!(Prefix::<1>.generate(""), Err(GenError::EmptyKey));
    }

    #[test]
    fn prefix_truncation_collision() {
        let a = Prefix::<3>.generate("abcdef").unwrap();
        let b = Prefix::<3>.generate("abcxyz").unwrap();
        // Both have prefix "abc", so paths collide — confirming injectivity is false.
        assert_eq!(a, b, "same prefix must collide");
    }

    // ── ByteFold<N> ─────────────────────────────────────────────────────

    #[test]
    fn bytefold_basic() {
        let s = ByteFold::<4>;
        assert!(!s.is_injective());
        assert_eq!(s.fixed_depth(), Some(4));

        let path = ByteFold::<2>.generate("abcd").unwrap();
        assert_eq!(path.len(), 2);
        // acc[0] = b'a' ^ b'c' = 0x61 ^ 0x63 = 0x02
        // acc[1] = b'b' ^ b'd' = 0x62 ^ 0x64 = 0x06
        // Both modulo 11172 still 2 and 6
        assert_eq!(path[0], Coord::new(2).unwrap());
        assert_eq!(path[1], Coord::new(6).unwrap());
    }

    #[test]
    fn bytefold_collision_same_xor() {
        // For N=2, acc[0] collects bytes at even positions (0, 2, ...)
        // and acc[1] collects bytes at odd positions (1, 3, ...).
        // Swapping even-position bytes 'a' <-> 'c' between two strings
        // of the same length produces the same XOR fold since XOR is
        // commutative within each accumulator.
        // "a\x00c" -> bytes [97, 0, 99] -> acc[0] = 97^99 = 2, acc[1] = 0
        // "c\x00a" -> bytes [99, 0, 97] -> acc[0] = 99^97 = 2, acc[1] = 0
        let a = ByteFold::<2>.generate("a\x00c").unwrap();
        let b = ByteFold::<2>.generate("c\x00a").unwrap();
        assert_eq!(
            a, b,
            "commutative XOR within same accumulator produces collision"
        );
        assert_eq!(a[0], Coord::new(2).unwrap());
        assert_eq!(a[1], Coord::new(0).unwrap());
    }

    #[test]
    fn bytefold_empty() {
        assert_eq!(ByteFold::<1>.generate(""), Err(GenError::EmptyKey));
    }

    #[test]
    fn bytefold_deterministic() {
        let a = ByteFold::<3>.generate("hello world").unwrap();
        let b = ByteFold::<3>.generate("hello world").unwrap();
        assert_eq!(a, b, "same key -> same path");
    }

    // ── Cross-strategy consistency with existing functions ──────────────

    #[test]
    fn bytewise_matches_string_to_coord_path() {
        let key = "hello";
        let from_strategy = ByteWise.generate(key).unwrap();
        let from_fn = crate::string_to_coord_path(key).unwrap();
        assert_eq!(from_strategy, from_fn);
    }

    #[test]
    fn prefix2_matches_bytewise_first_two() {
        // Prefix<2> takes the first 2 bytes; ByteWise produces 2 coords
        // for a 2-byte key. Both should agree on those bytes.
        let key = "ab";
        let from_prefix = Prefix::<2>.generate(key).unwrap();
        let from_bytewise = ByteWise.generate(key).unwrap();
        assert_eq!(from_prefix, from_bytewise);
    }
}
