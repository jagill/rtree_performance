use std::cmp::Ordering;

use crate::hilbert::Hilbert;
use crate::utils::divup;
use crate::{HasEnvelope, PackedRTreeUnsorted, RTree, Rectangle};

type Entry = (usize, Rectangle);

pub struct PackedRTree {
    pub raw_rtree: PackedRTreeUnsorted,
    shuffled_indices: Vec<usize>,
}

impl RTree for PackedRTree {
    fn is_empty(&self) -> bool {
        self.raw_rtree.is_empty()
    }

    fn height(&self) -> usize {
        self.raw_rtree.height()
    }

    fn degree(&self) -> usize {
        self.raw_rtree.degree()
    }

    fn envelope(&self) -> Rectangle {
        self.raw_rtree.envelope()
    }

    fn query_rect(&self, query: &Rectangle) -> Vec<usize> {
        let raw_results = self.raw_rtree.query_rect(query);
        raw_results
            .into_iter()
            .map(|i| self.shuffled_indices[i])
            .collect()
    }
}

impl PackedRTree {
    pub fn new_empty() -> Self {
        PackedRTree {
            raw_rtree: PackedRTreeUnsorted::new_empty(),
            shuffled_indices: Vec::new(),
        }
    }

    pub fn new_hilbert(degree: usize, items: &[impl HasEnvelope]) -> Self {
        let total_envelope = Rectangle::of(items);
        if total_envelope.is_empty() {
            return Self::new_empty();
        }

        let hilbert_square = Hilbert::new(total_envelope);
        let mut entries: Vec<(u32, usize, Rectangle)> = items
            .iter()
            .map(|i| i.envelope())
            .enumerate()
            .map(|(i, e)| (hilbert_square.hilbert(e.center()), i, e))
            .collect();

        entries.sort_unstable_by_key(|&(h, _, _)| h);
        let rects: Vec<Rectangle> = entries.iter().map(|(_h, _i, rect)| *rect).collect();
        PackedRTree {
            shuffled_indices: entries.iter().map(|(_h, i, _e)| *i).collect(),
            raw_rtree: PackedRTreeUnsorted::new(degree, rects),
        }
    }

    pub fn new_omt_old(items: &[impl HasEnvelope]) -> Self {
        if items.is_empty() {
            return Self::new_empty();
        }

        // TODO: Calculate ncols, nrows dynamically to improve packing
        let ncols = 4;
        let nrows = 4;
        let degree = ncols * nrows;

        let mut entries: Vec<Entry> = items
            .iter()
            .map(|item| item.envelope())
            .enumerate()
            .collect();
        let mut offsets = vec![0];
        offsets.extend(partition_omt(&mut entries, ncols, nrows, 0));
        offsets.sort_unstable();

        let total_size = divup(items.len(), degree) * degree;
        let mut shuffled_indices: Vec<usize> = Vec::with_capacity(total_size);
        let mut rects: Vec<Rectangle> = Vec::with_capacity(total_size);
        // If we match an empty rect, this will cause an out-of-bounds panic.
        let empty_index = usize::MAX;
        let empty_rect = Rectangle::new_empty();
        for (&last_offset, &next_offset) in offsets.iter().zip(&offsets[1..]) {
            let these_entries = &entries[last_offset..next_offset];
            shuffled_indices.extend(these_entries.iter().map(|(i, _e)| i));
            rects.extend(these_entries.iter().map(|(_i, e)| e));
            // Pad leaves so that we always have a multiple of degree
            let excess = degree - (next_offset - last_offset);
            shuffled_indices.extend(vec![empty_index; excess]);
            rects.extend(vec![empty_rect; excess]);
        }

        PackedRTree {
            shuffled_indices,
            raw_rtree: PackedRTreeUnsorted::new(degree, rects),
        }
    }

    pub fn new_omt(items: &[impl HasEnvelope]) -> Self {
        if items.is_empty() {
            return Self::new_empty();
        }

        let degree = 16;

        let mut entries: Vec<Entry> = items
            .iter()
            .map(|item| item.envelope())
            .enumerate()
            .collect();
        let scale = find_scale(entries.len(), degree);
        partition_omt_2(&mut entries, degree, scale);

        let shuffled_indices = entries.iter().map(|(i, _e)| *i).collect();
        let rects: Vec<Rectangle> = entries.iter().map(|(_i, rect)| *rect).collect();
        PackedRTree {
            shuffled_indices,
            raw_rtree: PackedRTreeUnsorted::new(degree, rects),
        }
    }

