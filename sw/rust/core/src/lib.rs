#![no_std]

#[cfg(feature = "alloc")]
extern crate alloc;

pub mod coord;
pub mod coord_path;
pub mod coord_set;
pub mod coord_space;

// CoordSpaceN: multi-syllable, heap-backed tree (N>1).
// Requires alloc feature (default: on).
#[cfg(feature = "alloc")]
pub mod coord_space_n;

// DynCoordSpace: dynamic depth, heap-backed trie.
#[cfg(feature = "alloc")]
pub mod dyn_coord_space;

pub use coord::Coord;
pub use coord_path::CoordPath;
pub use coord_set::CoordSet;

// ── CoordSpace series — unified naming ──

/// 1-syllable: 11,172 identifiers. No allocator required.
pub use coord_space::CoordSpace;

#[cfg(feature = "alloc")]
pub use coord_space_n::CoordSpace12;
#[cfg(feature = "alloc")]
pub use coord_space_n::CoordSpace19;
#[cfg(feature = "alloc")]
pub use coord_space_n::CoordSpace2;
#[cfg(feature = "alloc")]
pub use coord_space_n::CoordSpace3;
#[cfg(feature = "alloc")]
pub use coord_space_n::CoordSpace6;
#[cfg(feature = "alloc")]
pub use coord_space_n::CoordSpaceN;
#[cfg(feature = "alloc")]
pub use dyn_coord_space::DynCoordSpace;
#[cfg(feature = "alloc")]
pub use dyn_coord_space::DynIter;

// Internal types (used by CoordSpace1):
pub use coord_space::FlatDrain;
pub use coord_space::FlatEntry;
pub use coord_space::FlatIter;
pub use coord_space::FlatIterMut;
