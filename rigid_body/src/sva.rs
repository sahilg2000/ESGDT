use core::ops::{Add, Mul, Sub};
use std::ops::{AddAssign, SubAssign};

use nalgebra::{Matrix3, Matrix6, Matrix6x1, Quaternion, SMatrix, UnitQuaternion, Vector3};

pub type Vector = Vector3<f64>;
pub type Matrix = Matrix3<f64>;

pub fn rx(angle: f64) -> Matrix {
    Matrix::new(
        1.0,
        0.0,
        0.0,
        0.0,
        angle.cos(),
        angle.sin(),
        0.0,
        -angle.sin(),
        angle.cos(),
    )
}

pub fn ry(angle: f64) -> Matrix {
    Matrix::new(
        angle.cos(),
        0.0,
        -angle.sin(),
        0.0,
        1.0,
        0.0,
        angle.sin(),
        0.0,
        angle.cos(),
    )
}

pub fn rz(angle: f64) -> Matrix {
    Matrix::new(
        angle.cos(),
        angle.sin(),
        0.0,
        -angle.sin(),
        angle.cos(),
        0.0,
        0.0,
        0.0,
        1.0,
    )
}

#[derive(Debug, Copy, Clone)]
pub struct Velocity {
    pub vel: Vector,
}

#[derive(Debug, Copy, Clone)]
pub struct Xform {
    pub position: Vector,
    pub rotation: Matrix,
}

impl Default for Xform {
    fn default() -> Self {
        Self::identity()
    }
}

impl Xform {
    pub fn new(position: Vector, rotation: Matrix) -> Self {
        Self { position, rotation }
    }
    pub fn identity() -> Self {
        Self {
            position: Vector::zeros(),
            rotation: Matrix::identity(),
        }
    }
    pub fn inverse(self) -> Self {
        Self {
            position: -(self.rotation * self.position),
            rotation: self.rotation.transpose(),
        }
    }
    pub fn rotx(angle: f64) -> Self {
        Self {
            rotation: rx(angle),
            ..Default::default()
        }
    }
    pub fn roty(angle: f64) -> Self {
        Self {
            rotation: ry(angle),
            ..Default::default()
        }
    }
    pub fn rotz(angle: f64) -> Self {
        Self {
            rotation: rz(angle),
            ..Default::default()
        }
    }
    pub fn posx(x: f64) -> Self {
        Self {
            position: Vector::new(x, 0.0, 0.0),
            ..Default::default()
        }
    }
    pub fn posy(y: f64) -> Self {
        Self {
            position: Vector::new(0.0, y, 0.0),
            ..Default::default()
        }
    }
    pub fn posz(z: f64) -> Self {
        Self {
            position: Vector::new(0.0, 0.0, z),
            ..Default::default()
        }
    }
    pub fn pos(x: f64, y: f64, z: f64) -> Self {
        Self {
            position: Vector::new(x, y, z),
            ..Default::default()
        }
    }

    pub fn quaternion(x: f64, y: f64, z: f64, w: f64) -> Self {
        let quaternion = Quaternion::new(x, y, z, w).normalize();
        // wow gross
        let rotation = UnitQuaternion::from_quaternion(quaternion)
            .to_rotation_matrix()
            .matrix()
            .clone();
        Self {
            position: Vector::zeros(),
            rotation,
        }
    }

    pub fn transform_point(self, point: Vector) -> Vector {
        self.rotation * (point - self.position)
    }

}

impl Mul<Xform> for Xform {
    type Output = Xform;

    fn mul(self, rhs: Xform) -> Xform {
        Xform {
            position: rhs.position + rhs.rotation.transpose() * self.position,
            rotation: self.rotation * rhs.rotation,
        }
    }
}

impl Mul<&Xform> for &Xform {
    type Output = Xform;

    fn mul(self, rhs: &Xform) -> Xform {
        Xform {
            position: rhs.position + rhs.rotation.transpose() * self.position,
            rotation: self.rotation * rhs.rotation,
        }
    }
}

impl Mul<&Xform> for &mut Xform {
    type Output = Xform;

    fn mul(self, rhs: &Xform) -> Xform {
        Xform {
            position: rhs.position + rhs.rotation.transpose() * self.position,
            rotation: self.rotation * rhs.rotation,
        }
    }
}

impl Mul<Motion> for Xform {
    type Output = Motion;

