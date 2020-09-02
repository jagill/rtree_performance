use crate::utils::{calculate_level_indices, copy_into_slice};
use crate::{Coordinate, Rectangle};

#[derive(Debug, Clone)]
pub struct SegRTree {
    degree: usize,
    size: usize,
    level_indices: Vec<usize>,
    tree: Vec<Rectangle>,
}

#[allow(dead_code)]
impl SegRTree {
    pub fn envelope(&self) -> Rectangle {
        self.get_rectangle(self.height() - 1, 0)
    }

    pub(crate) fn len(&self) -> usize {
        self.size
    }

    pub fn is_empty(&self) -> bool {
        self.size == 0
    }

    pub fn height(&self) -> usize {
        self.level_indices.len()
    }
    pub fn degree(&self) -> usize {
        self.degree
    }

    pub fn new_empty() -> Self {
        SegRTree {
            degree: 2,
            size: 0,
            level_indices: Vec::new(),
            tree: Vec::new(),
        }
    }

    pub fn new(mut degree: usize, rects: &[Rectangle]) -> Self {
        degree = degree.max(2);
        let size = rects.len();
        let level_indices = calculate_level_indices(degree, size);
        let tree_size = level_indices[level_indices.len() - 1] + 1;
        let empty_rect = Rectangle::new_empty();
        let mut tree = vec![empty_rect; tree_size];
        copy_into_slice(&mut tree, 0, rects);

        for level in 1..level_indices.len() {
            let level_index = level_indices[level];
            let previous_items = &tree[level_indices[level - 1]..level_index];
            let next_items: Vec<Rectangle> = previous_items
                .chunks(degree)
                .map(|items| Rectangle::of(items))
                .collect();
            copy_into_slice(&mut tree, level_index, &next_items);
        }

        tree.shrink_to_fit();
        SegRTree {
            degree,
            size,
            level_indices,
            tree,
        }
    }

    pub fn query_point(&self, coord: Coordinate) -> Vec<usize> {
        self.query_rect(Rectangle::new(coord, coord))
    }

    pub fn query_rect(&self, rect: Rectangle) -> Vec<usize> {
        let mut results = Vec::new();
        if self.is_empty() {
            return results;
        }

        // Stack entries: (level, offset)
        let mut stack = Vec::new();
        if rect.intersects(self.envelope()) {
            stack.push(self.root())
        }
        while let Some((level, offset)) = stack.pop() {
            if level == 0 {
                results.push(offset);
            } else {
                let child_level = level - 1;
                let first_child_offset = self.degree * offset;
                for child_offset in first_child_offset..(first_child_offset + self.degree) {
                    if rect.intersects(self.get_rectangle(child_level, child_offset)) {
                        stack.push((child_level, child_offset));
                    }
                }
            }
        }

        results
    }

    pub fn query_rect_2(&self, rect: Rectangle) -> Vec<usize> {
        let mut results = Vec::new();
        if self.is_empty() {
            return results;
        }

        // Stack entries: (level, offset)
        let mut stack = Vec::new();
        if rect.intersects(self.envelope()) {
            stack.push(self.root())
        }
        while let Some((level, offset)) = stack.pop() {
            if level == 0 {
                results.push(offset);
            } else {
                let child_level = level - 1;
                let first_child_offset = self.degree * offset;
                let first_child_index = self.find_index(child_level, first_child_offset);
                let children = &self.tree[first_child_index..(first_child_index + self.degree)];
                for (inc, tree_rect) in children.iter().enumerate() {
                    if rect.intersects(*tree_rect) {
                        stack.push((child_level, first_child_offset + inc));
                    }
                }
            }
        }

        results
    }

    pub fn query_self_intersections(&self) -> Vec<(usize, usize)> {
        let mut results = Vec::new();
        if self.is_empty() {
            return results;
        }

        // Stack entries: (level, offset)
        let mut stack = vec![(self.height(), 0, self.height(), 0)];

        while let Some((level_a, offset_a, level_b, offset_b)) = stack.pop() {
            let rect_a = self.get_rectangle(level_a, offset_a);
            let rect_b = self.get_rectangle(level_b, offset_b);
            if !rect_a.intersects(rect_b) {
                continue;
            }

            if level_a == 0 && level_b == 0 {
                if offset_a < offset_b {
                    results.push((offset_a, offset_b));
                }
            } else if level_a == level_b {
                let child_level = level_a - 1;
                let first_child_offset = self.degree * offset_a;
                for child_offset in first_child_offset..(first_child_offset + self.degree) {
                    stack.push((child_level, child_offset, level_b, offset_b));
                }
            } else {
                assert_eq!(level_a + 1, level_b);
                let child_level = level_b - 1;
                let first_child_offset = self.degree * offset_b;
                let last_child_offset = first_child_offset + self.degree;
                for child_offset in first_child_offset..last_child_offset {
                    stack.push((level_a, offset_a, child_level, child_offset));
                }
            }
        }

        results
    }

