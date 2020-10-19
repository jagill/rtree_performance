/**
 * A fast, low memory footprint static Rtree.
 *
 * This implementation is cache-oblivious and SIMD-friendly, but does not do
 * anything explicit to enable vectorizatino.
 */
use crate::utils::calculate_level_indices;
use crate::{RTree, Rectangle};
use core::ops::Range;

#[derive(Debug)]
pub struct PackedRTreeUnsorted {
    degree: usize,
    size: usize,
    // nodes in level i are (level_indices[i] .. level_indices[i + 1]) (end exclusive)
    level_indices: Vec<usize>,
    tree: Vec<Rectangle>,
}

impl RTree for PackedRTreeUnsorted {
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
            self.tree[self.level_indices[self.height() - 1]]
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
            let children = &self.tree[first_child_index..(first_child_index + self.degree)];

            for (inc, child) in children.iter().enumerate() {
                if query.intersects(child) {
                    let child_offset = first_child_offset + inc;
                    if query.contains(child) {
                        // If it contains the node, it contains (and intersects) all the leaves
                        results.extend(self.get_leaf_range(child_level, child_offset))
                    } else {
                        stack.push((child_level, child_offset));
                    }
                }
            }
        }

        results
    }
}

impl PackedRTreeUnsorted {
    pub fn leaves(&self) -> &[Rectangle] {
        let leaf_size = self.level_indices.len();
        if leaf_size == 0 {
            &[]
        } else if leaf_size == 1 {
            &self.tree[..1]
        } else {
            &self.tree[..self.level_indices[1]]
        }
    }

    pub fn new_empty() -> Self {
        Self {
            degree: 2,
            size: 0,
            level_indices: Vec::new(),
            tree: Vec::new(),
        }
    }

    pub fn new(mut degree: usize, mut rects: Vec<Rectangle>) -> Self {
        if rects.is_empty() {
            return PackedRTreeUnsorted::new_empty();
        }

        degree = degree.max(2);
        let size = rects.len();
        let level_indices = calculate_level_indices(degree, size);
        let tree_size = level_indices[level_indices.len() - 1] + 1;
        rects.reserve(tree_size);

        for level in 1..level_indices.len() {
            let level_index = level_indices[level];
            rects.extend(vec![Rectangle::new_empty(); level_index - rects.len()]);
            assert_eq!(rects.len(), level_index);

            let level_rects = &rects[level_indices[level - 1]..level_indices[level]];
            let next_rects: Vec<Rectangle> = level_rects
                .chunks(degree)
                .map(|rects| Rectangle::of(rects))
                .collect();
            rects.extend(next_rects);
        }

        rects.shrink_to_fit();

        Self {
            degree,
            size,
            level_indices,
            tree: rects,
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
            .filter(|i| !self.tree[*i].is_empty())
            .collect();
        result
    }
}
