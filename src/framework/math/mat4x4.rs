use std::f32::consts::PI;
use std::ops::*;

use framework::math::Quaternion;
use framework::math::Vec3;

#[derive(Copy, Clone)]
pub struct Mat4x4 {
	pub m: [f32; 16],
}

impl Mat4x4 {
	pub fn new() -> Mat4x4 {
		Mat4x4 {
			m: [
				1.0, 0.0, 0.0, 0.0,
				0.0, 1.0, 0.0, 0.0,
				0.0, 0.0, 1.0, 0.0,
				0.0, 0.0, 0.0, 1.0,
			],
		}
	}

	pub fn diagonal(&mut self, d: f32) -> Mat4x4 {
		self.m = [
			d, 0.0, 0.0, 0.0,
			0.0, d, 0.0, 0.0,
			0.0, 0.0, d, 0.0,
			0.0, 0.0, 0.0, d,
		];
		*self
	}

	pub fn identity(&mut self) -> Mat4x4 {
		self.m = [
			1.0, 0.0, 0.0, 0.0,
			0.0, 1.0, 0.0, 0.0,
			0.0, 0.0, 1.0, 0.0,
			0.0, 0.0, 0.0, 1.0,
		];
			*self
	}

	pub fn transpose(&mut self) -> Mat4x4 {
		self.m = [
			self.m[0], self.m[4], self.m[8], self.m[12],
			self.m[1], self.m[5], self.m[9], self.m[13],
			self.m[2], self.m[6], self.m[10], self.m[14],
			self.m[3], self.m[7], self.m[11], self.m[15],
		];
		*self
	}

	pub fn translate(&mut self, t: Vec3) -> Mat4x4 {
		let translated = Mat4x4 { m: [
			1.0, 0.0, 0.0, t.x,
			0.0, 1.0, 0.0, t.y,
			0.0, 0.0, 1.0, t.z,
			0.0, 0.0, 0.0, 1.0,
		]} * *self;

		self.m = translated.m;
		*self
	}

	pub fn scale(&mut self, s: Vec3) -> Mat4x4 {
		let scaled = Mat4x4 { m: [
			s.x, 0.0, 0.0, 0.0,
			0.0, s.y, 0.0, 0.0,
			0.0, 0.0, s.z, 0.0,
			0.0, 0.0, 0.0, 1.0,
		]} * *self;

		self.m = scaled.m;
		*self	
	}

	pub fn mirror(&mut self) -> Mat4x4 {
		let mirror = Mat4x4 { m: [
			-1.0, 0.0, 0.0, 0.0,
			0.0, -1.0, 0.0, 0.0,
			0.0, 0.0, -1.0, 0.0,
			0.0, 0.0, 0.0, 1.0,
		]} * *self;

		self.m = mirror.m;
		*self
	}

	pub fn ortho(&mut self, left: f32, right: f32, bottom: f32, top: f32,
		z_near: f32, z_far: f32) -> Mat4x4 {

		let width = right - left;
		let height = top - bottom;
		let depth = z_far - z_near;

		self.m = [
			2.0/width, 0.0, 0.0, -(right+left)/width,
			0.0, 2.0/height, 0.0, -(top+bottom)/height,
			0.0, 0.0, -2.0/depth, -(z_far + z_near)/depth,
			0.0, 0.0, 0.0, 0.0,
		];
		*self
	}

	pub fn perspective(&mut self, fovy: f32, aspect_ratio: f32,
		z_near: f32, z_far: f32) -> Mat4x4 {

		let rad: f32 = (fovy / 2.0) * PI / 180.0;
		let y_scale: f32 = 1.0 / rad.tan();
		let x_scale: f32 = y_scale / aspect_ratio;
		let frustum_length: f32 = z_far - z_near;

		self.m = [
			x_scale, 0.0, 0.0, 0.0,
			0.0, y_scale, 0.0, 0.0,
			0.0, 0.0, (z_far + z_near) / frustum_length, (-2.0)*z_near*z_far/frustum_length,
			0.0, 0.0, 1.0, 0.0,
		];
		*self
	}

	pub fn look_at(&mut self, eye: Vec3, look: Vec3, up: Vec3) -> Mat4x4 {
		let l = look.normalized();
		let r = look.cross(up);
		let u = l.cross(r).normalized();

		//	Calculation of camera matrix:
		//		Orientationmatrix		*	  Translationmatrix
		//	|right.x  up.x  -look.x  0|		|1 0 0 -eye.x	|
		//	|right.y  up.y  -look.y  0|		|0 1 0 -eye.x	|
		//	|right.z  up.z  -look.z  0|		|0 0 1 -eye.x	|
		//	|0 		  0 	0 		 1|		|0 0 0  1		|

		self.m = [
			r.x, u.x, -l.x, -r.x * eye.x - u.x *eye.y + l.x *eye.z,
			r.y, u.y, -l.y, -r.y * eye.x - u.y *eye.y + l.y *eye.z,
			r.z, u.z, -l.z, -r.z * eye.x - u.z *eye.y + l.z *eye.z,
			0.0, 0.0, 0.0, 1.0,
		];

		*self
	}

	pub fn camera(&mut self, position: Vec3, orientation: Quaternion) -> Mat4x4 {
		let r = orientation.right();
		let u = orientation.up();
		let f = orientation.forward();

		self.m = [
			r.x, r.y, r.z, -r.x * position.x - r.y * position.y - r.z * position.z,
			u.x, u.y, u.z, -u.x * position.x - u.y * position.y - u.z * position.z,
			f.x, f.y, f.z, -f.x * position.x - f.y * position.y - f.z * position.z,
			0.0, 0.0, 0.0, 1.0,
		];

		*self
	}
}

impl Add for Mat4x4 {
	type Output = Mat4x4;

	fn add(self, r: Mat4x4) -> Mat4x4 {
		let mut res = Mat4x4::new();
		res.m = [
			self.m[0]+r.m[0], self.m[1]+r.m[1], self.m[2]+r.m[2], self.m[3]+r.m[3],
			self.m[4]+r.m[4], self.m[5]+r.m[5], self.m[6]+r.m[6], self.m[7]+r.m[7],
			self.m[8]+r.m[8], self.m[9]+r.m[9], self.m[10]+r.m[10], self.m[11]+r.m[11],
			self.m[12]+r.m[12], self.m[13]+r.m[13], self.m[14]+r.m[14], self.m[15]+r.m[15],
		];
		res
	}
}

impl Sub for Mat4x4 {
	type Output = Mat4x4;

	fn sub(self, r: Mat4x4) -> Mat4x4 {
		let mut res = Mat4x4::new();
		res.m = [
			self.m[0]-r.m[0], self.m[1]-r.m[1], self.m[2]-r.m[2], self.m[3]-r.m[3],
			self.m[4]-r.m[4], self.m[5]-r.m[5], self.m[6]-r.m[6], self.m[7]-r.m[7],
			self.m[8]-r.m[8], self.m[9]-r.m[9], self.m[10]-r.m[10], self.m[11]-r.m[11],
			self.m[12]-r.m[12], self.m[13]-r.m[13], self.m[14]-r.m[14], self.m[15]-r.m[15],
		];
		res
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