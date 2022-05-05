macro_rules! decl_hit_records_hittables {
    (($hit_record:ident, $hittable:ident) => [$(($var_ty:ty, $variant:ident)),*]) => {
        #[derive(Default, Debug, Copy, Clone, PartialEq)]
        pub struct $hit_record {
            pub p: crate::vector::Point3,
            pub normal: crate::vector::Vector3,
            pub t: f32,
            pub front_face: bool,
        }

        impl $hit_record {
            pub fn set_face_normal(&mut self, r: &crate::ray::Ray, outward_normal: &crate::vector::Vector3) {
                self.front_face = (r.dir | outward_normal) < 0.0;
                self.normal = if self.front_face { *outward_normal } else { -outward_normal };
            }
        }

        pub trait $hittable {
            fn hit(&self, r: &crate::ray::Ray, t_min: f32, t_max: f32, rec: &mut HitRecord) -> bool;
        }

        paste::paste! {
            $(
                #[derive(Default, Debug, Copy, Clone, PartialEq)]
                pub struct [< $hit_record $variant >] {
                    pub p: crate::vector::[< Point3 $variant >],
                    pub normal: crate::vector::[< Vector3 $variant >],
                    pub t: $var_ty,
                    pub front_face: bool,
                }

                impl [< $hit_record $variant >] {
                    pub fn set_face_normal(
                        &mut self,
                        r: &crate::ray::[< Ray $variant >],
                        outward_normal: &crate::vector::[< Vector3 $variant>]
                    ) {
                        self.front_face = (r.dir | outward_normal) < 0.0;
                        self.normal = if self.front_face { *outward_normal } else { -outward_normal };
                    }
                }

                pub trait [< $hittable $variant >] {
                    fn hit(
                        &self,
                        r: &crate::ray::[< Ray $variant >],
                        t_min: $var_ty,
                        t_max: $var_ty,
                        rec: &mut [< HitRecord $variant>]
                    ) -> bool;
                }
            )*
        }
    };
}

decl_hit_records_hittables!((HitRecord, Hittable) => [(f64, D)]);
