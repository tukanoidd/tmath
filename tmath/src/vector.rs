use std::ops::{Add, DivAssign, Mul, Sub};

use num_traits::{real::Real, Zero};

#[macro_export]
macro_rules! vector {
    ($($member:expr),*) => {
	    $crate::vector::Vector::new([$($member),*])
    };
}

pub type Vector1<T> = Vector<1, T>;
pub type Vector2<T> = Vector<2, T>;
pub type Vector3<T> = Vector<3, T>;
pub type Vector4<T> = Vector<4, T>;

macro_rules! vec_types {
    ($($n:literal),*) => {
        paste::paste! {
            $(
                pub type [< Vector $n F >] = [< Vector $n >]<f32>;
                pub type [< Vector $n D >] = [< Vector $n >]<f64>;
                pub type [< Vector $n I >] = [< Vector $n >]<i32>;
                pub type [< Vector $n L >] = [< Vector $n >]<i64>;
                pub type [< Vector $n U >] = [< Vector $n >]<u32>;
                pub type [< Vector $n UL >] = [< Vector $n >]<u64>;
            )*
        }
    };
}

vec_types!(1, 2, 3, 4);

#[derive(Debug, Copy, Clone)]
pub struct Vector<const N: usize, T>(pub(crate) [T; N]);

impl<const N: usize, T> Default for Vector<N, T>
where
    T: Default + Copy,
{
    fn default() -> Self {
        Self::new_val(T::default())
    }
}

pub use equality::*;

mod equality {
    use super::*;

    impl<const N: usize, T> PartialEq for Vector<N, T>
    where
        T: PartialEq,
    {
        fn eq(&self, other: &Self) -> bool {
            self.0 == other.0
        }
    }

    impl<const N: usize, T> Eq for Vector<N, T> where T: Eq {}
}

impl<const N: usize, T> Vector<N, T> {
    pub const DIMENSIONS: usize = N;

    pub const fn dimensions(&self) -> usize {
        N
    }

    pub const fn new(val: [T; N]) -> Self {
        assert!(N > 0);
        Self(val)
    }

    #[inline]
    pub const fn new_val(val: T) -> Self
    where
        T: Copy,
    {
        Self::new([val; N])
    }

    pub fn length_squared(&self) -> T
    where
        T: Copy + Add<Output = T> + Mul<Output = T> + Zero,
    {
        self.0.iter().fold(T::zero(), |sum, &val| sum + val * val)
    }

    pub fn length(&self) -> T
    where
        T: Copy + Add<Output = T> + Mul<Output = T> + Real + Zero,
    {
        self.length_squared().sqrt()
    }

    pub fn dot(&self, other: &Self) -> T
    where
        T: Default + Copy + Add<Output = T> + Mul<Output = T>,
    {
        self.0
            .iter()
            .enumerate()
            .fold(T::default(), |sum, (i, &val)| sum + val * other[i])
    }

    pub fn normalize(&mut self)
    where
        T: Default + Real + Zero + DivAssign,
    {
        let len = self.length();

        if len > T::zero() {
            *self /= len;
        }
    }

    pub fn normalized(&self) -> Self
    where
        T: Default + Real + Zero,
    {
        let len = self.length();

        if len > T::zero() {
            self / len
        } else {
            *self
        }
    }
}

impl<T> Vector3<T> {
    pub fn cross(&self, other: &Self) -> Self
    where
        T: Copy + Sub<Output = T> + Mul<Output = T>,
    {
        Self([
            self.y() * other.z() - self.z() * other.y(),
            self.z() * other.x() - self.x() * other.z(),
            self.x() * other.y() - self.y() * other.x(),
        ])
    }
}

pub use deref::*;
mod deref {
    use super::*;

    use std::ops::{Deref, DerefMut};

    impl<T> Deref for Vector1<T> {
        type Target = T;

        fn deref(&self) -> &Self::Target {
            &self[0]
        }
    }

    impl<T> DerefMut for Vector1<T> {
        fn deref_mut(&mut self) -> &mut Self::Target {
            &mut self[0]
        }
    }
}

pub use casts::*;
mod casts {
    use super::*;

    impl<const N: usize, T> From<[T; N]> for Vector<N, T> {
        fn from(val: [T; N]) -> Self {
            Self(val)
        }
    }