    pub fn new_omt_2(items: &[impl HasEnvelope]) -> Self {
        if items.is_empty() {
            return Self::new_empty();
        }

        let degree = 16;

        let mut entries: Vec<Entry> = items
            .iter()
            .map(|item| item.envelope())
            .enumerate()
            .collect();
        let scale = find_scale(entries.len(), degree);
        partition_omt_2(&mut entries, degree, scale);

        let shuffled_indices = entries.iter().map(|(i, _e)| *i).collect();
        let rects: Vec<Rectangle> = entries.iter().map(|(_i, rect)| *rect).collect();
        PackedRTree {
            shuffled_indices,
            raw_rtree: PackedRTreeUnsorted::new(degree, rects),
        }
    }
}

fn x_center(rect: &Rectangle) -> f64 {
    (rect.x_min + rect.x_max) / 2.
}

fn y_center(rect: &Rectangle) -> f64 {
    (rect.y_min + rect.y_max) / 2.
}

/// Order by x center, empties are last
fn cmp_x(entry1: &Entry, entry2: &Entry) -> Ordering {
    total_cmp(x_center(&entry1.1), x_center(&entry2.1))
}

/// Order by y center, empties are last
fn cmp_y(entry1: &Entry, entry2: &Entry) -> Ordering {
    total_cmp(y_center(&entry1.1), y_center(&entry2.1))
}

fn total_cmp(x1: f64, x2: f64) -> Ordering {
    // TODO: Use total_cmp when stabilized
    // x1.total_cmp(&x2)
    match x1.partial_cmp(&x2) {
        Some(ord) => ord,
        None => {
            // One or both is a NaN
            if x1.is_nan() {
                if x2.is_nan() {
                    Ordering::Equal
                } else {
                    Ordering::Greater
                }
            } else {
                Ordering::Less
            }
        }
    }
}

fn find_scale(num_entries: usize, degree: usize) -> usize {
    let mut scale = 1;
    let mut chunk_size = degree;

    while divup(num_entries, chunk_size) > 1 {
        scale += 1;
        chunk_size *= degree;
    }

    scale
}

fn find_cols_rows_remainder(num_chunks: usize) -> (usize, usize, usize) {
    let num_rows = (num_chunks as f64).sqrt().floor() as usize;
    let num_cols = if num_rows * (num_rows + 1) <= num_chunks {
        num_rows + 1
    } else {
        num_rows
    };
    let remainder = num_chunks % (num_rows * num_cols);
    (num_cols, num_rows, remainder)
}

fn partition_omt_2(entries: &mut [Entry], degree: usize, scale: usize) {
    if scale == 0 {
        return;
    }

    let num_entries = entries.len();
    let chunk_size = degree.checked_pow(scale as u32).unwrap();
    match divup(num_entries, chunk_size) {
        0 => return,
        1 => {
            return partition_omt_2(entries, degree, scale - 1);
        }
        i if i > degree => panic!(
            "partition_omt called with insufficient scale: {} num_entries {}, degree {}",
            scale, num_entries, degree
        ),
        _ => (),
    }

    let boundaries = find_boundaries(num_entries, chunk_size);
    sort_by_boundaries(entries, &boundaries, true);
    for bounds in boundaries.windows(2) {
        let (low, hi) = (bounds[0], bounds[1]);
        if hi - low > chunk_size {
            let mut row_boundaries: Vec<usize> = (low..hi).step_by(chunk_size).collect();
            row_boundaries.push(hi);
            sort_by_boundaries(&mut entries[low..hi], &row_boundaries, false);
        }
    }
    if scale > 1 {
        for low in (0..num_entries).step_by(chunk_size) {
            partition_omt_2(&mut entries[low..(low + chunk_size)], degree, scale - 1);
        }
    }
}

