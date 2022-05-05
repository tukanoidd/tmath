use paste::paste;

macro_rules! decl_rays {
    ($($ray:ident => ($var_ty:ty, $variant:ident));*) => {
        #[derive(Default, Debug, Copy, Clone, PartialEq)]
        pub struct Ray {
            pub orig: crate::vector::Point3,
            pub dir: crate::vector::Vector3,
        }

        impl Ray {
            #[inline]
            pub fn new(origin: crate::vector::Point3, direction: crate::vector::Vector3) -> Self {
                Self {
                    orig: origin,
                    dir: direction,
                }
            }

            #[inline]
            pub fn at(&self, t: f32) -> crate::vector::Point3 {
                self.orig + t * self.dir
            }
        }

        paste! {
            $(
                #[derive(Default, Debug, Copy, Clone, PartialEq)]
                pub struct [< $ray $variant>] {
                    pub orig: crate::vector::[< Point3 $variant >],
                    pub dir: crate::vector::[< Vector3 $variant >],
                }

                impl [< $ray $variant>] {
                    #[inline]
                    pub fn new(
                        origin: crate::vector::[< Point3 $variant >],
                        direction: crate::vector::[< Vector3 $variant >]
                    ) -> Self {
                        Self {
                            orig: origin,
                            dir: direction,
                        }
                    }

                    #[inline]
                    pub fn at(&self, t: $var_ty) -> crate::vector::[< Point3 $variant >] {
                        self.orig + t * self.dir
                    }
                }
            )*
        }
    };
}

decl_rays![Ray => (f64, D)];
