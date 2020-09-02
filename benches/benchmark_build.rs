use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use rtree_performance::{Coordinate, Flatbush, PackedRTree, Rectangle, SegRTree};

use rtree_performance::from_wkt::{parse_wkt, Geometry};
use std::fs;
use std::path::Path;

pub fn construction_benchmark(c: &mut Criterion) {
    let rectangles_list = get_rectangles_list("africa");
    println!("Benchmarking {} polygons", rectangles_list.len());
    let mut group = c.benchmark_group("build_africa");

    for (poly_idx, rectangles) in rectangles_list.iter().enumerate() {
        println!("Polygon {} has {} segments.", poly_idx, rectangles.len());
        for degree in [8, 16].iter() {
            group.bench_with_input(
                BenchmarkId::new(format!("seg_rtree_build.{}", poly_idx), degree),
                degree,
                |b, &d| {
                    b.iter(|| {
                        SegRTree::new(d, rectangles);
                    })
                },
            );
            group.bench_with_input(
                BenchmarkId::new(format!("packed_rtree_build.{}", poly_idx), degree),
                degree,
                |b, &d| {
                    b.iter(|| {
                        PackedRTree::new(d, rectangles);
                    })
                },
            );
            group.bench_with_input(
                BenchmarkId::new(format!("flatbush_unsorted_build.{}", poly_idx), degree),
                degree,
                |b, &d| {
                    b.iter(|| {
                        Flatbush::new_unsorted(d, rectangles);
                    })
                },
            );
            // group.bench_with_input(
            //     BenchmarkId::new(format!("flatbush_sorted_build.{}", poly_idx), degree),
            //     degree,
            //     |b, &d| {
            //         b.iter(|| {
            //             Flatbush::new(d, rectangles);
            //         })
            //     },
            // );
        }
    }
    group.finish();
}

criterion_group!(benches, construction_benchmark);
criterion_main!(benches);

//// Utility functions

fn read_test_case(name: &str) -> Vec<Vec<Geometry>> {
    let filename = format!("benches/testdata/{}.wkt", name);
    let filepath = Path::new("/Users/jagill/dev/seg_rtree").join(Path::new(&filename));
    let contents = fs::read_to_string(Path::new(&filepath)).unwrap();

    contents
        .split("\n\n")
        .map(|f| parse_wkt(f).unwrap())
        .collect()
}

fn get_rectangles_list(name: &str) -> Vec<Vec<Rectangle>> {
    let positions_list: Vec<Vec<Coordinate>> = read_test_case(name)
        .into_iter()
        .map(|mut geoms| geoms.remove(0))
        .filter_map(|geom| match geom {
            Geometry::Polygon(poly) => Some(poly.shell),
            _ => None,
        })
        .collect();
    let rectangles_list: Vec<Vec<Rectangle>> = positions_list
        .iter()
        .map(|positions| {
            positions
                .windows(2)
                .map(|c| Rectangle::new(c[0], c[1]))
                .collect()
        })
        .collect();
    rectangles_list
}