    pub fn query_other_intersections(&self, other: &SegRTree) -> Vec<(usize, usize)> {
        let mut results = Vec::new();
        if self.is_empty() || other.is_empty() {
            return results;
        }

        // Stack entries: (level, offset)
        let mut stack = vec![(self.height(), 0, other.height(), 0)];

        while let Some((level_a, offset_a, level_b, offset_b)) = stack.pop() {
            let rect_a = self.get_rectangle(level_a, offset_a);
            let rect_b = other.get_rectangle(level_b, offset_b);
            if !rect_a.intersects(rect_b) {
                continue;
            }

            if level_a == 0 && level_b == 0 {
                results.push((offset_a, offset_b));
            } else if level_a >= level_b {
                let child_level = level_a - 1;
                let first_child_offset = self.degree * offset_a;
                for child_offset in first_child_offset..(first_child_offset + self.degree) {
                    stack.push((child_level, child_offset, level_b, offset_b));
                }
            } else {
                let child_level = level_b - 1;
                let first_child_offset = other.degree * offset_b;
                let last_child_offset = first_child_offset + other.degree;
                for child_offset in first_child_offset..last_child_offset {
                    stack.push((level_a, offset_a, child_level, child_offset));
                }
            }
        }

        results
    }

    fn find_index(&self, level: usize, offset: usize) -> usize {
        self.level_indices[level] + offset
    }

    pub(crate) fn get_rectangle(&self, level: usize, offset: usize) -> Rectangle {
        self.tree[self.level_indices[level] + offset]
    }

    pub(crate) fn get_low_high(&self, level: usize, offset: usize) -> (usize, usize) {
        let width = self.degree.pow(level as u32);
        // index is for coordinates, and coordinates.len() == rectangles.len() + 1
        let max_index = self.size;
        (width * offset, max_index.min(width * (offset + 1)))
    }

    pub(crate) fn root(&self) -> (usize, usize) {
        (self.height() - 1, 0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Coordinate, Rectangle};
    use rand::rngs::SmallRng;
    use rand::{Rng, SeedableRng};

    #[test]
    fn test_empty_seg_rtree() {
        let r = Rectangle {
            x_min: -10.,
            y_min: -5.,
            x_max: 1.,
            y_max: 5.,
        };
        let tree = SegRTree::new_empty();
        assert_eq!(tree.len(), 0);
        assert_eq!(tree.height(), 0);
        assert_eq!(tree.query_rect(r), Vec::<usize>::new());
    }

    fn _assert_queries(max_index: usize, tree: &SegRTree, rects: &[Rectangle]) {
        #[allow(clippy::needless_range_loop)]
        for i in 0..=max_index {
            assert_eq!(tree.query_rect(rects[i]), vec![i]);
        }
    }

    fn assert_low_high(rtree: &SegRTree, height: usize, offset: usize, size: usize) {
        let (low, high) = rtree.get_low_high(height, offset);
        assert!(low <= size);
        assert!(high <= size);
    }

    #[test]
    fn test_low_high_indices() {
        let mut rng = SmallRng::seed_from_u64(177);

        for _i in 0..50 {
            let size = rng.gen_range(1, 1000);
            let zero = Coordinate::new(0., 0.);
            let rect = Rectangle::new(zero, zero);
            let rects = vec![rect; size];
            let rtree = SegRTree::new(16, &rects);
            assert_low_high(&rtree, rtree.height(), 0, size);
        }
    }

    #[test]
    fn test_build_tree_unsorted() {
        let degree = 4;
        let e0 = Rectangle::new((7.0, 44.).into(), (8., 48.).into());
        let e1 = Rectangle::new((25., 48.).into(), (35., 55.).into());
        let e2 = Rectangle::new((98., 46.).into(), (99., 56.).into());
        let e3 = Rectangle::new((58., 65.).into(), (73., 79.).into());
        let e4 = Rectangle::new((43., 40.).into(), (44., 45.).into());
        let e5 = Rectangle::new((97., 87.).into(), (100., 91.).into());
        let e6 = Rectangle::new((92., 46.).into(), (108., 57.).into());
        let e7 = Rectangle::new((7.1, 48.).into(), (10., 56.).into());
        let envs = vec![e0, e1, e2, e3, e4, e5, e6, e7];

        let rtree = SegRTree::new(degree, &envs);
        let query_point = (43., 43.).into();
        let query_rect = Rectangle::new(query_point, query_point);

        let results = rtree.query_rect(query_rect);
        assert_eq!(results, vec![4]);
        let results = rtree.query_rect_2(query_rect);
        assert_eq!(results, vec![4]);
    }
}
