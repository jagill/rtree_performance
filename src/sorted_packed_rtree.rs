use crate::hilbert::Hilbert;
use crate::{HasEnvelope, PackedRTreeNative, RTree, Rectangle};

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
    pub fn new_hilbert_sorted(degree: usize, items: &[impl HasEnvelope]) -> Self {
        let total_envelope = Rectangle::of(items);
        if total_envelope.is_empty() {
            return SortedPackedRTree {
                raw_rtree: PackedRTreeNative::new_empty(),
                shuffled_indices: Vec::new(),
            };
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
}