    pub use vec1::*;
    mod vec1 {
        use super::*;

        impl<T> From<T> for Vector1<T> {
            fn from(x: T) -> Self {
                Self::new([x])
            }
        }

        impl<T> From<Vector2<T>> for Vector1<T>
        where
            T: Copy,
        {
            fn from(v2: Vector2<T>) -> Self {
                Self([v2[0]])
            }
        }

        impl<T> From<Vector3<T>> for Vector1<T>
        where
            T: Copy,
        {
            fn from(v3: Vector3<T>) -> Self {
                Self([v3[0]])
            }
        }

        impl<T> From<Vector4<T>> for Vector1<T>
        where
            T: Copy,
        {
            fn from(v4: Vector4<T>) -> Self {
                Self([v4[0]])
            }
        }
    }

    pub use vec2::*;
    mod vec2 {
        use super::*;

        impl<T> From<(T, T)> for Vector2<T> {
            fn from((x, y): (T, T)) -> Self {
                Self([x, y])
            }
        }

        impl<T> From<(Vector1<T>, Vector1<T>)> for Vector2<T>
        where
            T: Copy,
        {
            fn from((v1, v2): (Vector1<T>, Vector1<T>)) -> Self {
                Self([*v1, *v2])
            }
        }

        impl<T> From<Vector1<T>> for Vector2<T>
        where
            T: Default + Copy,
        {
            fn from(x: Vector1<T>) -> Self {
                Self([*x, T::default()])
            }
        }

        impl<T> From<Vector3<T>> for Vector2<T>
        where
            T: Copy,
        {
            fn from(v3: Vector3<T>) -> Self {
                Self([v3[0], v3[1]])
            }
        }

        impl<T> From<Vector4<T>> for Vector2<T>
        where
            T: Copy,
        {
            fn from(v4: Vector4<T>) -> Self {
                Self([v4[0], v4[1]])
            }
        }
    }

    pub use vec3::*;
    mod vec3 {
        use super::*;

        impl<T> From<(T, T, T)> for Vector3<T> {
            fn from((x, y, z): (T, T, T)) -> Self {
                Self([x, y, z])
            }
        }

        impl<T> From<(Vector1<T>, Vector1<T>, Vector1<T>)> for Vector3<T>
        where
            T: Copy,
        {
            fn from((v1, v2, v3): (Vector1<T>, Vector1<T>, Vector1<T>)) -> Self {
                Self([*v1, *v2, *v3])
            }
        }

        impl<T> From<(T, Vector2<T>)> for Vector3<T>
        where
            T: Copy,
        {
            fn from((x, yz): (T, Vector2<T>)) -> Self {
                Self([x, yz[0], yz[1]])
            }
        }

        impl<T> From<(Vector1<T>, Vector2<T>)> for Vector3<T>
        where
            T: Copy,
        {
            fn from((x, yz): (Vector1<T>, Vector2<T>)) -> Self {
                Self([*x, yz[0], yz[1]])
            }
        }

        impl<T> From<(Vector2<T>, T)> for Vector3<T>
        where
            T: Copy,
        {
            fn from((xy, z): (Vector2<T>, T)) -> Self {
                Self([xy[0], xy[1], z])
            }
        }

        impl<T> From<(Vector2<T>, Vector1<T>)> for Vector3<T>
        where
            T: Copy,
        {
            fn from((xy, z): (Vector2<T>, Vector1<T>)) -> Self {
                Self([xy[0], xy[1], *z])
            }
        }

        impl<T> From<Vector1<T>> for Vector3<T>
        where
            T: Default + Copy,
        {
            fn from(v1: Vector1<T>) -> Self {
                Self([*v1, T::default(), T::default()])
            }
        }

        impl<T> From<Vector2<T>> for Vector3<T>
        where
            T: Default + Copy,
        {
            fn from(v2: Vector2<T>) -> Self {
                Self([v2[0], v2[1], T::default()])
            }
        }

        impl<T> From<Vector4<T>> for Vector3<T>
        where
            T: Copy,
        {
            fn from(v4: Vector4<T>) -> Self {
                Self([v4[0], v4[1], v4[2]])
            }
        }
    }

    pub use vec4::*;
    mod vec4 {
        use super::*;

