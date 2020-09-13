use std::cmp::Ordering;

use crate::hilbert::Hilbert;
use crate::{HasEnvelope, PackedRTreeNative, RTree, Rectangle};

type Entry = (usize, Rectangle);

pub struct SortedPackedRTree {
    raw_rtree: PackedRTreeNative,
    shuffled_indices: Vec<usize>,
}

impl RTree for SortedPackedRTree {
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

impl SortedPackedRTree {
    pub fn new_empty() -> Self {
        SortedPackedRTree {
            raw_rtree: PackedRTreeNative::new_empty(),
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
        SortedPackedRTree {
            shuffled_indices: entries.iter().map(|(_h, i, _e)| *i).collect(),
            raw_rtree: PackedRTreeNative::new(degree, &rects),
        }
    }

    pub fn new_omt(_degree: usize, items: &[impl HasEnvelope]) -> Self {
        // FIXME: adding _degree param just to make tests easier.
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
        // While 0 is a valid index, those leaves are empty and should never match
        let empty_index = 0;
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

        SortedPackedRTree {
            shuffled_indices,
            raw_rtree: PackedRTreeNative::new(degree, &rects),
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
    // TODO: Use total_cmp when stabilized
    let x1 = x_center(&entry1.1);
    let x2 = x_center(&entry2.1);
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

/// Order by y center, empties are last
fn cmp_y(entry1: &Entry, entry2: &Entry) -> Ordering {
    let y1 = y_center(&entry1.1);
    let y2 = y_center(&entry2.1);
    match y1.partial_cmp(&y2) {
        Some(ord) => ord,
        None => {
            // One or both is a NaN
            if y1.is_nan() {
                if y2.is_nan() {
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
    // TODO: Use partition_at_index_by here
    entries.sort_unstable_by(cmp_x);
    let column_size = divup(size, ncols);
    for ix in (0..size).step_by(column_size) {
        let actual_column_size = size.min(ix + column_size) - ix;
        let row_size = divup(actual_column_size, nrows);
        entries[ix..(ix + actual_column_size)].sort_unstable_by(cmp_y);
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