    fn mul(self, rhs: Motion) -> Motion {
        Motion {
            v: self.rotation * (rhs.v - self.position.cross(&rhs.w)),
            w: self.rotation * rhs.w,
        }
    }
}

impl Mul<Motion> for &Xform {
    type Output = Motion;

    fn mul(self, rhs: Motion) -> Motion {
        Motion {
            v: self.rotation * (rhs.v - self.position.cross(&rhs.w)),
            w: self.rotation * rhs.w,
        }
    }
}

impl Mul<Motion> for &mut Xform {
    type Output = Motion;

    fn mul(self, rhs: Motion) -> Motion {
        Motion {
            v: self.rotation * (rhs.v - self.position.cross(&rhs.w)),
            w: self.rotation * rhs.w,
        }
    }
}

impl Mul<&Motion> for &Xform {
    type Output = Motion;

    fn mul(self, rhs: &Motion) -> Motion {
        Motion {
            v: self.rotation * (rhs.v - self.position.cross(&rhs.w)),
            w: self.rotation * rhs.w,
        }
    }
}

impl Mul<Force> for Xform {
    type Output = Force;

    fn mul(self, rhs: Force) -> Force {
        Force {
            f: self.rotation * rhs.f,
            m: self.rotation * (rhs.m - self.position.cross(&rhs.f)),
        }
    }
}

impl Mul<Vector> for Xform {
    type Output = Vector;

    fn mul(self, rhs: Vector) -> Vector {
        self.rotation * rhs
    }
}

impl Mul<Velocity> for Xform {
    type Output = Velocity;

