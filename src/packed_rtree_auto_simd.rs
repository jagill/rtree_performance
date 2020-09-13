use crate::utils::{calculate_level_indices, copy_into_slice};
use crate::{Coordinate, RTree, Rectangle};

// [x_min, y_min, -xmax, -ymax]
#[repr(align(64))]
#[derive(Copy, Clone, Debug)]
pub struct BBox([f64; 4]);
impl BBox {
    const EMPTY_BBOX: BBox = BBox([f64::INFINITY; 4]);

    pub fn to_rectangle(&self) -> Rectangle {
        Rectangle {
            x_min: self.0[0],
            y_min: self.0[1],
            x_max: -self.0[2],
            y_max: -self.0[3],
        }
    }
}

impl From<&Rectangle> for BBox {
    fn from(rect: &Rectangle) -> Self {
        BBox([rect.x_min, rect.y_min, -rect.x_max, -rect.y_max])
    }
}

#[derive(Debug, Clone)]
pub struct PackedRTreeAutoSimd {
    degree: usize,
    size: usize,
    level_indices: Vec<usize>,
    tree: Vec<BBox>,
}

impl RTree for PackedRTreeAutoSimd {
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
            self.get_bbox(self.height() - 1, 0).to_rectangle()
        }
    }

    fn query_rect(&self, rect: &Rectangle) -> Vec<usize> {
        let mut results = Vec::new();
        if self.is_empty() {
            return results;
        }

        // Rearrange this for fast checking
        let query_bbox = BBox([rect.x_max, rect.y_max, -rect.x_min, -rect.y_min]);

        // Stack entries: (level, offset)
        let mut stack = Vec::new();
        {
            let root = self.root();
            let root_bbox = self.get_bbox(root.0, root.1);
            if (root_bbox.0[0] <= query_bbox.0[0])
                & (root_bbox.0[1] <= query_bbox.0[1])
                & (root_bbox.0[2] <= query_bbox.0[2])
                & (root_bbox.0[3] <= query_bbox.0[3])
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
                children.iter().enumerate().for_each(|(inc, tree_bbox)| {
                    if (tree_bbox.0[0] <= query_bbox.0[0])
                        & (tree_bbox.0[1] <= query_bbox.0[1])
                        & (tree_bbox.0[2] <= query_bbox.0[2])
                        & (tree_bbox.0[3] <= query_bbox.0[3])
                    {
                        stack.push((child_level, first_child_offset + inc))
                    }
                });
            }
        }

        results
    }
}

#[allow(dead_code)]
impl PackedRTreeAutoSimd {
    pub(crate) fn len(&self) -> usize {
        self.size
    }

    pub fn new_empty() -> Self {
        Self {
            degree: 2,
            size: 0,
            level_indices: Vec::new(),
            tree: Vec::new(),
        }
    }

    pub fn new(mut degree: usize, rects: &[Rectangle]) -> Self {
        if rects.is_empty() {
            return PackedRTreeAutoSimd::new_empty();
        }

        degree = degree.max(2);
        let size = rects.len();
        let level_indices = calculate_level_indices(degree, size);
        let tree_size = level_indices[level_indices.len() - 1] + 1;
        let mut tree = vec![BBox::EMPTY_BBOX; tree_size];
        for (i, rect) in rects.iter().enumerate() {
            tree[i] = rect.into();
        }

        for level in 1..level_indices.len() {
            let level_index = level_indices[level];
            let previous_items = &tree[level_indices[level - 1]..level_index];
            let next_items: Vec<BBox> = previous_items
                .chunks(degree)
                .map(|bboxes| {
                    let mut out = BBox::EMPTY_BBOX;
                    for bbox in bboxes {
                        out.0[0] = out.0[0].min(bbox.0[0]);
                        out.0[1] = out.0[1].min(bbox.0[1]);
                        out.0[2] = out.0[2].min(bbox.0[2]);
                        out.0[3] = out.0[3].min(bbox.0[3]);
                    }
                    out
                })
                .collect();
            copy_into_slice(&mut tree, level_index, &next_items);
        }

        tree.shrink_to_fit();
        Self {
            degree,
            size,
            level_indices,
            tree,
        }
    }

    pub fn query_point(&self, coord: Coordinate) -> Vec<usize> {
        self.query_rect(&Rectangle::new(coord, coord))
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
mod tests {}
