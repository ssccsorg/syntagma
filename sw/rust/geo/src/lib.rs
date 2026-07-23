//! # tagma-geo: Spatial operations for Tagma
//!
//! Provides higher-level spatial query primitives built on
//! [`CoordCube`](tagma_core::CoordCube):
//!
//! - **Bounding box** — enumerate all paths within a hyper-rectangle
//! - **Proximity** — enumerate all paths within an L∞ (Chebyshev) radius
//! - **Hamming filtering** — constrain proximity to a Hamming distance
//!
//! This crate depends only on [`tagma-core`] and does **not** modify or
//! replace any existing storage primitives.  It is an optional, additive
//! layer for applications that need spatial reasoning over `CoordPath`
//! keys.
//!
//! # Relationship to the storage layer
//!
//! `tagma-geo` generates `CoordPath` values for spatial queries.  The
//! actual lookup of stored values is left to `tagma-kv` extension
//! methods or client code.
//!
//! # Example
//!
//! ```rust
//! use tagma_core::{Coord, CoordPath, CoordCube};
//! use tagma_geo::spatial::SpatialOps;
//!
//! // 2 dimensions, 1 syllable each → N=2
//! let path = CoordPath::<2>::new([Coord::new(5).unwrap(), Coord::new(5).unwrap()]);
//! let cube = CoordCube::<2, 2, 1>::from_path(path);
//!
//! // Find all paths within L∞ radius 1
//! let nearby: Vec<_> = cube.proximity(1).collect();
//! assert_eq!(nearby.len(), 9);
//! ```

pub mod spatial;

pub use spatial::BoundingBoxIter;
pub use spatial::HammingFilter;
pub use spatial::SpatialOps;
