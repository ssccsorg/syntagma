//! A coordinate path read as a name.
//!
//! `CoordPath` is an index into a `CoordSpace`; `CoordId` is the same sequence
//! read as an identity. The text is fixed-width base-36, three digits an axis,
//! so an identity is ASCII at every depth and no byte of it depends on a
//! normalization form.
//!
//! The coordinate space is unchanged: a value is still one of the 11,172 the
//! syllable block admits, and the structure that number comes from is the
//! reason an identity is order-preserving and injective. Only the spelling of
//! that value left the block.
//!
//! Measured at depth 20 on the ARMv8.4-A reference machine: value to text
//! 55 ns, text to value 35 ns, against 203 ns and 24 ns for the syllable
//! spelling. The render fell and the parse rose. A layer that holds the value
//! pays neither, which is why the text belongs at a boundary and the value
//! belongs in the container.

use core::cmp::Ordering;
use core::fmt;
use core::hash::{Hash, Hasher};

#[cfg(feature = "serde")]
use alloc::format;

use crate::coord::Coord;
use crate::coord_path::CoordPath;

/// Characters one axis value occupies in the text.
///
/// A value is below 11,172, which three base-36 digits carry (46,656), so the
/// text is `N * 3` bytes and the axis count decides the length.
pub const DIGITS_PER_AXIS: usize = 3;

/// The alphabet in ascending code point order, which is also the digit order.
///
/// Ascending order is what makes sorting the text sort the coordinate, and the
/// fixed width is what makes a prefix of the text a prefix of the axes. Lowercase
/// is the form a base-36 conversion already emits and the case the hex beside a
/// key is written in. The other case is not accepted on the way back, because two
/// spellings of one value would cost both properties.
pub const ALPHABET: &[u8; 36] = b"0123456789abcdefghijklmnopqrstuvwxyz";

/// The number of values one digit carries, taken from the alphabet so the
/// arithmetic cannot drift from the characters it writes.
const RADIX: u32 = ALPHABET.len() as u32;

// The notation is only well defined while these hold, so they are checked where
// they can fail to compile rather than asserted in prose.
const _: () = {
    assert!(ALPHABET.len() == 36, "the alphabet is base 36");
    assert!(
        36 * 36 * 36 > Coord::N_VALID,
        "three digits must carry every axis value"
    );
    assert!(
        36 * 36 <= Coord::N_VALID,
        "two digits must not, or the fixed width would be wrong"
    );
};

/// The value a digit stands for, or `None` if the byte is not one.
#[inline]
const fn digit(byte: u8) -> Option<u32> {
    match byte {
        b'0'..=b'9' => Some((byte - b'0') as u32),
        b'a'..=b'z' => Some((byte - b'a' + 10) as u32),
        _ => None,
    }
}

/// A coordinate path used as an identity.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CoordId<const N: usize>(pub CoordPath<N>);

impl<const N: usize> CoordId<N> {
    /// Reads the canonical text: exactly `N * 3` ASCII digits.
    ///
    /// A wrong length, a byte outside the alphabet, or an axis value above the
    /// coordinate space is `None`. The text names one sequence only, so the
    /// other cases are refused rather than folded.
    pub fn from_string(text: &str) -> Option<Self> {
        let bytes = text.as_bytes();
        if bytes.len() != N * DIGITS_PER_AXIS {
            return None;
        }
        let mut coords = [Coord::new(0)?; N];
        for (axis, chunk) in bytes.as_chunks::<DIGITS_PER_AXIS>().0.iter().enumerate() {
            let mut value: u32 = 0;
            for &byte in chunk {
                value = value * RADIX + digit(byte)?;
            }
            coords[axis] = Coord::new(value as u16)?;
        }
        Some(CoordId(CoordPath::new(coords)))
    }

    /// The path this id names.
    #[inline]
    pub const fn path(&self) -> &CoordPath<N> {
        &self.0
    }

    /// The coordinates in path order.
    #[inline]
    pub const fn coords(&self) -> &[Coord; N] {
        self.0.coords()
    }

    /// Iterates the coordinates in path order.
    #[inline]
    pub fn iter(&self) -> core::slice::Iter<'_, Coord> {
        self.0.iter()
    }
}

impl<const N: usize> fmt::Display for CoordId<N> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for coord in self.0.iter() {
            let value = coord.index() as u32;
            let high = value / (RADIX * RADIX);
            let rest = value % (RADIX * RADIX);
            let digits = [
                ALPHABET[high as usize],
                ALPHABET[(rest / RADIX) as usize],
                ALPHABET[(rest % RADIX) as usize],
            ];
            // The alphabet is ASCII, so the three bytes are always valid text.
            let text = core::str::from_utf8(&digits).map_err(|_| fmt::Error)?;
            f.write_str(text)?;
        }
        Ok(())
    }
}

impl<const N: usize> Hash for CoordId<N> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        for coord in self.0.iter() {
            coord.index().hash(state);
        }
    }
}

impl<const N: usize> PartialOrd for CoordId<N> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<const N: usize> Ord for CoordId<N> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.0.coords().cmp(other.0.coords())
    }
}

#[cfg(feature = "serde")]
impl<const N: usize> serde::Serialize for CoordId<N> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut text = alloc::string::String::with_capacity(N * DIGITS_PER_AXIS);
        for coord in self.0.iter() {
            let value = coord.index() as u32;
            let high = value / (RADIX * RADIX);
            let rest = value % (RADIX * RADIX);
            text.push(ALPHABET[high as usize] as char);
            text.push(ALPHABET[(rest / RADIX) as usize] as char);
            text.push(ALPHABET[(rest % RADIX) as usize] as char);
        }
        serializer.serialize_str(&text)
    }
}

#[cfg(feature = "serde")]
impl<'de, const N: usize> serde::Deserialize<'de> for CoordId<N> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct Canonical<const N: usize>;

        impl<'de, const N: usize> serde::de::Visitor<'de> for Canonical<N> {
            type Value = CoordId<N>;

            fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(
                    f,
                    "a canonical coordinate of {} characters",
                    N * DIGITS_PER_AXIS
                )
            }

            fn visit_str<E>(self, text: &str) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                CoordId::from_string(text).ok_or_else(|| {
                    E::custom(format!(
                        "expected {} base-36 digits, got {:?}",
                        N * DIGITS_PER_AXIS,
                        text
                    ))
                })
            }
        }

        deserializer.deserialize_str(Canonical::<N>)
    }
}
