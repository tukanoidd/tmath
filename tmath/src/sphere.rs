macro_rules! hit_func_impl {
    ($self:expr, $r:expr, $t_min:expr, $t_max:expr, $rec:expr) => {
        let oc = $r.orig - $self.center;
        let a = $r.dir.magnitude_sq();
        let half_b = oc | $r.dir;
        let c = oc.magnitude_sq() - $self.radius * $self.radius;
        let discriminant = half_b * half_b - a * c;

        if discriminant < 0.0 {
            return false;
        } else {
            let sqrtd = discriminant.sqrt();
            let mut root = (-half_b - sqrtd) / a;

            if root < $t_min || $t_max < root {
                root = (-half_b + sqrtd) / a;

                if root < $t_min || $t_max < root {
                    return false;
                }
            }

            $rec.t = root;
            $rec.p = $r.at($rec.t);

            let outward_normal = ($rec.p - $self.center) / $self.radius;
            $rec.set_face_normal($r, &outward_normal);

            return true;
        }
    };
}

macro_rules! decl_spheres {
    ($sphere:ident => [$(($var_ty:ty, $variant:ident)),*]) => {
        #[derive(Default, Debug, Copy, Clone, PartialEq)]
        pub struct $sphere {
            pub center: crate::vector::Point3,
            pub radius: f32,
        }

        impl $sphere {
            #[inline]
            pub fn new(center: crate::vector::Point3, radius: f32) -> Self {
                Self { center, radius }
            }
        }

        impl crate::hittable::Hittable for $sphere {
            fn hit(
                &self,
                r: &crate::ray::Ray,
                t_min: f32,
                t_max: f32,
                rec: &mut crate::hittable::HitRecord
            ) -> bool {
                hit_func_impl!(self, r, t_min, t_max, rec);
            }
        }

        paste::paste! {
            $(
                #[derive(Default, Debug, Copy, Clone, PartialEq)]
                pub struct [< $sphere $variant >] {
                    pub center: crate::vector::[< Point3 $variant>],
                    pub radius: $var_ty,
                }

                impl [< $sphere $variant >] {
                    #[inline]
                    pub fn new(center: crate::vector::[< Point3 $variant >], radius: $var_ty) -> Self {
                        Self { center, radius }
                    }
                }

                impl crate::hittable::[< Hittable $variant >] for [< $sphere $variant >] {
                    fn hit(
                        &self,
                        r: &crate::ray::[< Ray $variant >],
                        t_min: $var_ty,
                        t_max: $var_ty,
                        rec: &mut crate::hittable::[< HitRecord $variant >]
                    ) -> bool {
                        hit_func_impl!(self, r, t_min, t_max, rec);
                    }
                }
            )*
        }
    };
}

decl_spheres!(Sphere => [(f64, D)]);