        impl<T> From<(T, T, T, T)> for Vector4<T> {
            fn from((x, y, z, w): (T, T, T, T)) -> Self {
                Self([x, y, z, w])
            }
        }

        impl<T> From<(Vector1<T>, Vector1<T>, Vector1<T>, Vector1<T>)> for Vector4<T>
        where
            T: Copy,
        {
            fn from((x, y, z, w): (Vector1<T>, Vector1<T>, Vector1<T>, Vector1<T>)) -> Self {
                Self([*x, *y, *z, *w])
            }
        }

        impl<T> From<(T, T, Vector2<T>)> for Vector4<T>
        where
            T: Copy,
        {
            fn from((x, y, zw): (T, T, Vector2<T>)) -> Self {
                Self([x, y, zw[0], zw[1]])
            }
        }

        impl<T> From<(Vector1<T>, Vector1<T>, Vector2<T>)> for Vector4<T>
        where
            T: Copy,
        {
            fn from((x, y, zw): (Vector1<T>, Vector1<T>, Vector2<T>)) -> Self {
                Self([*x, *y, zw[0], zw[1]])
            }
        }

        impl<T> From<(T, Vector3<T>)> for Vector4<T>
        where
            T: Copy,
        {
            fn from((x, yzw): (T, Vector3<T>)) -> Self {
                Self([x, yzw[0], yzw[1], yzw[2]])
            }
        }

        impl<T> From<(Vector1<T>, Vector3<T>)> for Vector4<T>
        where
            T: Copy,
        {
            fn from((x, yzw): (Vector1<T>, Vector3<T>)) -> Self {
                Self([*x, yzw[0], yzw[1], yzw[2]])
            }
        }

        impl<T> From<(Vector2<T>, T, T)> for Vector4<T>
        where
            T: Copy,
        {
            fn from((xy, z, w): (Vector2<T>, T, T)) -> Self {
                Self([xy[0], xy[1], z, w])
            }
        }

        impl<T> From<(Vector2<T>, Vector1<T>, Vector1<T>)> for Vector4<T>
        where
            T: Copy,
        {
            fn from((xy, z, w): (Vector2<T>, Vector1<T>, Vector1<T>)) -> Self {
                Self([xy[0], xy[1], *z, *w])
            }
        }

        impl<T> From<(Vector3<T>, T)> for Vector4<T>
        where
            T: Copy,
        {
            fn from((xyz, w): (Vector3<T>, T)) -> Self {
                Self([xyz[0], xyz[1], xyz[2], w])
            }
        }

        impl<T> From<(Vector3<T>, Vector1<T>)> for Vector4<T>
        where
            T: Copy,
        {
            fn from((xyz, w): (Vector3<T>, Vector1<T>)) -> Self {
                Self([xyz[0], xyz[1], xyz[2], *w])
            }
        }

        impl<T> From<(T, Vector2<T>, T)> for Vector4<T>
        where
            T: Copy,
        {
            fn from((x, yz, w): (T, Vector2<T>, T)) -> Self {
                Self([x, yz[0], yz[1], w])
            }
        }

        impl<T> From<(Vector1<T>, Vector2<T>, Vector1<T>)> for Vector4<T>
        where
            T: Copy,
        {
            fn from((x, yz, w): (Vector1<T>, Vector2<T>, Vector1<T>)) -> Self {
                Self([*x, yz[0], yz[1], *w])
            }
        }

        impl<T> From<Vector1<T>> for Vector4<T>
        where
            T: Default + Copy,
        {
            fn from(x: Vector1<T>) -> Self {
                Self([*x, T::default(), T::default(), T::default()])
            }
        }

        impl<T> From<Vector2<T>> for Vector4<T>
        where
            T: Default + Copy,
        {
            fn from(xy: Vector2<T>) -> Self {
                Self([xy[0], xy[1], T::default(), T::default()])
            }
        }

        impl<T> From<Vector3<T>> for Vector4<T>
        where
            T: Default + Copy,
        {
            fn from(xyz: Vector3<T>) -> Self {
                Self([xyz[0], xyz[1], xyz[2], T::default()])
            }
        }
    }
}

pub use indexing::*;
mod indexing {
    use super::*;

    use std::ops::{Index, IndexMut};

    impl<const N: usize, T> Index<usize> for Vector<N, T> {
        type Output = T;

