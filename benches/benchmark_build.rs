mod utils;

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use rtree_performance::{PackedRTree, PackedRTreeUnsorted};

use utils::{get_positions_list, make_rectangles_list};

pub fn construction_benchmark(c: &mut Criterion) {
    let name = "africa";
    let positions_list = get_positions_list(name);
    let rectangles_list = make_rectangles_list(&positions_list);
    println!(
        "Benchmarking {} build: {} polygons",
        name,
        rectangles_list.len()
    );
    let mut group = c.benchmark_group(format!("build_{}", name));

    for (poly_idx, rectangles) in rectangles_list.iter().enumerate() {
        println!("Polygon {} has {} segments.", poly_idx, rectangles.len());
        for degree in [16].iter() {
            group.bench_with_input(
                BenchmarkId::new(format!("packed_rtree_unsorted_build.{}", poly_idx), degree),
                degree,
                |b, &d| {
                    b.iter(|| {
                        PackedRTreeUnsorted::new(d, rectangles.clone());
                    })
                },
            );
            // group.bench_with_input(
            //     BenchmarkId::new(format!("packed_rtree_auto_simd_build.{}", poly_idx), degree),
            //     degree,
            //     |b, &d| {
            //         b.iter(|| {
            //             PackedRTreeAutoSimd::new(d, rectangles);
            //         })
            //     },
            // );
            group.bench_with_input(
                BenchmarkId::new(format!("packed_rtree_hilbert_build.{}", poly_idx), degree),
                degree,
                |b, &d| {
                    b.iter(|| {
                        PackedRTree::new_hilbert(d, rectangles);
                    })
                },
            );
            group.bench_with_input(
                BenchmarkId::new(format!("packed_rtree_omt_build.{}", poly_idx), degree),
                degree,
                |b, &_d| {
                    b.iter(|| {
                        PackedRTree::new_omt(rectangles);
                    })
                },
            );
        }
    }
    group.finish();
}

criterion_group!(benches, construction_benchmark);
criterion_main!(benches);
