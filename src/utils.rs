use crate::{Coordinate, Rectangle};

pub fn rectangles_from_coordinates(coords: &[Coordinate]) -> Vec<Rectangle> {
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
        // least multiple of degree >= level_size
        let level_capacity = degree * (divup(level_size, degree));
        level_indices.push(level_indices[level] + level_capacity);
        level += 1;
        level_size = level_capacity / degree;
        assert_eq!(level_indices.len(), level + 1);
    }
    level_indices
}

pub(crate) fn copy_into_slice<T: Copy>(slice: &mut [T], index: usize, items: &[T]) {
    let (_, subslice) = slice.split_at_mut(index);
    let (subslice, _) = subslice.split_at_mut(items.len());
    subslice.copy_from_slice(items);
}

pub(crate) fn divup(dividend: usize, divisor: usize) -> usize {
    let quotient = dividend / divisor;
    match dividend % divisor {
        0 => quotient,
        _ => quotient + 1,
    }
}
