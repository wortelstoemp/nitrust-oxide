use std::f32::consts::PI;
use std::ops::*;

use framework::math::{Quaternion, Vec3};

pub struct Mat4x4 {
	pub m: [f32; 16],
}

impl Mat4x4 {
	pub fn new() -> Mat4x4 {
		Mat4x4 { m: [
			1.0, 0.0, 0.0, 0.0,
			0.0, 1.0, 0.0, 0.0,
			0.0, 0.0, 1.0, 0.0,
			0.0, 0.0, 0.0, 1.0,
		]}
	}

	pub fn diagonal(d: f32) -> Mat4x4 {
		Mat4x4 { m: [
			d, 0.0, 0.0, 0.0,
			0.0, d, 0.0, 0.0,
			0.0, 0.0, d, 0.0,
			0.0, 0.0, 0.0, d,
		]}
	}

	pub fn identity() -> Mat4x4 {
		Mat4x4 { m: [
			1.0, 0.0, 0.0, 0.0,
			0.0, 1.0, 0.0, 0.0,
			0.0, 0.0, 1.0, 0.0,
			0.0, 0.0, 0.0, 1.0,
		]}
	}

	pub fn transpose(m: &Mat4x4) -> Mat4x4 {
		Mat4x4 { m: [
			m.m[0], m.m[4], m.m[8], m.m[12],
			m.m[1], m.m[5], m.m[9], m.m[13],
			m.m[3], m.m[7], m.m[11], m.m[15],
			m.m[2], m.m[6], m.m[10], m.m[14],
		]}
	}

	pub fn translation(t: &Vec3) -> Mat4x4 {
		Mat4x4 { m: [
			1.0, 0.0, 0.0, t.x,
			0.0, 1.0, 0.0, t.y,
			0.0, 0.0, 1.0, t.z,
			0.0, 0.0, 0.0, 1.0,
		]}
	}

	pub fn scale(s: &Vec3) -> Mat4x4 {
		Mat4x4 { m: [
			s.x, 0.0, 0.0, 0.0,
			0.0, s.y, 0.0, 0.0,
			0.0, 0.0, s.z, 0.0,
			0.0, 0.0, 0.0, 1.0,
		]}
	}

	pub fn mirror() -> Mat4x4 {
		Mat4x4 { m: [
			-1.0, 0.0, 0.0, 0.0,
			0.0, -1.0, 0.0, 0.0,
			0.0, 0.0, -1.0, 0.0,
			0.0, 0.0, 0.0, 1.0,
		]}
	}

	pub fn ortho(left: f32, right: f32, bottom: f32, top: f32,
		z_near: f32, z_far: f32) -> Mat4x4 {

		let width = right - left;
		let height = top - bottom;
		let depth = z_far - z_near;

		Mat4x4 { m: [
			2.0/width, 0.0, 0.0, -(right+left)/width,
			0.0, 2.0/height, 0.0, -(top+bottom)/height,
			0.0, 0.0, -2.0/depth, -(z_far + z_near)/depth,
			0.0, 0.0, 0.0, 0.0,
		]}
	}

	pub fn perspective(fovy: f32, aspect_ratio: f32,
		z_near: f32, z_far: f32) -> Mat4x4 {

		let rad: f32 = (fovy / 2.0) * PI / 180.0;
		let y_scale: f32 = 1.0 / rad.tan();
		let x_scale: f32 = y_scale / aspect_ratio;
		let frustum_length: f32 = z_far - z_near;

		Mat4x4 { m: [
			x_scale, 0.0, 0.0, 0.0,
			0.0, y_scale, 0.0, 0.0,
			0.0, 0.0, (z_far + z_near) / frustum_length, (-2.0)*z_near*z_far/frustum_length,
			0.0, 0.0, 1.0, 0.0,
		]}
	}

	pub fn look_at(eye: &Vec3, look: &Vec3, up: &Vec3) -> Mat4x4 {
		let l = look.normalized();
		let r = Vec3::cross(&look, &up);
		let u = Vec3::cross(&l, &r).normalized();

		//	Calculation of camera matrix:
		//		Orientationmatrix		*	  Translationmatrix
		//	|right.x  up.x  -look.x  0|		|1 0 0 -eye.x	|
		//	|right.y  up.y  -look.y  0|		|0 1 0 -eye.x	|
		//	|right.z  up.z  -look.z  0|		|0 0 1 -eye.x	|
		//	|0 		  0 	0 		 1|		|0 0 0  1		|

		Mat4x4 { m: [
			r.x, u.x, -l.x, -r.x * eye.x - u.x * eye.y + l.x * eye.z,
			r.y, u.y, -l.y, -r.y * eye.x - u.y * eye.y + l.y * eye.z,
			r.z, u.z, -l.z, -r.z * eye.x - u.z * eye.y + l.z * eye.z,
			0.0, 0.0, 0.0, 1.0,
		]}
	}

