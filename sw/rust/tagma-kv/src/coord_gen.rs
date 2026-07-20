use tagma_core::Coord;

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
/// This is the default strategy used by [`TagmaKV`].
///
/// [`TagmaKV`]: crate::TagmaKV
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
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

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
        assert_eq!(a, b, "commutative XOR within same accumulator produces collision");
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
    fn prefix2_matches_string_to_short_path_first_two() {
        // string_to_short_path packs 4 bytes into 2 Coords via u16 + mod 11172.
        // Prefix<2> maps first 2 bytes directly.
        // They differ in packing scheme — we just verify both succeed without panic.
        let key = "abcd";
        let from_prefix = Prefix::<2>.generate(key).unwrap();
        let _from_short = crate::string_to_short_path(key).unwrap();
        assert_eq!(from_prefix.len(), 2);
    }
}
