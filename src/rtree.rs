use crate::Rectangle;

pub trait RTree {
    fn new(degree: usize, rects: &[Rectangle]) -> Self;
    fn is_empty(&self) -> bool;
    fn height(&self) -> usize;
    fn degree(&self) -> usize;
    fn envelope(&self) -> Rectangle;
    fn query_rect(&self, rect: &Rectangle) -> Vec<usize>;
}