    fn mul(self, rhs: Velocity) -> Velocity {
        Velocity {
            vel: self.rotation * rhs.vel,
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub struct Motion {
    pub v: Vector,
    pub w: Vector,
}

impl Motion {
    pub fn new(v_data: [f64; 3], w_data: [f64; 3]) -> Self {
        Self {
            v: Vector::new(v_data[0], v_data[1], v_data[2]),
            w: Vector::new(w_data[0], w_data[1], w_data[2]),
        }
    }
    pub fn zero() -> Self {
        Self {
            v: Vector::zeros(),
            w: Vector::zeros(),
        }
    }

    pub fn cross_v(self, rhs: Motion) -> Motion {
        Motion {
            v: self.w.cross(&rhs.v) + self.v.cross(&rhs.w),
            w: self.w.cross(&rhs.w),
        }
    }

    pub fn cross_f(self, rhs: Force) -> Force {
        Force {
            f: self.w.cross(&rhs.f),
            m: self.w.cross(&rhs.m) + self.v.cross(&rhs.f),
        }
    }

    pub fn velocity_point(self, point: Vector) -> Velocity {
        Velocity {
            vel: self.w.cross(&point) + self.v,
        }
    }
}

impl Add<Motion> for Motion {
    type Output = Motion;
    fn add(self, rhs: Motion) -> Motion {
        Motion {
            v: self.v + rhs.v,
            w: self.w + rhs.w,
        }
    }
}

impl Default for Motion {
    fn default() -> Self {
        Self::zero()
    }
}

impl Mul<Motion> for f64 {
    type Output = Motion;
    fn mul(self, rhs: Motion) -> Motion {
        Motion {
            w: self * rhs.w,
            v: self * rhs.v,
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub struct Force {
    pub f: Vector,
    pub m: Vector,
}

impl Force {
    pub fn new(f_data: [f64; 3], m_data: [f64; 3]) -> Self {
        Self {
            f: Vector::new(f_data[0], f_data[1], f_data[2]),
            m: Vector::new(m_data[0], m_data[1], m_data[2]),
        }
    }
    pub fn zero() -> Self {
        Self {
            f: Vector::zeros(),
            m: Vector::zeros(),
        }
    }

    pub fn self_outer_product(self) -> InertiaAB {
        InertiaAB {
            m: self.f * self.f.transpose(),
            c: self.m * self.f.transpose(),
            moi: self.m * self.m.transpose(),
        }
    }

    pub fn force_point(force: Vector, point: Vector) -> Force {
        Force {
            f: force,
            m: point.cross(&force),
        }
    }

    pub fn from_mat(mat: &Matrix6x1<f64>) -> Self {
        Self {
            m: Vector::new(mat[(0, 0)], mat[(1, 0)], mat[(2, 0)]),
            f: Vector::new(mat[(3, 0)], mat[(4, 0)], mat[(5, 0)]),
        }
    }
}

impl Default for Force {
    fn default() -> Self {
        Self::zero()
    }
}

impl Add<Force> for Force {
    type Output = Force;
    fn add(self, rhs: Force) -> Force {
        Force {
            m: self.m + rhs.m,
            f: self.f + rhs.f,
        }
    }
}

impl AddAssign<Force> for Force {
    fn add_assign(&mut self, rhs: Force) {
        self.m += rhs.m;
        self.f += rhs.f;
    }
}

impl Mul<Force> for f64 {
    type Output = Force;
    fn mul(self, rhs: Force) -> Force {
        Force {
            m: self * rhs.m,
            f: self * rhs.f,
        }
    }
}

impl Sub for Force {
    type Output = Force;
    fn sub(self, rhs: Force) -> Force {
        Force {
            m: self.m - rhs.m,
            f: self.f - rhs.f,
        }
    }
}

impl SubAssign<Force> for Force {
    fn sub_assign(&mut self, rhs: Force) {
        self.m -= rhs.m;
        self.f -= rhs.f;
    }
}

// impl Mul<&Motion> for &Force {
//     type Output = Force;

//     fn mul(self, rhs: &Motion) -> Force {
//         Force {
//             f: self.f,
//             m: self.m + self.f.cross(&rhs.v),
//         }
//     }
// }

impl Mul<&Motion> for &Force {
    type Output = f64;

    fn mul(self, rhs: &Motion) -> f64 {
        self.m.dot(&rhs.w) + self.f.dot(&rhs.v)
    }
}

impl Mul<&Force> for &Motion {
    type Output = f64;

    fn mul(self, rhs: &Force) -> f64 {
        self.w.dot(&rhs.m) + self.v.dot(&rhs.f)
    }
}

#[derive(Default, Debug, Copy, Clone)]
pub struct Inertia {
    m: f64,
    c: Vector,
    moi: Matrix,
}

impl Inertia {
    pub fn new(m: f64, c: Vector, moi: Matrix) -> Inertia {
        Inertia { m, c, moi }
    }
    pub fn zero() -> Inertia {
        Inertia {
            m: 0.0,
            c: Vector::zeros(),
            moi: Matrix::zeros(),
        }
    }
}

impl Mul<Motion> for Inertia {
    type Output = Force;
    fn mul(self, rhs: Motion) -> Force {
        Force {
            f: self.m * rhs.v - self.c.cross(&rhs.w),
            m: self.moi * rhs.w + self.c.cross(&rhs.v),
        }
    }
}

#[derive(Default, Debug, Copy, Clone)]
pub struct InertiaAB {
    m: Matrix,
    c: Matrix,
    moi: Matrix,
}

impl InertiaAB {
    pub fn zero() -> InertiaAB {
        InertiaAB {
            m: Matrix::zeros(),
            c: Matrix::zeros(),
            moi: Matrix::zeros(),
        }
    }

    pub fn from_mat(mat: &Matrix6<f64>) -> Self {
        // mat = [
        //     [moi, c],
        //     [c.t, m]
        //     ]
        let mut moi = Matrix::zeros();
        let mut c = Matrix::zeros();
        let mut m = Matrix::zeros();
        for i in 0..3 {
            for j in 0..3 {
                moi[(i, j)] = mat[(i, j)];
                c[(i, j)] = mat[(i, j + 3)];
                m[(i, j)] = mat[(i + 3, j + 3)];
            }
        }
        InertiaAB { m, c, moi }
    }
}

impl From<Inertia> for InertiaAB {
    fn from(i: Inertia) -> Self {
        let c_cross = i.c.cross_matrix();
        InertiaAB {
            m: i.m * Matrix::identity(),
            c: i.m * c_cross,
            moi: i.moi - i.m * c_cross * c_cross,
        }
    }
}

impl Mul<Motion> for InertiaAB {
    type Output = Force;
    fn mul(self, rhs: Motion) -> Force {
        Force {
            f: self.m * rhs.v + self.c.transpose() * rhs.w,
            m: self.moi * rhs.w + self.c * rhs.v,
        }
    }
}

impl Mul<InertiaAB> for f64 {
    type Output = InertiaAB;
    fn mul(self, rhs: InertiaAB) -> InertiaAB {
        InertiaAB {
            c: self * rhs.c,
            m: self * rhs.m,
            moi: self * rhs.moi,
        }
    }
}

impl Add<InertiaAB> for InertiaAB {
    type Output = InertiaAB;
    fn add(self, rhs: InertiaAB) -> InertiaAB {
        InertiaAB {
            c: self.c + rhs.c,
            m: self.m + rhs.m,
            moi: self.moi + rhs.moi,
        }
    }
}

impl AddAssign<InertiaAB> for InertiaAB {
    fn add_assign(&mut self, rhs: InertiaAB) {
        self.c += rhs.c;
        self.m += rhs.m;
        self.moi += rhs.moi;
    }
}

impl Sub<InertiaAB> for InertiaAB {
    type Output = InertiaAB;
    fn sub(self, rhs: InertiaAB) -> InertiaAB {
        InertiaAB {
            c: self.c - rhs.c,
            m: self.m - rhs.m,
            moi: self.moi - rhs.moi,
        }
    }
}

impl Mul<InertiaAB> for Xform {
    type Output = InertiaAB;

    fn mul(self, inertia: InertiaAB) -> InertiaAB {
        let rot_m = self.rotation;
        let i_cross = self.position.cross_matrix();

        InertiaAB {
            // definitely not tested
            m: rot_m * inertia.m * rot_m.transpose(),
            c: rot_m * (inertia.c - (i_cross * inertia.m)) * rot_m.transpose(),
            moi: rot_m
                * ((inertia.moi - (i_cross * inertia.c.transpose()))
                    + ((inertia.c - (i_cross * inertia.m)) * i_cross))
                * rot_m.transpose(),
        }
    }
}

pub struct MotionArray<const N: usize> {
    pub motions: [Motion; N],
}

pub struct ForceArray<const N: usize> {
    pub forces: [Force; N],
}

impl<const N: usize> ForceArray<N> {
    pub fn to_mat(&self) -> SMatrix<f64, 6, N> {
        let mut array = SMatrix::<f64, 6, N>::zeros();
        for i in 0..N {
            array[(0, i)] = self.forces[i].m.x;
            array[(1, i)] = self.forces[i].m.y;
            array[(2, i)] = self.forces[i].m.z;
            array[(3, i)] = self.forces[i].f.x;
            array[(4, i)] = self.forces[i].f.y;
            array[(5, i)] = self.forces[i].f.z;
        }
        array
    }
}

impl<const N: usize> Mul<&Motion> for &ForceArray<N> {
    type Output = SMatrix<f64, N, 1>;
    fn mul(self, rhs: &Motion) -> SMatrix<f64, N, 1> {
        let mut array = [0.; N];
        for i in 0..N {
            array[i] = rhs * &self.forces[i];
        }
        SMatrix::from_column_slice(&array)
    }
}

impl<const N: usize> Mul<&SMatrix<f64, N, 1>> for &MotionArray<N> {
    type Output = Motion;
    fn mul(self, rhs: &SMatrix<f64, N, 1>) -> Motion {
        let mut v = Vector::zeros();
        let mut w = Vector::zeros();
        for i in 0..N {
            v += self.motions[i].v * rhs[(i, 0)];
            w += self.motions[i].w * rhs[(i, 0)];
        }
        Motion { v, w }
    }
}

impl<const N: usize> MotionArray<N> {
    pub fn new(motions: [Motion; N]) -> Self {
        Self { motions }
    }
}

impl<const N: usize> Mul<&MotionArray<N>> for InertiaAB {
    type Output = ForceArray<N>;
    fn mul(self, rhs: &MotionArray<N>) -> ForceArray<N> {
        let mut forces = [Force::zero(); N];
        for i in 0..N {
            forces[i] = self * rhs.motions[i];
        }
        ForceArray { forces }
    }
}

impl<const N: usize> Mul<&ForceArray<N>> for &MotionArray<N> {
    type Output = nalgebra::SMatrix<f64, N, N>;
    fn mul(self, rhs: &ForceArray<N>) -> nalgebra::SMatrix<f64, N, N> {
        let array = nalgebra::SMatrix::from_fn(|i, j| &self.motions[i] * &rhs.forces[j]);
        array
    }
}

impl<const N: usize> Mul<Force> for &MotionArray<N> {
    type Output = SMatrix<f64, N, 1>;
    fn mul(self, rhs: Force) -> SMatrix<f64, N, 1> {
        let mut array = [0.; N];
        for i in 0..N {
            array[i] = &self.motions[i] * &rhs;
        }
        SMatrix::from_column_slice(&array)
    }
}
