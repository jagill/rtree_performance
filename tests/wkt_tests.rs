mod test_utils;

use rtree_performance::{PackedRTree, RTree, Rectangle};
use test_utils::{get_positions_list, get_random_rects, make_rectangles_list, read_test_case};

fn get_results_brute_force(query: &Rectangle, rectangles: &[Rectangle]) -> Vec<Rectangle> {
    let mut result: Vec<Rectangle> = rectangles
        .iter()
        .copied()
        .filter(|r| query.intersects(r))
        .collect();
    result.sort_unstable_by(order_rectangles);
    result
}

fn get_results_rtree(
    query: &Rectangle,
    rectangles: &[Rectangle],
    rtree: &mut impl RTree,
) -> Vec<Rectangle> {
    let mut result: Vec<Rectangle> = rtree
        .query_rect(&query)
        .iter()
        .map(|&i| rectangles[i])
        .collect();
    result.sort_unstable_by(order_rectangles);
    result
}

#[test]
fn test_wkt_polygons() {
    for test_case in &["africa", "europe", "plane", "uk", "usa-lower48", "world"] {
        let positions_list = get_positions_list(test_case);
        let mut rectangles_list = make_rectangles_list(&positions_list);
        println!("{} has {} polygons", test_case, positions_list.len());
        let geometries = read_test_case(test_case);
        println!("{} has {} geometries", test_case, geometries.len());
        let mut total_coords = 0;
        for (idx, coords) in positions_list.iter().enumerate() {
            let num_shell_coords = coords.len();
            total_coords += num_shell_coords;
            println!("Polygon {} {} nCoords {}", test_case, idx, num_shell_coords);
            let rectangles = &mut rectangles_list[idx];
            let universe = Rectangle::of(rectangles);
            let mut hilbert_rtree = PackedRTree::new_hilbert(8, rectangles);
            let mut omt_rtree = PackedRTree::new_omt(rectangles);

            // let mut omt_leaves: Vec<Rectangle> = omt_rtree
            //     .raw_rtree
            //     .leaves()
            //     .iter()
            //     .copied()
            //     .filter(|r| !r.is_empty())
            //     .collect();
            // rectangles.sort_unstable_by(order_rectangles);
            // omt_leaves.sort_unstable_by(order_rectangles);
            // assert_eq!(rectangles, &omt_leaves);

            for (query_idx, query) in get_random_rects(universe, 100, 192).iter().enumerate() {
                if query_idx != 23 {
                    continue;
                }
                println!(
                    "Trying {} {}: query {} {:?}",
                    test_case, idx, query_idx, query
                );
                let brute_results = get_results_brute_force(query, rectangles);
                let hilbert_results = get_results_rtree(query, rectangles, &mut hilbert_rtree);
                let omt_results = get_results_rtree(query, rectangles, &mut omt_rtree);
                println!(
                    "Num brute results {}  Num omt results {}",
                    brute_results.len(),
                    omt_results.len()
                );
                assert_eq!(
                    brute_results, hilbert_results,
                    "Hilbert failed on {} {}: query {:?}",
                    test_case, idx, query
                );
                assert_eq!(
                    brute_results, omt_results,
                    "OMT failed on {} {}: query {} {:?}",
                    test_case, idx, query_idx, query
                );
            }
        }
        println!("TestCase {} nCoords {}", test_case, total_coords);
    }
    // assert!(false);
}

// fn check_results(brute_results: &[Rectangle], rtree_results: &[Rectangle]) {

// }

use std::cmp::Ordering;

fn order_rectangles(a: &Rectangle, b: &Rectangle) -> Ordering {
    match (a.is_empty(), b.is_empty()) {
        (true, true) => Ordering::Equal,
        (true, false) => Ordering::Greater,
        (false, true) => Ordering::Less,
        (false, false) => a
            .x_min
            .partial_cmp(&b.x_min)
            .unwrap()
            .then(a.x_max.partial_cmp(&b.x_max).unwrap())
            .then(a.y_min.partial_cmp(&b.y_min).unwrap())
            .then(a.y_max.partial_cmp(&b.y_max).unwrap()),
    }
}
