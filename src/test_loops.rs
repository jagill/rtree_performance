#[cfg(target_arch = "x86")]
use std::arch::x86::*;
#[cfg(target_arch = "x86_64")]
use std::arch::x86_64::*;

pub struct Rectangle {
    pub x_min: f64,
    pub y_min: f64,
    pub x_max: f64,
    pub y_max: f64,
}

impl Rectangle {
    pub fn intersects(&self, other: &Rectangle) -> bool {
        self.x_min <= other.x_max
            && self.x_max >= other.x_min
            && self.y_min <= other.y_max
            && self.y_max >= other.y_min
    }
    pub fn intersects2(&self, other: &Rectangle) -> bool {
        (self.x_min <= other.x_max)
            & (self.x_max >= other.x_min)
            & (self.y_min <= other.y_max)
            & (self.y_max >= other.y_min)
    }
}

#[repr(C)]
pub union BBox {
    scalars: [f64; 4],
    vectors: [__m128d; 2],
}

#[repr(align(32))]
#[derive(Copy, Clone)]
pub struct BBox2([f64; 4]);

pub fn rectangle_loop(query_rect: Rectangle, tree_rects: &[Rectangle]) -> usize {
    let mut count = 0;
    for tree_rect in tree_rects {
        if query_rect.intersects(tree_rect) {
            count += 1;
        }
    }

    count
}

pub fn rectangle_loop2(query_rect: Rectangle, tree_rects: &[Rectangle]) -> usize {
    let mut count = 0;
    for tree_rect in tree_rects {
        if query_rect.intersects2(tree_rect) {
            count += 1;
        }
    }

    count
}

pub fn bbox_loop(query_bbox: BBox, tree_rects: &[BBox]) -> usize {
    let mut count = 0;
    for tree_bbox in tree_rects {
        unsafe {
            if tree_bbox.scalars[0] <= query_bbox.scalars[0]
                && tree_bbox.scalars[1] <= query_bbox.scalars[1]
                && tree_bbox.scalars[2] <= query_bbox.scalars[2]
                && tree_bbox.scalars[3] <= query_bbox.scalars[3]
            {
                count += 1;
            }
        }
    }

    count
}

pub fn bbox_loop2(query_bbox: BBox, tree_rects: &[BBox]) -> usize {
    let mut count = 0;
    for tree_bbox in tree_rects {
        unsafe {
            if (tree_bbox.scalars[0] <= query_bbox.scalars[0])
                & (tree_bbox.scalars[1] <= query_bbox.scalars[1])
                & (tree_bbox.scalars[2] <= query_bbox.scalars[2])
                & (tree_bbox.scalars[3] <= query_bbox.scalars[3])
            {
                count += 1;
            }
        }
    }

    count
}

pub fn bbox_loop3(query_bbox: BBox2, tree_rects: &[BBox2]) -> usize {
    let mut count = 0;
    for tree_bbox in tree_rects {
        if tree_bbox.0[0] <= query_bbox.0[0]
            && tree_bbox.0[1] <= query_bbox.0[1]
            && tree_bbox.0[2] <= query_bbox.0[2]
            && tree_bbox.0[3] <= query_bbox.0[3]
        {
            count += 1;
        }
    }

    count
}

pub fn bbox_loop4(query_bbox: BBox2, tree_rects: &[BBox2]) -> usize {
    let mut count = 0;
    for tree_bbox in tree_rects {
        if (tree_bbox.0[0] <= query_bbox.0[0])
            & (tree_bbox.0[1] <= query_bbox.0[1])
            & (tree_bbox.0[2] <= query_bbox.0[2])
            & (tree_bbox.0[3] <= query_bbox.0[3])
        {
            count += 1;
        }
    }

    count
}
