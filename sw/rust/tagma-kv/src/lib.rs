pub mod coord_gen;
pub mod coord_kv;
pub mod coord_kv2;
pub mod coord_kv_n;
pub mod dyn_coord_kv;

use tagma_core::Coord;

// Re-exports from the coord_gen module.
pub use coord_gen::{
    ByteFold, ByteWise, CharWise, CoordGen, CoordKey, DefaultDynamic, GenError, Prefix,
};

// Re-exports from the coord_kv module (traits).
pub use coord_kv::{CoordKV, CoordKVKey};

// Re-exports from concrete KV modules.
pub use coord_kv2::CoordKV2;
pub use coord_kv_n::CoordKVN;
pub use dyn_coord_kv::DynCoordKV;

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
