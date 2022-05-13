macro_rules! decl_rays {
    ($ray:ident => [$(($var_ty:ty, $($variant:ident)*)),*]) => {
        paste::paste! {
            $(
                #[derive(Default, Debug, Copy, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
                pub struct [< $ray $($variant)*>] {
                    pub origin: crate::vector::[< Point3 $($variant)* >],
                    pub direction: crate::vector::[< Vector3 $($variant)* >],
                    pub time: $var_ty
                }

                impl [< $ray $($variant)*>] {
                    #[inline]
                    pub fn new(
                        origin: crate::vector::[< Point3 $($variant)* >],
                        direction: crate::vector::[< Vector3 $($variant)* >],
                        time: $var_ty,
                    ) -> Self {
                        Self {
                            origin,
                            direction,
                            time
                        }
                    }

                    #[inline]
                    pub fn at(&self, t: $var_ty) -> crate::vector::[< Point3 $($variant)* >] {
                        self.origin + t * self.direction
                    }
                }
            )*
        }
    };
}

decl_rays!(Ray => [(f32, ), (f64, D)]);
