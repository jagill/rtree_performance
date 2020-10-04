use rand::rngs::SmallRng;
use rand::{Rng, SeedableRng};
use std::fs;
use std::path::Path;

use rtree_performance::from_wkt::{parse_wkt, Geometry};
use rtree_performance::utils::rectangles_from_coordinates;
use rtree_performance::{Coordinate, Rectangle};

//// Utility functions

pub(crate) fn read_test_case(name: &str) -> Vec<Geometry> {
    let filename = format!("tests/testdata/{}.wkt", name);
    let filepath = Path::new("/Users/jagill/dev/rtree_performance").join(Path::new(&filename));
    let contents = fs::read_to_string(Path::new(&filepath)).unwrap();

    contents
        .split("\n\n")
        .map(|f| parse_wkt(f).unwrap())
        .flatten()
        .collect()
}

pub(crate) fn get_positions_list(name: &str) -> Vec<Vec<Coordinate>> {
    let positions_list: Vec<Vec<Coordinate>> = read_test_case(name)
        .into_iter()
        .take(5)
        .filter_map(|geom| match geom {
            Geometry::Polygon(poly) => Some(poly.shell),
            _ => None,
        })
        .collect();
    positions_list
}

pub(crate) fn make_rectangles_list(positions_list: &[Vec<Coordinate>]) -> Vec<Vec<Rectangle>> {
    let rectangles_list: Vec<Vec<Rectangle>> = positions_list
        .iter()
        .map(|coords| rectangles_from_coordinates(&coords))
        .collect();
    rectangles_list
}

pub(crate) fn get_random_points(rect: Rectangle, n: usize, seed: u64) -> Vec<Coordinate> {
    let mut rng = SmallRng::seed_from_u64(seed);
    let mut results = Vec::new();
    for _i in 0..n {
        results.push(Coordinate::new(
            rng.gen_range(rect.x_min, rect.x_max),
            rng.gen_range(rect.y_min, rect.y_max),
        ));
    }

    results
}

pub(crate) fn get_random_rects(rect: Rectangle, n: usize, seed: u64) -> Vec<Rectangle> {
    let mut rng = SmallRng::seed_from_u64(seed);
    let mut results = Vec::new();
    for _i in 0..n {
        results.push(Rectangle::new(
            Coordinate::new(
                rng.gen_range(rect.x_min, rect.x_max),
                rng.gen_range(rect.y_min, rect.y_max),
            ),
            Coordinate::new(
                rng.gen_range(rect.x_min, rect.x_max),
                rng.gen_range(rect.y_min, rect.y_max),
            ),
        ));
    }

    results
}
