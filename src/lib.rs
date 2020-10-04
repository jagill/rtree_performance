#![feature(total_cmp)]
mod coordinate;
pub mod from_wkt;
mod hilbert;
mod packed_rtree;
mod packed_rtree_auto_simd;
mod packed_rtree_unsorted;
mod rectangle;
mod rtree;
pub mod utils;

pub use coordinate::Coordinate;
pub use packed_rtree::PackedRTree;
pub use packed_rtree_auto_simd::PackedRTreeAutoSimd;
pub use packed_rtree_unsorted::PackedRTreeUnsorted;
pub use rectangle::{HasEnvelope, Rectangle};
pub use rtree::RTree;

#[cfg(test)]
mod tests;
