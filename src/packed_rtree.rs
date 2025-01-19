use std::cmp::Ordering;

use crate::hilbert::Hilbert;
use crate::utils::divup;
use crate::{HasEnvelope, PackedRTreeUnsorted, RTree, Rectangle};

type Entry = (usize, Rectangle);
impl HasEnvelope for Entry {
    fn envelope(&self) -> Rectangle {
        self.1.envelope()
    }
}

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
        let chunk_size = partition_omt(&mut entries, degree);

        // Hilbert sort our chunks
        if chunk_size > degree {
            for idx in (0..degree).step_by(chunk_size) {
                let top = entries.len().min(idx + chunk_size);
                let slice = &mut entries[idx..top];

                let total_envelope = Rectangle::of(slice);
                if total_envelope.is_empty() {
                    continue;
                }

                let hilbert_square = Hilbert::new(total_envelope);
                // Really should partition into chunks, no need to sort smaller than chunk_size
                slice.sort_unstable_by_key(|(_i, e)| hilbert_square.hilbert(e.center()));
            }
        }

        // Build our shuffled indices, and Vec of Rectangles for the unsorted rtree
        let shuffled_indices = entries.iter().map(|(i, _e)| *i).collect();
        let mut rects = Vec::with_capacity(divup(chunk_size, degree) * degree);
        let empty_rect = Rectangle::new_empty();
        for i in 0..degree {
            let slice = &mut entries[i * chunk_size..(i + 1) * chunk_size];
            rects.extend(slice.iter().map(|(_i, e)| e));
            // We need to pad each l2
            rects.append(&mut vec![empty_rect; degree - slice.len()]);
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

/// Partition OMT once, returning the chunk size
fn partition_omt(entries: &mut [Entry], degree: usize) -> usize {
    let num_entries = entries.len();
    if num_entries <= degree {
        return num_entries;
    }

    let chunk_size = degree.max(divup(num_entries, degree));
    let num_chunks = divup(num_entries, chunk_size);

    let num_stripes = (degree as f32).sqrt() as usize;
    if num_stripes * num_stripes != degree {
        panic!("Degree must be a perfect squre.");
    }

    // Partition into 4 vertical stripes
    let stripe_size = chunk_size * num_stripes;
    partition_in_stripes(entries, stripe_size, true);

    // Partition each slice into 4 horizontal chunks
    for i in 0..num_stripes {
        let stripe_top = if i == num_stripes - 1 {
            entries.len()
        } else {
            (i + 1) * stripe_size
        };
        let stripe = &mut entries[i * stripe_size..stripe_top];
        partition_in_stripes(stripe, chunk_size, false);
    }

    chunk_size
}

// Partition entries into 4 groups of size using cmp
fn partition_in_stripes(entries: &mut [Entry], size: usize, by_x: bool) {
    if entries.len() <= size {
        return;
    }

    let compare = if by_x { cmp_x } else { cmp_y };
    if entries.len() <= 2 * size {
        entries.select_nth_unstable_by(size, compare);
        return;
    }
    entries.select_nth_unstable_by(2 * size, compare);
    entries[..2 * size].select_nth_unstable_by(size, compare);
    if entries.len() > 3 * size {
        entries[2 * size..].select_nth_unstable_by(size, compare);
    }
}

// Ported from github.com/mourner/rbush
#[allow(dead_code)]
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
    fn test_partition_omt() {
        let mut rects = make_grid_rects(5);
        let chunk_size = partition_omt(&mut rects, 16);
        assert_eq!(chunk_size, 2);
        println!("{:#?}", rects);
        // assert_eq!(rects, Vec::new());
    }

    fn make_grid_rects(sqrt_num: usize) -> Vec<Entry> {
        let mut rects = Vec::with_capacity(sqrt_num * sqrt_num);
        let mut a = 0;
        for i in 0..sqrt_num {
            for j in 0..sqrt_num {
                rects.push((a, Rectangle::from((i as f64, j as f64))));
                a += 1;
            }
        }

        rects
    }
}
