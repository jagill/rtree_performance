mod utils;

use criterion::{criterion_group, criterion_main, BatchSize, BenchmarkId, Criterion};

use rtree_performance::{PackedRTree, PackedRTreeUnsorted, RTree, Rectangle};
use utils::{get_positions_list, get_random_points, make_rectangles_list};

pub fn query_benchmark(c: &mut Criterion) {
    let name = "africa";
    let positions_list = get_positions_list(name);
    let rectangles_list = make_rectangles_list(&positions_list);
    println!(
        "Benchmarking {} query: {} polygons",
        name,
        rectangles_list.len()
    );
    let mut group = c.benchmark_group(format!("query_{}", name));

    for (poly_idx, rectangles) in rectangles_list.iter().enumerate() {
        let query_rects: Vec<_> = get_random_points(Rectangle::of(rectangles), 1000, 342)
            .into_iter()
            .map(|p| Rectangle::new(p, p))
            .collect();
        println!("Polygon {} has {} segments.", poly_idx, rectangles.len());
        // for &degree in [8, 16].iter() {
        for &degree in [8].iter() {
            let rtree_native = PackedRTreeUnsorted::new(degree, rectangles.clone());
            // let rtree_auto_simd = PackedRTreeAutoSimd::new(degree, rectangles);
            let rtree_hilbert = PackedRTree::new_hilbert(degree, rectangles);

            group.bench_function(
                BenchmarkId::new(format!("packed_rtree_unsorted_query.{}", poly_idx), degree),
                |b| {
                    let mut query_iter = query_rects.iter().cycle();
                    b.iter_batched(
                        move || query_iter.next().unwrap(),
                        |probe| rtree_native.query_rect(probe),
                        BatchSize::SmallInput,
                    )
                },
            );

            // group.bench_function(
            //     BenchmarkId::new(format!("packed_rtree_auto_simd_query.{}", poly_idx), degree),
            //     |b| {
            // let mut query_iter = query_rects.iter().cycle();
            // b.iter_batched(
            //     move || query_iter.next().unwrap(),
            //     |probe| rtree_omt.query_rect(probe),
            //     BatchSize::SmallInput,
            // )
            //     },
            // );

            group.bench_function(
                BenchmarkId::new(format!("packed_rtree_hilbert_query.{}", poly_idx), degree),
                |b| {
                    let mut query_iter = query_rects.iter().cycle();
                    b.iter_batched(
                        move || query_iter.next().unwrap(),
                        |probe| rtree_hilbert.query_rect(probe),
                        BatchSize::SmallInput,
                    )
                },
            );

            let rtree_omt = PackedRTree::new_omt(rectangles);
            group.bench_function(
                BenchmarkId::new(format!("packed_rtree_omt_query.{}", poly_idx), degree),
                |b| {
                    let mut query_iter = query_rects.iter().cycle();
                    b.iter_batched(
                        move || query_iter.next().unwrap(),
                        |probe| rtree_omt.query_rect(probe),
                        BatchSize::SmallInput,
                    )
                },
            );
        }
    }

    group.finish();
}

criterion_group!(benches, query_benchmark);

criterion_main!(benches);
