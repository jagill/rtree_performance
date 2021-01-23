/**
 * A fast, low memory footprint static Rtree.
 *
 * This implementation is cache-oblivious and SIMD-friendly, but does not do
 * anything explicit to enable vectorizatino.
 */
use crate::utils::calculate_level_indices;
use crate::HasEnvelope;
use crate::{RTree, Rectangle};
use core::ops::Range;
use std::f64;

#[derive(Debug)]
#[repr(align(64))]
struct Bounds(Vec<f64>);
impl Bounds {
    pub fn new() -> Self {
        Bounds(Vec::new())
    }
    pub fn with_capacity(capacity: usize) -> Self {
        Bounds(Vec::with_capacity(capacity))
    }
    pub fn push(&mut self, x: f64) {
        self.0.push(x);
    }
    pub fn len(&self) -> usize {
        self.0.len()
    }
    pub fn extend_from_slice(&mut self, other: &[f64]) {
        self.0.extend_from_slice(other);
    }
}

#[derive(Debug)]
pub struct PackedRTreeUnsortedTracked {
    degree: usize,
    size: usize,
    // nodes in level i are (level_indices[i] .. level_indices[i + 1]) (end exclusive)
    level_indices: Vec<usize>,
    x_mins: Bounds,
    y_mins: Bounds,
    x_maxs: Bounds,
    y_maxs: Bounds,
}

impl RTree for PackedRTreeUnsortedTracked {
    fn is_empty(&self) -> bool {
        self.size == 0
    }

    fn height(&self) -> usize {
        self.level_indices.len()
    }

    fn degree(&self) -> usize {
        self.degree
    }

    fn envelope(&self) -> Rectangle {
        if self.is_empty() {
            Rectangle::new_empty()
        } else {
            let last_index = self.level_indices[self.height() - 1];
            Rectangle {
                x_min: self.x_mins.0[last_index],
                y_min: self.y_mins.0[last_index],
                x_max: self.x_maxs.0[last_index],
                y_max: self.y_maxs.0[last_index],
            }
        }
    }

    /**
     * Find geometries that might intersect the query_rect.
     *
     * This only checks bounding-box intersection, so the candidates must be
     * checked by the caller.
     */
    fn query_rect(&self, query: &Rectangle) -> Vec<usize> {
        let mut results = Vec::new();
        if self.is_empty() {
            return results;
        }

        let mut stack = Vec::new();
        if query.intersects(&self.envelope()) {
            stack.push(self.root());
        }

        // The todo_list will keep a LIFO stack of nodes to be processed.
        // The invariant is that everything in todo_list (envelope) intersects
        // query_rect.
        while let Some((level, offset)) = stack.pop() {
            if level == 0 {
                results.push(offset);
                continue;
            }

            let child_level = level - 1;
            let first_child_offset = self.degree * offset;
            let first_child_index = self.level_indices[child_level] + first_child_offset;
            let last_child_index = first_child_index + self.degree;
            let mut intersects = vec![true; self.degree];
            let mut contains = vec![true; self.degree];

            for (idx, &x_min) in self.x_mins.0[first_child_index..last_child_index]
                .iter()
                .enumerate()
            {
                intersects[idx] &= query.x_max >= x_min;
                contains[idx] &= query.x_min >= x_min;
            }

            for (idx, &y_min) in self.y_mins.0[first_child_index..last_child_index]
                .iter()
                .enumerate()
            {
                intersects[idx] &= query.y_max >= y_min;
                contains[idx] &= query.y_min >= y_min;
            }

            for (idx, &x_max) in self.x_maxs.0[first_child_index..last_child_index]
                .iter()
                .enumerate()
            {
                intersects[idx] &= query.x_min <= x_max;
                contains[idx] &= query.x_max <= x_max;
            }

            for (idx, &y_max) in self.y_maxs.0[first_child_index..last_child_index]
                .iter()
                .enumerate()
            {
                intersects[idx] &= query.y_min <= y_max;
                contains[idx] &= query.y_max <= y_max;
            }

            for (idx, (&intersect, contain)) in intersects.iter().zip(contains).enumerate() {
                let child_offset = first_child_offset + idx;
                if contain {
                    results.extend(self.get_leaf_range(child_level, child_offset))
                } else if intersect {
                    stack.push((child_level, child_offset));
                }
            }
        }

        results
    }
}

