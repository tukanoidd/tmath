use paste::paste;

use crate::quaternion::Quaternion;
use tmath_macros::Vector;

macro_rules! declare_vector_variants {
    (
        $len:literal;
        [$(
            ($($variant_name:ident, $variant_ty:ty),+)
        ),+]
    ) => {
        paste! {
            $($(
                #[repr(C)]
                #[derive(Default, Copy, Clone, PartialEq, Vector)]
                pub struct [< Vector $len $variant_name >]([$variant_ty; $len]);
            )*)*
        }
    };
}

macro_rules! declare_vectors {
    ($($len:literal),*) => {
        paste! {
            $(
                #[repr(C)]
                #[derive(Default, Copy, Clone, PartialEq, Vector)]
                pub struct [< Vector $len >]([f32; $len]);

                declare_vector_variants!(
                    $len;
                    [(D, f64), (I, i32), (L, i64), (U, u32), (UL, u64)]
                );
            )*
        }
    };
}

declare_vectors!(2, 3, 4);

// TODO(tukanoidd): implement every possible cast for vectors (macros)

impl Vector3 {
    pub fn rotate_about_angle_axis(&self, angle: f32, axis: &Self) -> Self {
        let q = Quaternion::new(angle, axis.normalized()).as_unit_norm();

        (q * Quaternion::new(0.0, *self) * -q).v
    }
}