fn find_boundaries(num_entries: usize, chunk_size: usize) -> Vec<usize> {
    let (n_cols, n_rows, remainder) = find_cols_rows_remainder(divup(num_entries, chunk_size));

    let mut current_pivot = 0;
    let mut pivots = vec![current_pivot];
    for x_cut in 0..n_cols {
        current_pivot += n_rows + if x_cut < remainder { 1 } else { 0 };
        pivots.push(num_entries.min(current_pivot * chunk_size));
    }

    pivots
}

fn sort_by_boundaries(entries: &mut [Entry], boundaries: &[usize], along_x: bool) {
    let mut stack = vec![0, boundaries.len() - 1];
    let entries_start = boundaries[0];

    while !stack.is_empty() {
        let high = stack.pop().unwrap();
        let low = stack.pop().unwrap();
        if (high - low) <= 1 {
            continue;
        }
        let range_min = boundaries[low] - entries_start;
        let range_max = boundaries[high] - entries_start;

        let mid = (low + high) / 2;
        let pivot = boundaries[mid] - range_min;
        if along_x {
            &mut entries[range_min..range_max].select_nth_unstable_by(pivot, cmp_x);
        } else {
            &mut entries[range_min..range_max].select_nth_unstable_by(pivot, cmp_y);
        }

        stack.extend(&[low, mid, mid, high]);
    }
}

fn partition_omt(entries: &mut [Entry], ncols: usize, nrows: usize, start: usize) -> Vec<usize> {
    let size = entries.len();
    if size < ncols * nrows {
        return vec![start + size];
    }

    let mut results = Vec::new();
    let column_size = divup(size, ncols);
    partition_to_chunks(column_size, entries, true);
    for ix in (0..size).step_by(column_size) {
        let actual_column_size = size.min(ix + column_size) - ix;
        let row_size = divup(actual_column_size, nrows);
        partition_to_chunks(row_size, &mut entries[ix..(ix + actual_column_size)], false);
        for iy in (ix..(ix + actual_column_size)).step_by(row_size) {
            let actual_row_size = (ix + actual_column_size).min(iy + row_size) - iy;
            results.extend(partition_omt(
                &mut entries[iy..(iy + actual_row_size)],
                ncols,
                nrows,
                iy,
            ))
        }
    }

    results
}

