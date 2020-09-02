mod utils;

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};

use rtree_performance::{Flatbush, PackedRTree, Rectangle, SegRTree};
use utils::{get_positions_list, get_random_points, make_rectangles_list};

pub fn query_benchmark(c: &mut Criterion) {
    let name = "africa";
    let positions_list = get_positions_list(name);
    let rectangles_list = make_rectangles_list(&positions_list);
    println!("Benchmarking {} polygons", rectangles_list.len());
    let mut group = c.benchmark_group(format!("query_{}", name));

    for (poly_idx, rectangles) in rectangles_list.iter().enumerate() {
        println!("Polygon {} has {} segments.", poly_idx, rectangles.len());
        for &degree in [8, 16].iter() {
            let seg_rtree = SegRTree::new(degree, rectangles);
            let query_rects: Vec<_> = get_random_points(seg_rtree.envelope(), 1000, 342)
                .into_iter()
                .map(|p| Rectangle::new(p, p))
                .collect();

            group.bench_function(
                BenchmarkId::new(format!("seg_rtree_query.{}", poly_idx), degree),
                |b| {
                    // for coords in &positions_list {
                    b.iter(|| {
                        for &rect in &query_rects {
                            black_box(seg_rtree.query_rect(rect));
                        }
                    })
                },
            );

            group.bench_function(
                BenchmarkId::new(format!("seg_rtree_query_2.{}", poly_idx), degree),
                |b| {
                    // for coords in &positions_list {
                    b.iter(|| {
                        for &rect in &query_rects {
                            black_box(seg_rtree.query_rect_2(rect));
                        }
                    })
                },
            );

            let packed_rtree = PackedRTree::new(degree, rectangles);

            group.bench_function(
                BenchmarkId::new(format!("packed_rtree_query.{}", poly_idx), degree),
                |b| {
                    // for coords in &positions_list {
                    b.iter(|| {
                        for &rect in &query_rects {
                            black_box(packed_rtree.query_rect(rect));
                        }
                    })
                },
            );

            let flatbush = Flatbush::new(degree, rectangles);

            group.bench_function(
                BenchmarkId::new(format!("flatbush_query.{}", poly_idx), degree),
                |b| {
                    // for coords in &positions_list {
                    b.iter(|| {
                        for &rect in &query_rects {
                            black_box(flatbush.query_rect(rect));
                        }
                    })
                },
            );
        }
    }

    group.finish();
}

criterion_group!(benches, query_benchmark);

criterion_main!(benches);
