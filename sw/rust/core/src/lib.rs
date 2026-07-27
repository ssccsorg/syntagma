#![no_std]

#[cfg(feature = "alloc")]
extern crate alloc;

pub mod coord;
pub mod coord_cube;
pub mod coord_path;
pub mod coord_set;
pub mod coord_space;

// CoordSpaceN: multi-character, heap-backed tree (N>1).
// Requires alloc feature (default: on).
#[cfg(feature = "alloc")]
pub mod coord_space_n;

// DynCoordSpace: dynamic depth, heap-backed trie.
#[cfg(feature = "alloc")]
pub mod dyn_coord_space;

// CoordSetN: sparse N-dimensional set (N>1).
#[cfg(feature = "alloc")]
pub mod coord_set_n;

// CoordSpaceDense: dense zeroed array family (true Tagma for any N).
#[cfg(feature = "alloc")]
pub mod coord_space_dense;

// CoordSpaceM: mmap-backed dense array family (N >= 3).
#[cfg(feature = "mmap")]
pub mod coord_space_m;

pub use coord::Coord;
pub use coord_cube::CoordCube;
pub use coord_path::CoordPath;
pub use coord_set::CoordSet;

// ── CoordSpace series — unified naming ──

/// 1-character: 11,172 identifiers. No allocator required.
pub use coord_space::CoordSpace;

#[cfg(feature = "alloc")]
pub use coord_space_n::CoordSpaceN;
#[cfg(feature = "alloc")]
pub use coord_space_n::CoordSpaceN12;
#[cfg(feature = "alloc")]
pub use coord_space_n::CoordSpaceN19;
#[cfg(feature = "alloc")]
pub use coord_space_n::CoordSpaceN2;
#[cfg(feature = "alloc")]
pub use coord_space_n::CoordSpaceN3;
#[cfg(feature = "alloc")]
pub use coord_space_n::CoordSpaceN6;
#[cfg(feature = "alloc")]
pub use dyn_coord_space::DynCoordSpace;
#[cfg(feature = "alloc")]
pub use dyn_coord_space::DynIter;

#[cfg(feature = "alloc")]
pub use coord_set_n::CoordSetN;

#[cfg(feature = "alloc")]
pub use coord_space_dense::CoordSpace2;

#[cfg(feature = "mmap")]
pub use coord_space_m::CoordSpaceM;
#[cfg(feature = "mmap")]
pub use coord_space_m::CoordSpaceM3;

// Internal types (used by CoordSpace):
pub use coord_space::FlatDrain;
pub use coord_space::FlatEntry;
pub use coord_space::FlatIter;
pub use coord_space::FlatIterMut;
