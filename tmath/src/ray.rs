macro_rules! decl_rays {
    ($ray:ident => [$(($var_ty:ty, $($variant:ident)*)),*]) => {
        paste::paste! {
            $(
                #[derive(Default, Debug, Copy, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
                pub struct [< $ray $($variant)*>] {
                    pub orig: crate::vector::[< Point3 $($variant)* >],
                    pub dir: crate::vector::[< Vector3 $($variant)* >],
                    pub tm: $var_ty
                }

                impl [< $ray $($variant)*>] {
                    #[inline]
                    pub fn new(
                        origin: crate::vector::[< Point3 $($variant)* >],
                        direction: crate::vector::[< Vector3 $($variant)* >],
                        time: $var_ty,
                    ) -> Self {
                        Self {
                            orig: origin,
                            dir: direction,
                            tm: time
                        }
                    }

                    #[inline]
                    pub fn at(&self, t: $var_ty) -> crate::vector::[< Point3 $($variant)* >] {
                        self.orig + t * self.dir
                    }
                }
            )*
        }
    };
}

decl_rays!(Ray => [(f32, ), (f64, D)]);
