use crate::{Coordinate, Rectangle};

pub(crate) fn rectangles_from_coordinates(coords: &[Coordinate]) -> Vec<Rectangle> {
    coords
        .windows(2)
        .map(|c| Rectangle::new(c[0], c[1]))
        .collect()
}

pub(crate) fn calculate_level_indices(degree: usize, num_items: usize) -> Vec<usize> {
    let mut level_indices: Vec<usize> = vec![0];

    let mut level = 0;
    let mut level_size = num_items;

    while level_size > 1 {
        let level_buffer = if level_size % degree > 0 { 1 } else { 0 };
        // least multiple of degree >= level_size
        let level_capacity = degree * (level_size / degree + level_buffer);
        level_indices.push(level_indices[level] + level_capacity);
        level += 1;
        level_size = level_capacity / degree;
        assert_eq!(level_indices.len(), level + 1);
    }
    level_indices
}

pub(crate) fn copy_into_slice<T: Copy>(tree: &mut [T], index: usize, items: &[T]) {
    let (_, subtree) = tree.split_at_mut(index);
    let (subtree, _) = subtree.split_at_mut(items.len());
    subtree.copy_from_slice(items);
}
