#![no_std]
extern crate alloc;

use alloc::vec::Vec;

pub mod coord_cube_map;
pub mod coord_gen;
pub mod coord_map;
pub mod coord_map2;
pub mod coord_map_n;
pub mod dyn_coord_map;

use tagma_core::Coord;

// Re-exports from the coord_gen module.
pub use coord_gen::{
    ByteFold, ByteWise, CharWise, CoordGen, CoordKey, DefaultDynamic, GenError, Prefix,
};

// Re-exports from the coord_map module (traits).
pub use coord_map::{CoordMap, CoordMapKey};

// Re-exports from concrete map modules.
pub use coord_map2::CoordMap2;
pub use coord_map_n::CoordMapN;
pub use dyn_coord_map::DynCoordMap;

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