// Ported from github.com/mourner/rbush
fn partition_to_chunks(chunk_size: usize, entries: &mut [Entry], along_x: bool) {
    let mut stack = vec![0, entries.len()];

    while !stack.is_empty() {
        let high = stack.pop().unwrap();
        let low = stack.pop().unwrap();
        if (high - low) <= chunk_size {
            continue;
        }

        let mid = low + chunk_size * divup(high - low, 2 * chunk_size);
        if along_x {
            &mut entries[low..high].select_nth_unstable_by(mid - low, cmp_x);
        } else {
            &mut entries[low..high].select_nth_unstable_by(mid - low, cmp_y);
        }

        stack.extend(vec![low, mid, mid, high]);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_find_scale() {
        assert_eq!(1, find_scale(0, 4));
        assert_eq!(1, find_scale(1, 4));
        assert_eq!(1, find_scale(3, 4));
        assert_eq!(1, find_scale(4, 4));
        assert_eq!(2, find_scale(5, 4));
        assert_eq!(2, find_scale(8, 4));
        assert_eq!(2, find_scale(16, 4));
        assert_eq!(3, find_scale(17, 4));
        assert_eq!(4, find_scale(9, 2));
    }

    #[test]
    fn test_find_cols_rows_remainder() {
        assert_eq!((2, 1, 0), find_cols_rows_remainder(2));
        assert_eq!((2, 1, 1), find_cols_rows_remainder(3));
        assert_eq!((2, 2, 0), find_cols_rows_remainder(4));
        assert_eq!((2, 2, 1), find_cols_rows_remainder(5));
        assert_eq!((3, 2, 0), find_cols_rows_remainder(6));
        assert_eq!((3, 2, 1), find_cols_rows_remainder(7));
        assert_eq!((3, 2, 2), find_cols_rows_remainder(8));
        assert_eq!((3, 3, 0), find_cols_rows_remainder(9));
        assert_eq!((3, 3, 1), find_cols_rows_remainder(10));
        assert_eq!((3, 3, 2), find_cols_rows_remainder(11));
        assert_eq!((4, 3, 0), find_cols_rows_remainder(12));
        assert_eq!((4, 3, 1), find_cols_rows_remainder(13));
        assert_eq!((4, 3, 2), find_cols_rows_remainder(14));
        assert_eq!((4, 3, 3), find_cols_rows_remainder(15));
        assert_eq!((4, 4, 0), find_cols_rows_remainder(16));
    }

    #[test]
    fn test_find_pivots() {
        assert_eq!(find_boundaries(2, 1), &[0, 1, 2]);
        assert_eq!(find_boundaries(3, 1), &[0, 2, 3]);
        assert_eq!(find_boundaries(4, 1), &[0, 2, 4]);
        assert_eq!(find_boundaries(5, 1), &[0, 3, 5]);
        assert_eq!(find_boundaries(6, 1), &[0, 2, 4, 6]);
        assert_eq!(find_boundaries(7, 1), &[0, 3, 5, 7]);
        assert_eq!(find_boundaries(8, 1), &[0, 3, 6, 8]);
        assert_eq!(find_boundaries(9, 1), &[0, 3, 6, 9]);
        assert_eq!(find_boundaries(10, 1), &[0, 4, 7, 10]);
        assert_eq!(find_boundaries(11, 1), &[0, 4, 8, 11]);
        assert_eq!(find_boundaries(12, 1), &[0, 3, 6, 9, 12]);
        assert_eq!(find_boundaries(13, 1), &[0, 4, 7, 10, 13]);
        assert_eq!(find_boundaries(14, 1), &[0, 4, 8, 11, 14]);
        assert_eq!(find_boundaries(15, 1), &[0, 4, 8, 12, 15]);
        assert_eq!(find_boundaries(16, 1), &[0, 4, 8, 12, 16]);
    }

    #[test]
    fn test_sort_by_boundaries() {
        let entries: Vec<Entry> = vec![
            (0, Rectangle::from((10., 10.))),
            (1, Rectangle::from((9., 9.))),
            (2, Rectangle::from((8., 8.))),
            (3, Rectangle::from((7., 7.))),
            (4, Rectangle::from((6., 6.))),
            (5, Rectangle::from((5., 5.))),
            (6, Rectangle::from((4., 4.))),
            (7, Rectangle::from((3., 3.))),
            (8, Rectangle::from((2., 2.))),
            (9, Rectangle::from((1., 1.))),
        ];
        let mut reordered = entries.clone();
        let boundaries = [0, 3, 6, 10];
        sort_by_boundaries(&mut reordered, &boundaries, true);
        let mut reordered_indices: Vec<usize> = reordered.into_iter().map(|(i, _e)| i).collect();
        for bounds in boundaries.windows(2) {
            let (low, hi) = (bounds[0], bounds[1]);
            reordered_indices[low..hi].sort_unstable();
        }
        assert_eq!(reordered_indices, vec![7, 8, 9, 4, 5, 6, 0, 1, 2, 3]);
    }

    #[test]
    fn test_partition_omt() {
        let entries: Vec<Entry> = vec![
            (0, Rectangle::from((10., 0.))),
            (1, Rectangle::from((9., 1.))),
            (2, Rectangle::from((8., 2.))),
            (3, Rectangle::from((7., 3.))),
            (4, Rectangle::from((6., 4.))),
            (5, Rectangle::from((5., 5.))),
            (6, Rectangle::from((4., 6.))),
            (7, Rectangle::from((3., 7.))),
            (8, Rectangle::from((2., 8.))),
            (9, Rectangle::from((1., 9.))),
        ];
        let mut reordered = entries.clone();
        let degree = 4;
        let scale = find_scale(entries.len(), degree);
        partition_omt_2(&mut reordered, degree, scale);
        let reordered_indices: Vec<usize> = reordered.into_iter().map(|(i, _e)| i).collect();
        // XXX: This is not really a good test.
        assert_eq!(reordered_indices, vec![9, 8, 7, 6, 5, 4, 3, 2, 1, 0]);
    }
}
