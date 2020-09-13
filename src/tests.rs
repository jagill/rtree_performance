use crate::{PackedRTreeAutoSimd, PackedRTreeUnsorted, RTree, Rectangle, SortedPackedRTree};

#[test]
fn test_empty_rtree() {
    assert_empty_rtree(PackedRTreeUnsorted::new_empty());
    assert_empty_rtree(PackedRTreeAutoSimd::new_empty());
    assert_empty_rtree(SortedPackedRTree::new_hilbert(2, &Vec::<Rectangle>::new()));
    assert_empty_rtree(SortedPackedRTree::new_omt(2, &Vec::<Rectangle>::new()));
}

fn assert_empty_rtree(tree: impl RTree) {
    let r = Rectangle {
        x_min: -10.,
        y_min: -5.,
        x_max: 1.,
        y_max: 5.,
    };
    assert!(tree.is_empty());
    assert!(tree.envelope().is_empty());
    assert_eq!(tree.height(), 0);
    assert_eq!(tree.query_rect(&r), Vec::<usize>::new());
}

fn _assert_queries(max_index: usize, tree: &PackedRTreeAutoSimd, rects: &[Rectangle]) {
    #[allow(clippy::needless_range_loop)]
    for i in 0..=max_index {
        assert_eq!(tree.query_rect(&rects[i]), vec![i]);
    }
}

#[test]
fn test_build_tree() {
    assert_build_tree(PackedRTreeUnsorted::new);
    assert_build_tree(PackedRTreeAutoSimd::new);
    assert_build_tree(SortedPackedRTree::new_hilbert);
    assert_build_tree(SortedPackedRTree::new_omt);
}

fn assert_build_tree<R, C>(constructor: C)
where
    R: RTree,
    C: FnOnce(usize, &[Rectangle]) -> R,
{
    let degree = 4;
    let e0 = Rectangle::new((7.0, 44.).into(), (8., 48.).into());
    let e1 = Rectangle::new((25., 48.).into(), (35., 55.).into());
    let e2 = Rectangle::new((98., 46.).into(), (99., 56.).into());
    let e3 = Rectangle::new((58., 65.).into(), (73., 79.).into());
    let e4 = Rectangle::new((43., 40.).into(), (44., 45.).into());
    let e5 = Rectangle::new((97., 87.).into(), (100., 91.).into());
    let e6 = Rectangle::new((92., 46.).into(), (108., 57.).into());
    let e7 = Rectangle::new((7.1, 48.).into(), (10., 56.).into());
    let envs = vec![e0, e1, e2, e3, e4, e5, e6, e7];

    let rtree = constructor(degree, &envs);
    let query_point = (43., 43.).into();
    let query_rect = Rectangle::new(query_point, query_point);

    let results = rtree.query_rect(&query_rect);
    assert_eq!(results, vec![4]);
}

fn get_envelopes() -> Vec<Rectangle> {
    #[rustfmt::skip]
        let rects: Vec<f64> = vec![
             8, 62, 11, 66,
            57, 17, 57, 19,
            76, 26, 79, 29,
            36, 56, 38, 56,
            92, 77, 96, 80,
            87, 70, 90, 74,
            43, 41, 47, 43,
             0, 58,  2, 62,
            76, 86, 80, 89,
            27, 13, 27, 15,
            71, 63, 75, 67,
            25,  2, 27,  2,
            87,  6, 88,  6,
            22, 90, 23, 93,
            22, 89, 22, 93,
            57, 11, 61, 13,
            61, 55, 63, 56,
            17, 85, 21, 87,
            33, 43, 37, 43,
             6,  1,  7,  3,
            80, 87, 80, 87,
            23, 50, 26, 52,
            58, 89, 58, 89,
            12, 30, 15, 34,
            32, 58, 36, 61,
            41, 84, 44, 87,
            44, 18, 44, 19,
            13, 63, 15, 67,
            52, 70, 54, 74,
            57, 59, 58, 59,
            17, 90, 20, 92,
            48, 53, 52, 56,
             2, 68, 92, 72,
            26, 52, 30, 52,
            56, 23, 57, 26,
            88, 48, 88, 48,
            66, 13, 67, 15,
             7, 82,  8, 86,
            46, 68, 50, 68,
            37, 33, 38, 36,
             6, 15,  8, 18,
            85, 36, 89, 38,
            82, 45, 84, 48,
            12,  2, 16,  3,
            26, 15, 26, 16,
            55, 23, 59, 26,
            76, 37, 79, 39,
            86, 74, 90, 77,
            16, 75, 18, 78,
            44, 18, 45, 21,
            52, 67, 54, 71,
            59, 78, 62, 78,
            24,  5, 24,  8,
            64, 80, 64, 83,
            66, 55, 70, 55,
             0, 17,  2, 19,
            15, 71, 18, 74,
            87, 57, 87, 59,
             6, 34,  7, 37,
            34, 30, 37, 32,
            51, 19, 53, 19,
            72, 51, 73, 55,
            29, 45, 30, 45,
            94, 94, 96, 95,
             7, 22, 11, 24,
            86, 45, 87, 48,
            33, 62, 34, 65,
            18, 10, 21, 14,
            64, 66, 67, 67,
            64, 25, 65, 28,
            27,  4, 31,  6,
            84,  4, 85,  5,
            48, 80, 50, 81,
             1, 61,  3, 61,
            71, 89, 74, 92,
            40, 42, 43, 43,
            27, 64, 28, 66,
            46, 26, 50, 26,
            53, 83, 57, 87,
            14, 75, 15, 79,
            31, 45, 34, 45,
            89, 84, 92, 88,
            84, 51, 85, 53,
            67, 87, 67, 89,
            39, 26, 43, 27,
            47, 61, 47, 63,
            23, 49, 25, 53,
            12,  3, 14,  5,
            16, 50, 19, 53,
            63, 80, 64, 84,
            22, 63, 22, 64,
            26, 66, 29, 66,
             2, 15,  3, 15,
            74, 77, 77, 79,
            64, 11, 68, 11,
            38,  4, 39,  8,
            83, 73, 87, 77,
            85, 52, 89, 56,
            74, 60, 76, 63,
            62, 66, 65, 67,
        ]
        .into_iter()
        .map(|v| v as f64)
        .collect();
    rects
        .chunks(4)
        .map(|r| Rectangle::new((r[0], r[1]).into(), (r[2], r[3]).into()))
        .collect()
}

