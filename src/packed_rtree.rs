use std::cmp::Ordering;

use crate::hilbert::Hilbert;
use crate::{HasEnvelope, PackedRTreeUnsorted, RTree, Rectangle};

type Entry = (usize, Rectangle);

pub struct PackedRTree {
    raw_rtree: PackedRTreeUnsorted,
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

    pub fn new_omt(items: &[impl HasEnvelope]) -> Self {
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
        let total_size = offsets[offsets.len() - 1];

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

fn divup(n: usize, d: usize) -> usize {
    let remainder = match n % d {
        0 => 0,
        _ => 1,
    };
    n / d + remainder
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
            order_stat::kth_by(&mut entries[low..high], mid - low, cmp_x);
        } else {
            order_stat::kth_by(&mut entries[low..high], mid - low, cmp_y);
        }

        stack.extend(vec![low, mid, mid, high]);
    }
}
