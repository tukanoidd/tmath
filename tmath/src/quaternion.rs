use crate::vector::{Vector3, Vector4};

macro_rules! ops {
	[s; $($op_name:ident),*] => {
		paste::paste! {
		    $(
		        impl std::ops::$op_name<f32> for Quaternion {
				    type Output = Self;

				    #[inline]
				    fn [< $op_name:lower >](self, rhs: f32) -> Self {
					    Self {
						    s: self.s.[< $op_name:lower >](rhs),
						    v: self.v.[< $op_name:lower >](rhs)
					    }
				    }
			    }

		        impl<'a> std::ops::$op_name<f32> for &'a Quaternion {
				    type Output = Quaternion;

				    #[inline]
				    fn [< $op_name:lower >](self, rhs: f32) -> Quaternion {
					    Quaternion {
						    s: self.s.[< $op_name:lower >](rhs),
						    v: self.v.[< $op_name:lower >](rhs)
					    }
				    }
			    }

		        impl std::ops::[< $op_name Assign >]<f32> for Quaternion {
			        #[inline]
			        fn [< $op_name:lower _assign >](&mut self, rhs: f32) {
				        self.s.[< $op_name:lower _assign >](rhs);
				        self.v.[< $op_name:lower _assign >](rhs);
			        }
		        }
		    )*
	    }
	};

    [q; $($op_name:ident),*] => {
	    paste::paste! {
		    $(
		        impl std::ops::$op_name for Quaternion {
				    type Output = Self;

				    #[inline]
				    fn [< $op_name:lower >](self, rhs: Self) -> Self {
					    Self {
						    s: self.s.[< $op_name:lower >](rhs.s),
						    v: self.v.[< $op_name:lower >](rhs.v)
					    }
				    }
			    }

		        impl std::ops::[< $op_name Assign >] for Quaternion {
			        #[inline]
			        fn [< $op_name:lower _assign >](&mut self, rhs: Self) {
				        self.s.[< $op_name:lower _assign >](rhs.s);
				        self.v.[< $op_name:lower _assign >](rhs.v);
			        }
		        }

		        impl<'a, 'b> std::ops::$op_name<&'b Quaternion> for &'a Quaternion {
			        type Output = Quaternion;

			        #[inline]
			        fn [< $op_name:lower >](self, rhs: &'b Quaternion) -> Quaternion {
				        Quaternion {
					        s: self.s.[< $op_name:lower >](rhs.s),
					        v: self.v.[< $op_name:lower >](rhs.v)
				        }
			        }
		        }

		        impl<'b> std::ops::[< $op_name Assign>]<&'b Quaternion> for Quaternion {
			        #[inline]
			        fn [< $op_name:lower _assign >](&mut self, rhs: &'b Quaternion) {
				        self.s.[< $op_name:lower _assign >](rhs.s);
				        self.v.[< $op_name:lower _assign >](rhs.v);
			        }
		        }
		    )*
	    }
    };
}

ops![s; Add, Sub, Mul, Div, Rem];
ops![q; Add, Sub];

#[repr(C)]
#[derive(Default, Copy, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Quaternion {
    pub s: f32,
    pub v: Vector3,
}

impl Quaternion {
    #[inline]
    pub fn new(s: f32, v: Vector3) -> Self {
        Self { s, v }
    }

    #[allow(clippy::eq_op)]
    #[inline]
    pub fn norm(&self) -> f32 {
        (self.s * self.s + (self.v | self.v)).sqrt()
    }

    pub fn normalize(&mut self) {
        let norm = self.norm();

        if norm != 0.0 {
            self.s /= norm;
            self.v /= norm;
        }
    }

    pub fn normalized(&self) -> Self {
        let norm = self.norm();

        if norm != 0.0 {
            *self / norm
        } else {
            *self
        }
    }

    pub fn unit_norm(&mut self) {
        let half_angle = self.s.to_radians() / 2.0;

        self.v.normalize();
        self.s = half_angle.cos();
        self.v *= half_angle.sin();
    }

    pub fn as_unit_norm(&self) -> Self {
        let half_angle = self.s.to_radians() / 2.0;

        Self {
            s: half_angle.cos(),
            v: self.v.normalized() * half_angle.sin(),
        }
    }

    #[inline]
    pub fn conjugate(&self) -> Self {
        Self {
            s: self.s,
            v: -self.v,
        }
    }

    pub fn inverse(&self) -> Self {
        let mut absolute_value = self.norm();
        absolute_value = 1.0 / (absolute_value * absolute_value);

        self.conjugate() * absolute_value
    }
}

impl std::ops::Mul for Quaternion {
    type Output = Self;

    #[inline]
    fn mul(self, rhs: Self) -> Self::Output {
        Self {
            s: self.s * rhs.s - (self.v | rhs.v),
            v: rhs.v * self.s + self.v * rhs.s + (self.v ^ rhs.v),
        }
    }
}

impl std::ops::MulAssign for Quaternion {
    #[inline]
    fn mul_assign(&mut self, rhs: Self) {
        self.s = self.s * rhs.s - (self.v | rhs.v);
        self.v = rhs.v * self.s + self.v * rhs.s + (self.v ^ rhs.v);
    }
}

impl<'a, 'b> std::ops::Mul<&'b Quaternion> for &'a Quaternion {
    type Output = Quaternion;

    #[inline]
    fn mul(self, rhs: &'b Quaternion) -> Self::Output {
        Quaternion {
            s: self.s * rhs.s - (self.v | rhs.v),
            v: rhs.v * self.s + self.v * rhs.s + (self.v ^ rhs.v),
        }
    }
}

impl<'b> std::ops::MulAssign<&'b Quaternion> for Quaternion {
    #[inline]
    fn mul_assign(&mut self, rhs: &'b Quaternion) {
        self.s = self.s * rhs.s - (self.v | rhs.v);
        self.v = rhs.v * self.s + self.v * rhs.s + (self.v ^ rhs.v);
    }
}

impl std::ops::Neg for Quaternion {
    type Output = Self;

    #[inline]
    fn neg(self) -> Self::Output {
        self.conjugate()
    }
}

impl std::ops::Index<usize> for Quaternion {
    type Output = f32;

    fn index(&self, index: usize) -> &Self::Output {
        match index {
            3 => &self.s,
            _ => &self.v[index],
        }
    }
}

impl From<[f32; 4]> for Quaternion {
    #[inline]
    fn from(rhs: [f32; 4]) -> Self {
        Self {
            s: rhs[3],
            v: (rhs[0], rhs[1], rhs[2]).into(),
        }
    }
}

impl From<(f32, f32, f32, f32)> for Quaternion {
    #[inline]
    fn from((x, y, z, w): (f32, f32, f32, f32)) -> Self {
        Self {
            s: w,
            v: (x, y, z).into(),
        }
    }
}

impl From<(Vector3, f32)> for Quaternion {
    #[inline]
    fn from((v, s): (Vector3, f32)) -> Self {
        Self { s, v }
    }
}

impl From<Vector4> for Quaternion {
    #[inline]
    fn from(rhs: Vector4) -> Self {
        Self {
            s: rhs[3],
            v: (rhs[0], rhs[1], rhs[2]).into(),
        }
    }
}
