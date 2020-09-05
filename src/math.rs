#[derive(Debug, Copy, Clone)]
pub struct BBox {
    min: glam::Vec3,
    max: glam::Vec3,
}

impl BBox {
    pub fn from_points<I>(points: I) -> Self 
        where I: Iterator<Item = glam::Vec3> {
            let (min, max) = points.fold((glam::Vec3::zero(), glam::Vec3::zero()), |(mut min, mut max), p| {
                min.set_x(f32::min(min.x(), p.x()));
                min.set_y(f32::min(min.y(), p.y()));
                min.set_z(f32::min(min.z(), p.z()));

                max.set_x(f32::max(max.x(), p.x()));
                max.set_y(f32::max(max.y(), p.y()));
                max.set_z(f32::max(max.z(), p.z()));
                
                (min, max)
            });

            Self { min, max }
    }

    pub fn min(self) -> glam::Vec3 {
        self.min
    }

    pub fn max(self) -> glam::Vec3 {
        self.max
    }
}