        fn index(&self, index: usize) -> &Self::Output {
            &self.0[index]
        }
    }

    impl<const N: usize, T> IndexMut<usize> for Vector<N, T> {
        fn index_mut(&mut self, index: usize) -> &mut Self::Output {
            &mut self.0[index]
        }
    }
}

macro_rules! impl_vec_getters {
    ($n:literal; $(($var_name:ident, $index:literal)),*) => {
        paste::paste! {
            impl<T> Vector<$n, T> {
                $(
                    pub fn $var_name(&self) -> T where T: Copy {
                        self[$index]
                    }

                    pub fn [< $var_name _ref >](&self) -> &T {
                        &self[$index]
                    }

                    pub fn [< $var_name _mut >](&mut self) -> &mut T {
                        &mut self[$index]
                    }

                    pub fn [< set_ $var_name >](&mut self, val: T) {
                        self[$index] = val;
                    }
                )*
            }
        }
    };
}

impl_vec_getters!(1; (x, 0));
impl_vec_getters!(2; (x, 0), (y, 1));
impl_vec_getters!(3; (x, 0), (y, 1), (z, 2));
impl_vec_getters!(4; (x, 0), (y, 1), (z, 2), (w, 3));

pub use ops::*;
mod ops {
    use super::*;

    macro_rules! impl_vec_ops {
        ($($op:ident),*) => {
            paste::paste! {
                $(
                    pub use [< $op:lower >]::*;
                    mod [< $op:lower >] {
                        use super::*;

                        use std::ops::$op;

                        impl<const N: usize, T> $op<T> for Vector<N, T>
                        where
                            T: Copy + $op<Output = T>,
                        {
                            type Output = Self;

                            fn [< $op:lower >](self, rhs: T) -> Self::Output {
                                Vector::new(self.0.map(|x| x.[< $op:lower >](rhs)))
                            }
                        }

                        impl<'a, const N: usize, T> $op<T> for &'a Vector<N, T>
                        where
                            T: Copy + $op<Output = T>,
                        {
                            type Output = Vector<N, T>;

                            fn [< $op:lower >](self, rhs: T) -> Self::Output {
                                (*self).[< $op:lower >](rhs)
                            }
                        }

                        impl<'b, const N: usize, T> $op<&'b T> for Vector<N, T>
                        where
                            T: Copy + $op<Output = T>,
                        {
                            type Output = Vector<N, T>;

                            fn [< $op:lower >](self, rhs: &'b T) -> Self::Output {
                                self.[< $op:lower >](*rhs)
                            }
                        }

                        impl<'a, 'b, const N: usize, T> $op<&'b T> for &'a Vector<N, T>
                        where
                            T: Copy + $op<Output = T>,
                        {
                            type Output = Vector<N, T>;

                            fn [< $op:lower >](self, rhs: &'b T) -> Self::Output {
                                self.[< $op:lower >](*rhs)
                            }
                        }

                        impl<const N: usize, T> $op<Vector<N, T>> for Vector<N, T>
                        where
                            T: Copy + $op<Output = T>,
                        {
                            type Output = Vector<N, T>;

                            fn [< $op:lower >](self, rhs: Vector<N, T>) -> Self::Output {
                                let mut i = 0;

                                Self::new(self.0.map(|x| {
                                    let res = x.[< $op:lower >](rhs[i]);
                                    i += 1;

                                    res
                                }))
                            }
                        }

                        impl<'b, const N: usize, T> $op<&'b Vector<N, T>> for Vector<N, T>
                        where
                            T: Copy + $op<Output = T>,
                        {
                            type Output = Vector<N, T>;

                            fn [< $op:lower >](self, rhs: &'b Vector<N, T>) -> Self::Output {
                                self.[< $op:lower >](*rhs)
                            }
                        }

                        impl<'a, const N: usize, T> $op<Vector<N, T>> for &'a Vector<N, T>
                        where
                            T: Copy + $op<Output = T>,
                        {
                            type Output = Vector<N, T>;

                            fn [< $op:lower >](self, rhs: Vector<N, T>) -> Self::Output {
                                (*self).[< $op:lower >](rhs)
                            }
                        }

                        impl<'a, 'b, const N: usize, T> $op<&'b Vector<N, T>> for &'a Vector<N, T>
                        where
                            T: Copy + $op<Output = T>,
                        {
                            type Output = Vector<N, T>;

                            fn [< $op:lower >](self, rhs: &'b Vector<N, T>) -> Self::Output {
                                self.[< $op:lower >](*rhs)
                            }
                        }
                    }

                    pub use [< $op:lower _assign >]::*;
                    mod [< $op:lower _assign >] {
                        use super::*;

                        use std::ops::[< $op Assign >];

                        impl<const N: usize, T> [< $op Assign >]<T> for Vector<N, T>
                        where
                            T: Copy + [< $op Assign >]<T>,
                        {
                            fn [< $op:lower _assign >](&mut self, rhs: T) {
                                self.0.iter_mut().for_each(|x| x.[< $op:lower _assign >](rhs))
                            }
                        }

                        impl<'b, const N: usize, T> [< $op Assign >]<&'b T> for Vector<N, T>
                        where
                            T: Copy + [< $op Assign >]<T>,
                        {
                            fn [< $op:lower _assign >](&mut self, rhs: &'b T) {
                                self.[< $op:lower _assign >](*rhs)
                            }
                        }

                        impl<const N: usize, T> [< $op Assign >]<Vector<N, T>> for Vector<N, T>
                        where
                            T: Copy + [< $op Assign >]<T>,
                        {
                            fn [< $op:lower _assign >](&mut self, rhs: Vector<N, T>) {
                                let mut i = 0;

                                self.0.iter_mut().for_each(|x| {
                                    x.[< $op:lower _assign >](rhs[i]);
                                    i += 1;
                                });
                            }
                        }

                        impl<'b, const N: usize, T> [< $op Assign >]<&'b Vector<N, T>> for Vector<N, T>
                        where
                            T: Copy + [< $op Assign >]<T>,
                        {
                            fn [< $op:lower _assign >](&mut self, rhs: &'b Vector<N, T>) {
                                self.[< $op:lower _assign >](*rhs)
                            }
                        }
                    }
                )*
            }
        };
    }