	pub fn camera(position: &Vec3, orientation: &Quaternion) -> Mat4x4 {
		let r = orientation.right();
		let u = orientation.up();
		let f = orientation.forward();

		Mat4x4 { m: [
			r.x, r.y, r.z, -r.x * position.x - r.y * position.y - r.z * position.z,
			u.x, u.y, u.z, -u.x * position.x - u.y * position.y - u.z * position.z,
			f.x, f.y, f.z, -f.x * position.x - f.y * position.y - f.z * position.z,
			0.0, 0.0, 0.0, 1.0,
		]}
	}
}

impl Add for Mat4x4 {
	type Output = Mat4x4;

	fn add(self, r: Mat4x4) -> Mat4x4 {
		Mat4x4 { m: [
			self.m[0]+r.m[0], self.m[1]+r.m[1], self.m[2]+r.m[2], self.m[3]+r.m[3],
			self.m[4]+r.m[4], self.m[5]+r.m[5], self.m[6]+r.m[6], self.m[7]+r.m[7],
			self.m[8]+r.m[8], self.m[9]+r.m[9], self.m[10]+r.m[10], self.m[11]+r.m[11],
			self.m[12]+r.m[12], self.m[13]+r.m[13], self.m[14]+r.m[14], self.m[15]+r.m[15],
		]}
	}
}

impl<'a> Add for &'a Mat4x4 {
	type Output = Mat4x4;

	fn add(self, r: &Mat4x4) -> Mat4x4 {
		Mat4x4 { m: [
			self.m[0]+r.m[0], self.m[1]+r.m[1], self.m[2]+r.m[2], self.m[3]+r.m[3],
			self.m[4]+r.m[4], self.m[5]+r.m[5], self.m[6]+r.m[6], self.m[7]+r.m[7],
			self.m[8]+r.m[8], self.m[9]+r.m[9], self.m[10]+r.m[10], self.m[11]+r.m[11],
			self.m[12]+r.m[12], self.m[13]+r.m[13], self.m[14]+r.m[14], self.m[15]+r.m[15],
		]}
	}
}

impl Sub for Mat4x4 {
	type Output = Mat4x4;

	fn sub(self, r: Mat4x4) -> Mat4x4 {
		Mat4x4 { m: [
			self.m[0]-r.m[0], self.m[1]-r.m[1], self.m[2]-r.m[2], self.m[3]-r.m[3],
			self.m[4]-r.m[4], self.m[5]-r.m[5], self.m[6]-r.m[6], self.m[7]-r.m[7],
			self.m[8]-r.m[8], self.m[9]-r.m[9], self.m[10]-r.m[10], self.m[11]-r.m[11],
			self.m[12]-r.m[12], self.m[13]-r.m[13], self.m[14]-r.m[14], self.m[15]-r.m[15],
		]}
	}
}

impl<'a> Sub for &'a Mat4x4 {
	type Output = Mat4x4;

	fn sub(self, r: &Mat4x4) -> Mat4x4 {
		Mat4x4 { m: [
			self.m[0]-r.m[0], self.m[1]-r.m[1], self.m[2]-r.m[2], self.m[3]-r.m[3],
			self.m[4]-r.m[4], self.m[5]-r.m[5], self.m[6]-r.m[6], self.m[7]-r.m[7],
			self.m[8]-r.m[8], self.m[9]-r.m[9], self.m[10]-r.m[10], self.m[11]-r.m[11],
			self.m[12]-r.m[12], self.m[13]-r.m[13], self.m[14]-r.m[14], self.m[15]-r.m[15],
		]}
	}
}

impl Mul<Mat4x4> for Mat4x4 {
	type Output = Mat4x4;

