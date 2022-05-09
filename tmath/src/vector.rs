use paste::paste;

use tmath_macros::{cast_all_vectors, Vector};

use crate::quaternion::Quaternion;

macro_rules! declare_vector_variants {
    (
        $len:literal;
        [$(
            ($($variant_ty:ty, $($variant_name:ident)*),+)
        ),+]
    ) => {
        paste! {
            $($(
                #[repr(C)]
                #[derive(Default, Copy, Clone, PartialEq, Vector)]
                pub struct [< Vector $len $($variant_name)* >]([$variant_ty; $len]);
            )*)*
        }
    };
}

macro_rules! declare_vectors {
    ($($len:literal),*) => {
        paste! {
            $(
                declare_vector_variants!(
                    $len;
                    [(f32, ), (f64, D), (i32, I), (i64, L), (u32, U), (u64, UL)]
                );
            )*
        }
    };
}

declare_vectors!(2, 3, 4);

cast_all_vectors![
    (Vector2, 2, f32),
    (Vector3, 3, f32),
    (Vector4, 4, f32),
    (Vector2D, 2, f64),
    (Vector3D, 3, f64),
    (Vector4D, 4, f64),
    (Vector2I, 2, i32),
    (Vector3I, 3, i32),
    (Vector4I, 4, i32),
    (Vector2L, 2, i64),
    (Vector3L, 3, i64),
    (Vector4L, 4, i64),
    (Vector2U, 2, u32),
    (Vector3U, 3, u32),
    (Vector4U, 4, u32),
    (Vector2UL, 2, u64),
    (Vector3UL, 3, u64),
    (Vector4UL, 4, u64)
];

impl Vector3 {
    pub fn rotate_about_angle_axis(&self, angle: f32, axis: &Self) -> Self {
        let q = Quaternion::new(angle, axis.normalized()).as_unit_norm();

        (q * Quaternion::new(0.0, *self) * -q).v
    }
}
