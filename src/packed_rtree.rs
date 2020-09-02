use crate::utils::{calculate_level_indices, copy_into_slice};
use crate::{Coordinate, Rectangle};

// [x_min, y_min, -xmax, -ymax]
type BBox = [f64; 4];
const EMPTY_BBOX: BBox = [f64::INFINITY; 4];

#[derive(Debug, Clone)]
pub struct PackedRTree {
    degree: usize,
    size: usize,
    level_indices: Vec<usize>,
    tree: Vec<BBox>,
}

#[allow(dead_code)]
impl PackedRTree {
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
        PackedRTree {
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
        let mut tree = vec![EMPTY_BBOX; tree_size];
        for (i, rect) in rects.iter().enumerate() {
            tree[i] = [rect.x_min, rect.y_min, -rect.x_max, -rect.y_max];
        }

        for level in 1..level_indices.len() {
            let level_index = level_indices[level];
            let previous_items = &tree[level_indices[level - 1]..level_index];
            let next_items: Vec<BBox> = previous_items
                .chunks(degree)
                .map(|bboxes| {
                    let mut out = EMPTY_BBOX;
                    for bbox in bboxes {
                        out[0] = out[0].min(bbox[0]);
                        out[1] = out[1].min(bbox[1]);
                        out[2] = out[2].min(bbox[2]);
                        out[3] = out[3].min(bbox[3]);
                    }
                    out
                })
                .collect();
            copy_into_slice(&mut tree, level_index, &next_items);
        }

        tree.shrink_to_fit();
        PackedRTree {
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

        // Rearrange this for fast checking
        let query_bbox = [rect.x_max, rect.y_max, -rect.x_min, -rect.y_min];

        // Stack entries: (level, offset)
        let mut stack = Vec::new();
        {
            let root = self.root();
            let root_bbox = self.get_bbox(root.0, root.1);
            if root_bbox[0] <= query_bbox[0]
                && root_bbox[1] <= query_bbox[1]
                && root_bbox[2] <= query_bbox[2]
                && root_bbox[3] <= query_bbox[3]
            {
                stack.push(root);
            }
        }

        while let Some((level, offset)) = stack.pop() {
            if level == 0 {
                results.push(offset);
            } else {
                let child_level = level - 1;
                let first_child_offset = self.degree * offset;
                let first_child_index = self.find_index(child_level, first_child_offset);
                let children = &self.tree[first_child_index..(first_child_index + self.degree)];
                for (inc, tree_bbox) in children.iter().enumerate() {
                    if tree_bbox[0] <= query_bbox[0]
                        && tree_bbox[1] <= query_bbox[1]
                        && tree_bbox[2] <= query_bbox[2]
                        && tree_bbox[3] <= query_bbox[3]
                    {
                        stack.push((child_level, first_child_offset + inc));
                    }
                }
            }
        }

        results
    }

    fn find_index(&self, level: usize, offset: usize) -> usize {
        self.level_indices[level] + offset
    }

    pub(crate) fn get_bbox(&self, level: usize, offset: usize) -> BBox {
        self.tree[self.find_index(level, offset)]
    }

    pub(crate) fn root(&self) -> (usize, usize) {
        (self.height() - 1, 0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_seg_rtree() {
        let r = Rectangle {
            x_min: -10.,
            y_min: -5.,
            x_max: 1.,
            y_max: 5.,
        };
        let tree = PackedRTree::new_empty();
        assert_eq!(tree.len(), 0);
        assert_eq!(tree.height(), 0);
        assert_eq!(tree.query_rect(r), Vec::<usize>::new());
    }

    fn _assert_queries(max_index: usize, tree: &PackedRTree, rects: &[Rectangle]) {
        #[allow(clippy::needless_range_loop)]
        for i in 0..=max_index {
            assert_eq!(tree.query_rect(rects[i]), vec![i]);
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

        let rtree = PackedRTree::new(degree, &envs);
        let query_point = (43., 43.).into();
        let query_rect = Rectangle::new(query_point, query_point);

        let results = rtree.query_rect(query_rect);
        assert_eq!(results, vec![4]);
    }
}