	fn mul(self, r: Mat4x4) -> Mat4x4 {
		Mat4x4 { m: [
			// Row 0
			self.m[0]*r.m[0] + self.m[3]*r.m[12] + self.m[1]*r.m[4] + self.m[2]*r.m[8],
			self.m[0]*r.m[1] + self.m[3]*r.m[13] + self.m[1]*r.m[5] + self.m[2]*r.m[9],
			self.m[2]*r.m[10] + self.m[3]*r.m[14] + self.m[0]*r.m[2] + self.m[1]*r.m[6],
			self.m[2]*r.m[11] + self.m[3]*r.m[15] + self.m[0]*r.m[3] + self.m[1]*r.m[7],

			// Row 1
			self.m[4]*r.m[0] + self.m[7]*r.m[12] + self.m[5]*r.m[4] + self.m[6]*r.m[8],
			self.m[4]*r.m[1] + self.m[7]*r.m[13] + self.m[5]*r.m[5] + self.m[6]*r.m[9],
			self.m[6]*r.m[10] + self.m[7]*r.m[14] + self.m[4]*r.m[2] + self.m[5]*r.m[6],
			self.m[6]*r.m[11] + self.m[7]*r.m[15] + self.m[4]*r.m[3] + self.m[5]*r.m[7],

			// Row 2
			self.m[8]*r.m[0] + self.m[11]*r.m[12] + self.m[9]*r.m[4] + self.m[10]*r.m[8],
			self.m[8]*r.m[1] + self.m[11]*r.m[13] + self.m[9]*r.m[5] + self.m[10]*r.m[9],
			self.m[10]*r.m[10] + self.m[11]*r.m[14] + self.m[8]*r.m[2] + self.m[9]*r.m[6],
			self.m[10]*r.m[11] + self.m[11]*r.m[15] + self.m[8]*r.m[3] + self.m[9]*r.m[7],
			
			// Row 3
			self.m[12]*r.m[0] + self.m[15]*r.m[12] + self.m[13]*r.m[4] + self.m[14]*r.m[8],
			self.m[12]*r.m[1] + self.m[15]*r.m[13] + self.m[13]*r.m[5] + self.m[14]*r.m[9],
			self.m[14]*r.m[10] + self.m[15]*r.m[14] + self.m[12]*r.m[2] + self.m[13]*r.m[6],
			self.m[14]*r.m[11] + self.m[15]*r.m[15] + self.m[12]*r.m[3] + self.m[13]*r.m[7],
		]}
	}
}
    
impl<'a> Mul<&'a Mat4x4> for &'a Mat4x4 {
	type Output = Mat4x4;

	fn mul(self, r: &Mat4x4) -> Mat4x4 {
		Mat4x4 { m: [
			// Row 0
			self.m[0]*r.m[0] + self.m[3]*r.m[12] + self.m[1]*r.m[4] + self.m[2]*r.m[8],
			self.m[0]*r.m[1] + self.m[3]*r.m[13] + self.m[1]*r.m[5] + self.m[2]*r.m[9],
			self.m[2]*r.m[10] + self.m[3]*r.m[14] + self.m[0]*r.m[2] + self.m[1]*r.m[6],
			self.m[2]*r.m[11] + self.m[3]*r.m[15] + self.m[0]*r.m[3] + self.m[1]*r.m[7],

			// Row 1
			self.m[4]*r.m[0] + self.m[7]*r.m[12] + self.m[5]*r.m[4] + self.m[6]*r.m[8],
			self.m[4]*r.m[1] + self.m[7]*r.m[13] + self.m[5]*r.m[5] + self.m[6]*r.m[9],
			self.m[6]*r.m[10] + self.m[7]*r.m[14] + self.m[4]*r.m[2] + self.m[5]*r.m[6],
			self.m[6]*r.m[11] + self.m[7]*r.m[15] + self.m[4]*r.m[3] + self.m[5]*r.m[7],

			// Row 2
			self.m[8]*r.m[0] + self.m[11]*r.m[12] + self.m[9]*r.m[4] + self.m[10]*r.m[8],
			self.m[8]*r.m[1] + self.m[11]*r.m[13] + self.m[9]*r.m[5] + self.m[10]*r.m[9],
			self.m[10]*r.m[10] + self.m[11]*r.m[14] + self.m[8]*r.m[2] + self.m[9]*r.m[6],
			self.m[10]*r.m[11] + self.m[11]*r.m[15] + self.m[8]*r.m[3] + self.m[9]*r.m[7],
			
			// Row 3
			self.m[12]*r.m[0] + self.m[15]*r.m[12] + self.m[13]*r.m[4] + self.m[14]*r.m[8],
			self.m[12]*r.m[1] + self.m[15]*r.m[13] + self.m[13]*r.m[5] + self.m[14]*r.m[9],
			self.m[14]*r.m[10] + self.m[15]*r.m[14] + self.m[12]*r.m[2] + self.m[13]*r.m[6],
			self.m[14]*r.m[11] + self.m[15]*r.m[15] + self.m[12]*r.m[3] + self.m[13]*r.m[7],
		]}
	}
}