use paste::paste;

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
