//! A coordinate path read as a name.
//!
//! `CoordPath` is an index into a `CoordSpace`; `CoordId` is the same sequence
//! read as an identity. It prints one compositional character an axis, parses
//! back from that text, orders as the coordinates do, and hashes over them.
//! It mints nothing: a layer that derives ids from content adds that itself.

use core::cmp::Ordering;
use core::fmt;
use core::hash::{Hash, Hasher};

use crate::coord::Coord;
use crate::coord_path::CoordPath;

/// A coordinate path used as an identity.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CoordId<const N: usize>(pub CoordPath<N>);

impl<const N: usize> CoordId<N> {
    /// Reads the canonical text: exactly `N` compositional characters.
    ///
    /// A wrong length or a character outside the coordinate space is `None`.
    /// The text names one sequence only, so the other cases are refused rather
    /// than folded.
    pub fn from_string(text: &str) -> Option<Self> {
        let mut coords = [Coord::new(0)?; N];
        let mut len = 0;
        for ch in text.chars() {
            if len == N {
                return None;
            }
            coords[len] = Coord::from_char(ch)?;
            len += 1;
        }
        if len == N {
            Some(CoordId(CoordPath::new(coords)))
        } else {
            None
        }
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
            write!(f, "{}", coord.to_char())?;
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
        let mut text = alloc::string::String::with_capacity(N * 3);
        for coord in self.0.iter() {
            text.push(coord.to_char());
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
                write!(f, "a canonical coordinate of {N} characters")
            }

            fn visit_str<E>(self, text: &str) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                CoordId::from_string(text).ok_or_else(|| {
                    E::custom("not a canonical coordinate: wrong length or an invalid character")
                })
            }
        }

        deserializer.deserialize_str(Canonical::<N>)
    }
}
