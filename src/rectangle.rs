use crate::Coordinate;

#[derive(Copy, Clone, Debug)]
pub struct Rectangle {
    pub x_min: f64,
    pub y_min: f64,
    pub x_max: f64,
    pub y_max: f64,
}

impl PartialEq for Rectangle {
    fn eq(&self, other: &Self) -> bool {
        if self.is_empty() {
            other.is_empty()
        } else {
            self.x_min == other.x_min
                && self.y_min == other.y_min
                && self.x_max == other.x_max
                && self.y_max == other.y_max
        }
    }
}

pub trait HasEnvelope {
    fn envelope(&self) -> Rectangle;
}

impl HasEnvelope for Coordinate {
    fn envelope(&self) -> Rectangle {
        Rectangle {
            x_min: self.x,
            y_min: self.y,
            x_max: self.x,
            y_max: self.y,
        }
    }
}

impl HasEnvelope for Rectangle {
    fn envelope(&self) -> Rectangle {
        *self
    }
}

impl Rectangle {
    pub fn new(p1: Coordinate, p2: Coordinate) -> Self {
        Rectangle {
            x_min: p1.x.min(p2.x),
            y_min: p1.y.min(p2.y),
            x_max: p1.x.max(p2.x),
            y_max: p1.y.max(p2.y),
        }
    }

    pub fn new_empty() -> Self {
        Rectangle {
            x_min: f64::NAN,
            y_min: f64::NAN,
            x_max: f64::NAN,
            y_max: f64::NAN,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.x_min.is_nan() || self.y_min.is_nan() || self.x_max.is_nan() || self.y_max.is_nan()
    }

    pub fn of<T: HasEnvelope>(items: &[T]) -> Self {
        items.iter().fold(Rectangle::new_empty(), |mut s, r| {
            s.expand(r.envelope());
            s
        })
    }

    pub fn center(&self) -> Coordinate {
        Coordinate {
            x: (self.x_max + self.x_min) / 2.,
            y: (self.y_max + self.y_min) / 2.,
        }
    }

    pub fn intersects<T: HasEnvelope>(&self, item: T) -> bool {
        let other = item.envelope();
        self.x_min <= other.x_max
            && self.x_max >= other.x_min
            && self.y_min <= other.y_max
            && self.y_max >= other.y_min
    }

    pub fn contains<T: HasEnvelope>(&self, item: T) -> bool {
        let other = item.envelope();
        self.x_min <= other.x_min
            && self.x_max >= other.x_max
            && self.y_min <= other.y_min
            && self.y_max >= other.y_max
    }

    pub fn merge<T: HasEnvelope>(&self, item: T) -> Self {
        let rect = item.envelope();
        Rectangle {
            x_min: self.x_min.min(rect.x_min),
            y_min: self.y_min.min(rect.y_min),
            x_max: self.x_max.max(rect.x_max),
            y_max: self.y_max.max(rect.y_max),
        }
    }

    pub fn expand<T: HasEnvelope>(&mut self, item: T) {
        let rect = item.envelope();
        self.x_min = self.x_min.min(rect.x_min);
        self.y_min = self.y_min.min(rect.y_min);
        self.x_max = self.x_max.max(rect.x_max);
        self.y_max = self.y_max.max(rect.y_max);
    }
}
