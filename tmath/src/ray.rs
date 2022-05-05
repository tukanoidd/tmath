use crate::vector::{Point3D, Vector3D};

#[derive(Default, Debug, Copy, Clone, PartialEq)]
pub struct Ray {
    pub orig: Point3D,
    pub dir: Vector3D,
}

impl Ray {
    #[inline]
    pub fn new(origin: Point3D, direction: Vector3D) -> Self {
        Self {
            orig: origin,
            dir: direction,
        }
    }

    #[inline]
    pub fn at(&self, t: f64) -> Point3D {
        self.orig + t * self.dir
    }
}
