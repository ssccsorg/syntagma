//! # tagma-geo: Spatial operations for Tagma
//!
//! Provides distance metrics and spatial query primitives built on
//! [`CoordCube`](tagma_core::CoordCube):
//!
//! - **Distance metrics** — Hamming, Euclidean (approx), Manhattan
//! - **Bounding box** — enumerate all paths within a hyper-rectangle
//! - **Proximity** — enumerate all paths within an L∞ (Chebyshev) radius
//! - **Hamming filtering** — constrain proximity to a Hamming distance
//!
//! This crate depends only on [`tagma-core`] and does **not** modify or
//! replace any existing storage primitives.

pub mod spatial;

pub use spatial::BoundingBoxIter;
pub use spatial::DistanceMetrics;
pub use spatial::HammingFilter;
pub use spatial::SpatialOps;