impl PackedRTreeUnsortedTracked {
    // pub fn leaves(&self) -> &[Rectangle] {
    //     let leaf_size = self.level_indices.len();
    //     if leaf_size == 0 {
    //         &[]
    //     } else if leaf_size == 1 {
    //         &self.tree[..1]
    //     } else {
    //         &self.tree[..self.level_indices[1]]
    //     }
    // }

    pub fn new_empty() -> Self {
        Self {
            degree: 2,
            size: 0,
            level_indices: Vec::new(),
            x_mins: Bounds::new(),
            y_mins: Bounds::new(),
            x_maxs: Bounds::new(),
            y_maxs: Bounds::new(),
        }
    }

    pub fn new<IR: HasEnvelope>(mut degree: usize, rects: &[IR]) -> Self {
        if rects.is_empty() {
            return Self::new_empty();
        }

        degree = degree.max(2);
        let size = rects.len();
        let level_indices = calculate_level_indices(degree, size);
        let tree_size = level_indices[level_indices.len() - 1] + 1;
        let mut x_mins = Bounds::with_capacity(tree_size);
        let mut y_mins = Bounds::with_capacity(tree_size);
        let mut x_maxs = Bounds::with_capacity(tree_size);
        let mut y_maxs = Bounds::with_capacity(tree_size);

        for rect in rects.iter().map(|r| r.envelope()) {
            x_mins.push(rect.x_min);
            y_mins.push(rect.y_min);
            x_maxs.push(rect.x_max);
            y_maxs.push(rect.y_max);
        }

        for level in 1..level_indices.len() {
            let level_index = level_indices[level];
            let padding = vec![f64::NAN; level_index - x_mins.len()];
            x_mins.extend_from_slice(&padding);
            y_mins.extend_from_slice(&padding);
            x_maxs.extend_from_slice(&padding);
            y_maxs.extend_from_slice(&padding);
            assert_eq!(x_mins.len(), level_index);
            assert_eq!(y_mins.len(), level_index);
            assert_eq!(x_maxs.len(), level_index);
            assert_eq!(y_maxs.len(), level_index);

            let mut idx = level_indices[level - 1];
            while idx < level_index {
                let mut x_min = f64::NAN;
                let mut y_min = f64::NAN;
                let mut x_max = f64::NAN;
                let mut y_max = f64::NAN;

                for _i in 0..degree {
                    x_min = x_min.min(x_mins.0[idx]);
                    y_min = y_min.min(y_mins.0[idx]);
                    x_max = x_max.max(x_maxs.0[idx]);
                    y_max = y_max.max(y_maxs.0[idx]);
                    idx += 1;
                }

                x_mins.push(x_min);
                y_mins.push(y_min);
                x_maxs.push(x_max);
                y_maxs.push(y_max);
            }
        }

        Self {
            degree,
            size,
            level_indices,
            x_mins,
            y_mins,
            x_maxs,
            y_maxs,
        }
    }

    pub(crate) fn root(&self) -> (usize, usize) {
        (self.height() - 1, 0)
    }

    /// Get the index range for leaf nodes under this node.
    pub(crate) fn get_leaf_range(&self, level: usize, offset: usize) -> Vec<usize> {
        // pub(crate) fn get_leaf_range(&self, level: usize, offset: usize) -> Range<usize> {
        let width = self.degree.pow(level as u32);
        let range = Range {
            start: width * offset,
            // index is for coordinates, and coordinates.len() == rectangles.len() + 1
            end: self.size.min(width * (offset + 1)),
        };
        let result: Vec<usize> = range
            .into_iter()
            .filter(|i| !self.x_mins.0[*i].is_nan())
            .collect();
        result
    }
}