    impl_vec_ops!(Add, Sub, Mul, Div, Rem);

    pub use neg::*;
    mod neg {
        use super::*;

        use std::ops::Neg;

        impl<const N: usize, T> Neg for Vector<N, T>
        where
            T: Neg<Output = T>,
        {
            type Output = Vector<N, T>;

            fn neg(self) -> Self::Output {
                Self(self.0.map(|x| -x))
            }
        }

        impl<'a, const N: usize, T> Neg for &'a Vector<N, T>
        where
            T: Copy + Neg<Output = T>,
        {
            type Output = Vector<N, T>;

            fn neg(self) -> Self::Output {
                Vector::new(self.0.map(|x| -x))
            }
        }
    }

    pub use dot::*;
    mod dot {
        use super::*;

        use std::ops::BitOr;

        impl<const N: usize, T> BitOr<Vector<N, T>> for Vector<N, T>
        where
            T: Default + Copy + Add<Output = T> + Mul<Output = T>,
        {
            type Output = T;

            fn bitor(self, rhs: Vector<N, T>) -> Self::Output {
                self.dot(&rhs)
            }
        }

        impl<'b, const N: usize, T> BitOr<&'b Vector<N, T>> for Vector<N, T>
        where
            T: Default + Copy + Add<Output = T> + Mul<Output = T>,
        {
            type Output = T;

            fn bitor(self, rhs: &'b Vector<N, T>) -> Self::Output {
                self.dot(rhs)
            }
        }

        impl<'a, const N: usize, T> BitOr<Vector<N, T>> for &'a Vector<N, T>
        where
            T: Default + Copy + Add<Output = T> + Mul<Output = T>,
        {
            type Output = T;

            fn bitor(self, rhs: Vector<N, T>) -> Self::Output {
                self.dot(&rhs)
            }
        }

        impl<'a, 'b, const N: usize, T> BitOr<&'b Vector<N, T>> for &'a Vector<N, T>
        where
            T: Default + Copy + Add<Output = T> + Mul<Output = T>,
        {
            type Output = T;

            fn bitor(self, rhs: &'b Vector<N, T>) -> Self::Output {
                self.dot(rhs)
            }
        }
    }