fn assert_intersections<R, C>(constructor: C)
where
    R: RTree,
    C: FnOnce(usize, &[Rectangle]) -> R,
{
    let envelopes = get_envelopes();
    let tree = constructor(16, &envelopes);
    let query_rect = Rectangle::new((40., 40.).into(), (60., 60.).into());

    let brute_results = find_brute_intersections(&query_rect, &envelopes);
    let mut rtree_results = tree.query_rect(&query_rect);
    rtree_results.sort();
    assert_eq!(rtree_results, brute_results);
}

#[test]
fn test_intersection_candidates() {
    assert_intersections(PackedRTreeUnsorted::new);
    assert_intersections(PackedRTreeAutoSimd::new);
    assert_intersections(SortedPackedRTree::new_hilbert);
    assert_intersections(SortedPackedRTree::new_omt);
}

// #[test]
// fn test_self_intersection_unsorted() {
//     let envelopes: Vec<Rectangle> = get_envelopes();
//     let f = Flatbush::new_unsorted(16, &envelopes);

//     let brute_results = find_brute_self_intersections(&envelopes);
//     let mut rtree_results = f.query_self_intersections();
//     rtree_results.sort();
//     assert_eq!(rtree_results, brute_results);
// }

// #[test]
// fn test_rtree_intersection_unsorted() {
//     let mut envelopes1 = get_envelopes();
//     let n_envs = envelopes1.len();
//     let envelopes2 = envelopes1.split_off(2 * envelopes1.len() / 3);
//     assert_eq!(envelopes1.len() + envelopes2.len(), n_envs);

//     let f1 = Flatbush::new_unsorted(&envelopes1, 16);
//     let f2 = Flatbush::new_unsorted(&envelopes2, 16);
//     let mut rtree_results = f1.find_other_rtree_intersection_candidates(&f2);
//     rtree_results.sort();
//     let brute_results = find_brute_cross_intersections(&envelopes1, &envelopes2);
//     assert_eq!(rtree_results, brute_results);
// }

// #[test]
// fn test_rtree_intersection_with_empty() {
//     let envelopes1 = get_envelopes();
//     let f1 = Flatbush::new(&envelopes1, 16);
//     let f2 = Flatbush::new_empty();
//     let rtree_results = f1.find_other_rtree_intersection_candidates(&f2);
//     assert_eq!(rtree_results, vec![]);
// }

fn find_brute_intersections(query_rect: &Rectangle, envelopes: &[Rectangle]) -> Vec<usize> {
    envelopes
        .iter()
        .enumerate()
        .filter(|(_, e)| e.intersects(query_rect))
        .map(|(i, _)| i)
        .collect()
}

// fn find_brute_self_intersections(envelopes: &[Rectangle]) -> Vec<(usize, usize)> {
//     let mut results = Vec::new();
//     for (i1, e1) in envelopes.iter().copied().enumerate() {
//         for (i2, e2) in envelopes.iter().copied().enumerate() {
//             if i1 >= i2 {
//                 continue;
//             }
//             if !e1.intersects(e2) {
//                 continue;
//             }
//             results.push((i1, i2))
//         }
//     }
//     results
// }

// fn find_brute_cross_intersections(
//     envelopes1: &[Rectangle],
//     envelopes2: &[Rectangle],
// ) -> Vec<(usize, usize)> {
//     type EnumEnv = (usize, Rectangle);
//     let envelopes1: Vec<EnumEnv> = envelopes1.iter().copied().enumerate().collect();
//     let envelopes2: Vec<EnumEnv> = envelopes2.iter().copied().enumerate().collect();
//     iproduct!(envelopes1, envelopes2)
//         .filter(|((_, e1), (_, e2))| e1.intersects(*e2))
//         .map(|((i1, _), (i2, _))| (i1, i2))
//         .collect()
// }