    pub use cross::*;
    mod cross {
        use super::*;

        use std::ops::{BitXor, BitXorAssign};

        impl<T> BitXor<Vector3<T>> for Vector3<T>
        where
            T: Copy + Sub<Output = T> + Mul<Output = T>,
        {
            type Output = Vector3<T>;

            fn bitxor(self, rhs: Vector3<T>) -> Self::Output {
                self.cross(&rhs)
            }
        }

        impl<'b, T> BitXor<&'b Vector3<T>> for Vector3<T>
        where
            T: Copy + Sub<Output = T> + Mul<Output = T>,
        {
            type Output = Vector3<T>;

            fn bitxor(self, rhs: &'b Vector3<T>) -> Self::Output {
                self.cross(rhs)
            }
        }

        impl<'a, T> BitXor<Vector3<T>> for &'a Vector3<T>
        where
            T: Copy + Sub<Output = T> + Mul<Output = T>,
        {
            type Output = Vector3<T>;

            fn bitxor(self, rhs: Vector3<T>) -> Self::Output {
                self.cross(&rhs)
            }
        }

        impl<'a, 'b, T> BitXor<&'b Vector3<T>> for &'a Vector3<T>
        where
            T: Copy + Sub<Output = T> + Mul<Output = T>,
        {
            type Output = Vector3<T>;

            fn bitxor(self, rhs: &'b Vector3<T>) -> Self::Output {
                self.cross(rhs)
            }
        }

        impl<T> BitXorAssign<Vector3<T>> for Vector3<T>
        where
            T: Copy + Sub<Output = T> + Mul<Output = T>,
        {
            fn bitxor_assign(&mut self, rhs: Vector3<T>) {
                *self = *self ^ rhs;
            }
        }

        impl<'b, T> BitXorAssign<&'b Vector3<T>> for Vector3<T>
        where
            T: Copy + Sub<Output = T> + Mul<Output = T>,
        {
            fn bitxor_assign(&mut self, rhs: &'b Vector3<T>) {
                *self = *self ^ rhs;
            }
        }
    }
}

#[cfg(feature = "serde")]
pub mod serialization {
    use super::*;

    use std::{any::Any, borrow::Borrow, fmt::Formatter};

    use serde::{
        de::{SeqAccess, Visitor},
        Deserialize, Deserializer, Serialize, Serializer,
    };

    impl<const N: usize, T> Serialize for Vector<N, T>
    where
        T: Serialize,
    {
        fn serialize<S>(&self, serializer: S) -> Result<serde::ser::Ok, dyn serde::ser::Error>
        where
            S: Serializer,
        {
            self.0.serialize(serializer)
        }
    }

    pub struct VectorVisitor<'de, const N: usize, T>;

    impl<'de, const N: usize, T> Visitor for VectorVisitor<'de, N, T> {
        type Value = Vector<N, T>;

        fn expecting(&self, formatter: &mut Formatter) -> std::fmt::Result {
            write!(formatter, "an [{}; {:?}] array", N, T::type_id())
        }

        fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, dyn serde::de::Error>
        where
            A: SeqAccess<'de>,
        {
            let res = [T::default(); N];

            let mut i = 0;
            while let Ok(Some(val)) = seq.next_element() {
                if i >= N {
                    return Err(serde::de::Error::invalid_length(i, &self));
                }

                res[i] = val;
            }

            if i < N - 1 {
                Err(serde::de::Error::invalid_length(i, &self))
            } else {
                Ok(Vector(res))
            }
        }
    }

    impl<'de, const N: usize, T> Deserialize for Vector<N, T>
    where
        T: Default,
    {
        fn deserialize<D>(deserializer: D) -> Result<Self, dyn serde::de::Error>
        where
            D: Deserializer<'de>,
        {
            deserializer.deserialize_seq(VectorVisitor::<N, T>)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn vector_creation() {
        let v3: Vector3I = Vector([1, 2, 3]);
        let v3_new: Vector3I = Vector3::new([1, 2, 3]);

        assert_eq!(v3, v3_new);

        let v3_same: Vector3I = Vector([1, 1, 1]);
        let v3_new_val_same: Vector3I = Vector3::new_val(1);

        assert_eq!(v3_same, v3_new_val_same);
    }

    #[test]
    fn vector_macro() {
        assert_eq!(vector!(1), Vector1I::new_val(1));
        assert_eq!(vector!(1, 2, 3), Vector3I::new([1, 2, 3]));
        assert_eq!(
            vector!(1, 2, 3, 4, 5),
            Vector::<5, i32>::new([1, 2, 3, 4, 5])
        );
    }

    #[test]
    fn length() {
        let v = vector!(2.0, 3.0, 1.0);

        assert!((v.length() - 3.741657).abs() < 0.05);
    }

    #[test]
    fn dot() {
        let v1: Vector3I = vector!(2, 3, 1);
        let v2: Vector3I = vector!(1, 2, 0);

        assert_eq!(v1.dot(&v2), 8);
        assert_eq!(v2.dot(&v1), 8);
        assert_eq!(v1 | v2, 8);
        assert_eq!(v2 | v1, 8);
    }

    #[test]
    fn cross() {
        let mut v1: Vector3I = vector!(2, 3, 1);
        let v2: Vector3I = vector!(1, 2, 0);
        let target = vector!(-2, 1, 1);

        assert_eq!(v1.cross(&v2), target);
        assert_eq!(v1 ^ v2, target);

        v1 ^= v2;
        assert_eq!(v1, target);
    }

    mod indexing {
        use super::*;

        #[test]
        fn vec1() {
            let v1: Vector1I = vector!(1);
            assert_eq!(v1[0], 1);
            assert!(std::panic::catch_unwind(|| { v1[1] }).is_err());
        }

        #[test]
        fn vec2() {
            let v2: Vector2I = vector!(1, 2);
            assert_eq!(v2[0], 1);
            assert_eq!(v2[1], 2);
            assert!(std::panic::catch_unwind(|| { v2[2] }).is_err());
        }

        #[test]
        fn vec3() {
            let v3: Vector3I = vector!(1, 2, 3);
            assert_eq!(v3[0], 1);
            assert_eq!(v3[1], 2);
            assert_eq!(v3[2], 3);
            assert!(std::panic::catch_unwind(|| { v3[3] }).is_err());
        }

        #[test]
        fn vec4() {
            let v4: Vector4I = vector!(1, 2, 3, 4);
            assert_eq!(v4[0], 1);
            assert_eq!(v4[1], 2);
            assert_eq!(v4[2], 3);
            assert_eq!(v4[3], 4);
            assert!(std::panic::catch_unwind(|| { v4[4] }).is_err());
        }
    }

    mod getters_setters {
        #[test]
        fn vec1() {
            let mut v1 = vector!(1);
            assert_eq!(v1.x(), 1);
            v1.set_x(2);
            assert_eq!(v1.x(), 2);
            *v1.x_mut() = 3;
            assert_eq!(v1.x(), 3);
        }

        #[test]
        fn vec2() {
            let mut v2 = vector!(1, 2);
            assert_eq!(v2.x(), 1);
            v2.set_x(2);
            assert_eq!(v2.x(), 2);
            *v2.x_mut() = 3;
            assert_eq!(v2.x(), 3);

            assert_eq!(v2.y(), 2);
            v2.set_y(3);
            assert_eq!(v2.y(), 3);
            *v2.y_mut() = 4;
            assert_eq!(v2.y(), 4);
        }

        #[test]
        fn vec3() {
            let mut v3 = vector!(1, 2, 3);
            assert_eq!(v3.x(), 1);
            v3.set_x(2);
            assert_eq!(v3.x(), 2);
            *v3.x_mut() = 3;
            assert_eq!(v3.x(), 3);

            assert_eq!(v3.y(), 2);
            v3.set_y(3);
            assert_eq!(v3.y(), 3);
            *v3.y_mut() = 4;
            assert_eq!(v3.y(), 4);

            assert_eq!(v3.z(), 3);
            v3.set_z(4);
            assert_eq!(v3.z(), 4);
            *v3.z_mut() = 5;
            assert_eq!(v3.z(), 5);
        }

        #[test]
        fn vec4() {
            let mut v4 = vector!(1, 2, 3, 4);
            assert_eq!(v4.x(), 1);
            v4.set_x(2);
            assert_eq!(v4.x(), 2);
            *v4.x_mut() = 3;
            assert_eq!(v4.x(), 3);

            assert_eq!(v4.y(), 2);
            v4.set_y(3);
            assert_eq!(v4.y(), 3);
            *v4.y_mut() = 4;
            assert_eq!(v4.y(), 4);

            assert_eq!(v4.z(), 3);
            v4.set_z(4);
            assert_eq!(v4.z(), 4);
            *v4.z_mut() = 5;
            assert_eq!(v4.z(), 5);

            assert_eq!(v4.w(), 4);
            v4.set_w(5);
            assert_eq!(v4.w(), 5);
            *v4.w_mut() = 6;
            assert_eq!(v4.w(), 6);
        }
    }

    mod casts {
        use super::*;

        #[test]
        fn vec1() {
            let target: Vector1I = vector!(1);

            let mut cast: Vector1I = 1.into();
            assert_eq!(cast, target);

            cast = vector!(1, 2).into();
            assert_eq!(cast, target);

            cast = vector!(1, 2, 3).into();
            assert_eq!(cast, target);

            cast = vector!(1, 2, 3, 4).into();
            assert_eq!(cast, target);
        }

        #[test]
        fn vec2() {
            let target: Vector2I = vector!(1, 2);

            let mut cast: Vector2I = (1, 2).into();
            assert_eq!(cast, target);

            cast = (vector!(1), vector!(2)).into();
            assert_eq!(cast, target);

            cast = vector!(1).into();
            assert_eq!(cast, vector!(1, 0));

            cast = vector!(1, 2, 3).into();
            assert_eq!(cast, target);

            cast = vector!(1, 2, 3, 4).into();
            assert_eq!(cast, target);
        }

        #[test]
        fn vec3() {
            let target: Vector3I = vector!(1, 2, 3);

            let mut cast: Vector3I = (1, 2, 3).into();
            assert_eq!(cast, target);

            cast = (vector!(1), vector!(2), vector!(3)).into();
            assert_eq!(cast, target);

            cast = (1, vector!(2, 3)).into();
            assert_eq!(cast, target);

            cast = (vector!(1), vector!(2, 3)).into();
            assert_eq!(cast, target);

            cast = (vector!(1, 2), 3).into();
            assert_eq!(cast, target);

            cast = (vector!(1, 2), vector!(3)).into();
            assert_eq!(cast, target);

            cast = vector!(1).into();
            assert_eq!(cast, vector!(1, 0, 0));

            cast = vector!(1, 2).into();
            assert_eq!(cast, vector!(1, 2, 0));

            cast = vector!(1, 2, 3, 4).into();
            assert_eq!(cast, target);
        }

        #[test]
        fn vec4() {
            let target: Vector4I = vector!(1, 2, 3, 4);

            let mut cast: Vector4I = (1, 2, 3, 4).into();
            assert_eq!(cast, target);

            cast = (vector!(1), vector!(2), vector!(3), vector!(4)).into();
            assert_eq!(cast, target);

            cast = (1, 2, vector!(3, 4)).into();
            assert_eq!(cast, target);

            cast = (vector!(1), vector!(2), vector!(3, 4)).into();
            assert_eq!(cast, target);

            cast = (1, vector!(2, 3, 4)).into();
            assert_eq!(cast, target);

            cast = (vector!(1), vector!(2, 3, 4)).into();
            assert_eq!(cast, target);

            cast = (vector!(1, 2), 3, 4).into();
            assert_eq!(cast, target);

            cast = (vector!(1, 2), vector!(3), vector!(4)).into();
            assert_eq!(cast, target);

            cast = (vector!(1, 2, 3), 4).into();
            assert_eq!(cast, target);

            cast = (vector!(1, 2, 3), vector!(4)).into();
            assert_eq!(cast, target);

            cast = (1, vector!(2, 3), 4).into();
            assert_eq!(cast, target);

            cast = (vector!(1), vector!(2, 3), vector!(4)).into();
            assert_eq!(cast, target);

            cast = vector!(1).into();
            assert_eq!(cast, vector!(1, 0, 0, 0));

            cast = vector!(1, 2).into();
            assert_eq!(cast, vector!(1, 2, 0, 0));

            cast = vector!(1, 2, 3).into();
            assert_eq!(cast, vector!(1, 2, 3, 0));
        }
    }

    mod ops {
        // TODO
    }
}
