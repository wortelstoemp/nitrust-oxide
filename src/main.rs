// Copyright 2015 Tom Quareme @wortelstoemp
// Note that this file currently is a mess
// When I get basic OpenGL 3.3 running this will evolve
// into an earlier renderer I wrote in LWJGL back in the day.
// Everything will be organized in different modules and files.

extern crate gl;
extern crate libc;
extern crate sdl2;
extern crate time;

use libc::c_void;
use gl::types::*;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::video::GLProfile;

use std::collections::HashMap;
use std::io;
use std::io::{ Error, ErrorKind };
use std::io::prelude::*;
use std::fs::File;
use std::ptr;
use std::str;
use std::ffi::CString;

use core::*;
use math::*;

mod core {
	use time::PreciseTime;
	use math::*;

	pub struct Clock {
		current: PreciseTime,
		last: PreciseTime,
		absolute: f32,
		delta: f32,
		fixed: f32,
		accumulator: f32,
	}

	impl Clock {
		pub fn new(fps: f32) -> Clock {
			Clock {
				current: PreciseTime::now(),
				last: PreciseTime::now(),
				absolute: 0.0,
				delta: 0.0,
				fixed: (1.0/fps),
				accumulator: 0.0,
			}
		}

		pub fn accumulating(&self) -> bool {
			(self.accumulator >= self.fixed)
		}

		pub fn accumulate(&mut self) {
			self.accumulator -= self.fixed;
			self.absolute += self.fixed;
		}

		pub fn delta(&mut self) -> f32 {
			self.current = PreciseTime::now();
			// self.current - self.last
			self.delta = match self.last.to(self.current).num_nanoseconds() {
				Some(value) => ((value as f32) / 1.0e6),
				None => 0.0,
			};

			if self.delta > self.fixed {	// > 0.25 alternative?
				self.delta = self.fixed;	// = 0.25 alternative?
			}

			self.last = self.current;
			self.accumulator += self.delta;

			self.delta
		}

		pub fn start(&mut self) {
			self.last = PreciseTime::now();
		}

		pub fn interpolation_alpha(&self) -> f32 {
			(self.accumulator / self.fixed)
		}
	}

	pub struct Transform {
		position: Vec3,
		scale: Vec3,
		orientation: Quaternion,
	}

	impl Transform {
		pub fn move_towards(&mut self, direction: Vec3, amount: f32) {
			self.position = self.position + direction.normalized() * amount;
		}

		pub fn rotate(&mut self, axis: Vec3, angle: f32) {
			self.orientation.rotate(axis, angle);
		}

		pub fn model(&self) -> Mat4x4 {
			Mat4x4::new().scale(self.scale) *
			self.orientation.matrix() *
			Mat4x4::new().translate(self.position)
		}

		pub fn mvp(&self, camera: &CameraComponent) -> Mat4x4 {
			camera.view_projection * self.model()
		}
	}

	pub struct CameraComponent {
		view_projection: Mat4x4,
	}

	impl CameraComponent {
		fn new_ortho(width: u32, height: u32, z_near: f32, z_far: f32) -> CameraComponent {
			CameraComponent {
				view_projection: Mat4x4::new()
					.ortho(0.0, width as f32, 0.0, height as f32, z_near, z_far),
			}
		}

		fn new_perspective(fovy: f32, width: u32, height: u32,
			z_near: f32, z_far: f32) -> CameraComponent {
			CameraComponent {
				view_projection: Mat4x4::new()
					.perspective(fovy, width as f32 / height as f32, z_near, z_far),
			}
		}
	}
}

mod math {
	use std::f32;
	use std::f32::consts::PI;
	use std::ops::*;

	#[derive(Copy, Clone)]
	pub struct Mat4x4 {
		m: [f32; 16],
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

		// TODO: rotate() although 3D rotation is better with Quaternions

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
			//			  Orientationmatrix		*	  Translationmatrix
			//		|right.x  up.x  -look.x  0|			  |1 0 0 -eye.x	|
			//		|right.y  up.y  -look.y  0|			  |0 1 0 -eye.x	|
			//		|right.z  up.z  -look.z  0|			  |0 0 1 -eye.x	|
			//		|0 		  0 	0 		 1|			  |0 0 0  1		|

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

	// ____________________________________________________________________________________________
	// Quaternion
	#[derive(Copy, Clone)]
	pub struct Quaternion {
		pub x: f32,
		pub y: f32,
		pub z: f32,
		pub w: f32,
	}

	impl Quaternion {
		pub fn new() -> Quaternion {
			Quaternion { x: 0.0, y: 0.0, z: 0.0, w: 1.0, }
		}

		pub fn set(&mut self, x: f32, y: f32, z: f32, w: f32) -> Quaternion {
			self.x = x;
			self.y = y;
			self.z = z;
			self.w = w;
			*self
		}

		pub fn from_axis(&mut self, axis: Vec3, angle: f32) -> Quaternion {
			let half_rad = angle * PI / 360.0;
			let half_sin = half_rad.sin();
			let half_cos = half_rad.cos();

			self.x = axis.x * half_sin;
			self.y = axis.y * half_sin;
			self.z = axis.z * half_sin;
			self.w = half_cos;

			*self
		}

		pub fn from_euler(&mut self, angles: Vec3) -> Quaternion {
			let rx = angles.x * PI / 360.0;
			let ry = angles.y * PI / 360.0;
			let rz = angles.z * PI / 360.0;

			let sin_x = rx.sin();
			let sin_y = ry.sin();
			let sin_z = -rz.sin();

			let cos_x = rx.cos();
			let cos_y = ry.cos();
			let cos_z = rz.cos();

			let sin_x_sin_y = sin_x * sin_y;
			let cos_x_cos_y = cos_x * cos_y;
			let cos_x_sin_y = cos_x * sin_y;
			let cos_y_sin_x = cos_y * sin_x;

			self.x = cos_x_sin_y * sin_z + cos_y_sin_x * cos_z;
			self.y = cos_x_sin_y * cos_z + cos_y_sin_x * sin_z;
			self.z = cos_x_cos_y * sin_z - sin_x_sin_y * cos_z;
			self.w = cos_x_cos_y * cos_z - sin_x_sin_y * sin_z;

			self.normalize()
		}


		pub fn rotate(&mut self, axis: Vec3, angle: f32) -> Quaternion {
			let half_rad = angle * PI / 360.0;
			let half_sin = half_rad.sin();
			let half_cos = half_rad.cos();

			let axis_norm = axis.normalized();

			let rx = -axis_norm.x * half_sin;
			let ry = -axis_norm.y * half_sin;
			let rz = -axis_norm.z * half_sin;
			let rw = half_cos;

			let (x, y, z, w) = (self.x, self.y, self.z, self.w);

			self.set(
				rw * x + rx * w + ry * z - rz * y,
				rw * y + ry * w + rz * x - rx * z,
				rw * z + rz * w + rx * y - ry * x,
			 	rw * w - rx * x - ry * y - rz * z
			);

			self.normalize()
		}

		pub fn normalized(&self) -> Quaternion {
			let inv_length = 1.0 / self.length();
			let (x, y, z, w) = (self.x, self.y, self.z, self.w);
			Quaternion {
				x: x * inv_length,
				y: y * inv_length,
				z: z * inv_length,
				w: w * inv_length,
			}
		}

		pub fn normalize(&mut self) -> Quaternion {
			let inv_length = 1.0 / self.length();
			let (x, y, z, w) = (self.x, self.y, self.z, self.w);
			self.set(
					x * inv_length,
					y * inv_length,
					z * inv_length,
					w * inv_length
			);

			*self
		}

		pub fn length(&self) -> f32 {
			(self.x * self.x + self.y * self.y + self.z * self.z + self.w * self.w).sqrt()
		}

		pub fn length_squared(&self) -> f32 {
			self.x * self.x + self.y * self.y + self.z * self.z + self.w * self.w
		}

		pub fn dot(&self, q: Quaternion) -> f32  {
			self.x * q.x + self.y * q.y + self.z * q.z + self.w * q.w
		}

		pub fn conjugate(&mut self) -> Quaternion {
			let (x, y, z, w) = (self.x, self.y, self.z, self.w);
			self.x = -x;
			self.y = -y;
			self.z = -z;
			self.w = w;
			*self
		}

		pub fn inverse(&mut self) -> Quaternion {
			let inv_length_squared = 1.0 /
				(self.x * self.x + self.y * self.y + self.z * self.z + self.w * self.w);
			let (x, y, z, w) = (self.x, self.y, self.z, self.w);
			self.set(
					-x * inv_length_squared,
					-y * inv_length_squared,
					-z * inv_length_squared,
					w * inv_length_squared
			);

			*self
		}

		// TODO: Make this work
		// pub fn slerp(q1: &Quaternion, q2: &Quaternion, amount: f32) -> Quaternion {
		// 	let epsilon = 1000.0;
		// 	let mut cos = q1.dot(q2);
		// 	let mut res = Quaternion{ x: q2.x, y: q2.y, z: q2.z, w: q2.w };
		//
		// 	if cos < 0.0 {
		// 		cos = -cos;
		// 		res = Quaternion{ x: -(q2.x), y: -(q2.y), z: -(q2.z), w: -(q2.w) };
		// 	}
		//
		// 	if cos.abs() >= (1.0 - epsilon) {
		// 		return res;
		// 		// return destination.subtr(q1).mult(amount).add(q1).normalize();
		// 	}
		//
		// 	let sin = (1.0 - cos * cos).sqrt();
		// 	let angle = sin.atan2(cos);
		// 	let inv_sin =  1.0 / sin;
		//
		// 	let src_factor = ((1.0 - amount) * angle).sin() * inv_sin;
		// 	let dest_factor = (amount * angle).sin() * inv_sin;
		//
		// 	res // q1.mult(sourceFactor).add(destination.mult(destinationFactor));
		// }

		// TODO: Make this work
		// pub fn nlerp(q1: Quaternion, q2: Quaternion, amount: f32) -> Quaternion {
		// 	let mut res = Quaternion {x: q2.x, y: q2.y, z: q2.z, w: q2.w };
		//
		// 	if q1.dot(&q2) < 0.0 {
		// 		res = Quaternion {x: -(q2.x), y: -(q2.y), z: -(q2.z), w: -(q2.w)};
		// 	}
		//
		// 	*((((res - q1) * amount) + q1).normalize())
		// }

		pub fn matrix(&self) -> Mat4x4 {
			let xx2 = 2.0 * self.x * self.x;
			let xy2 = 2.0 * self.x * self.y;
			let xz2 = 2.0 * self.x * self.z;
			let xw2 = 2.0 * self.x * self.w;
			let yy2 = 2.0 * self.y * self.y;
			let yz2 = 2.0 * self.y * self.z;
			let yw2 = 2.0 * self.y * self.w;
			let zz2 = 2.0 * self.z * self.z;
			let zw2 = 2.0 * self.z * self.w;

			Mat4x4 {
				m: [
					1.0 - (yy2 + zz2), xy2 + zw2, xz2 - yw2, 0.0,
				 	xy2 - zw2, 1.0 - (xx2 + zz2), yz2 + xw2, 0.0,
				 	xz2 + yw2, yz2 - xw2, 1.0 - (xx2 + yy2), 0.0,
				 	0.0, 0.0, 0.0, 1.0
				],
			}

		}

		pub fn forward(&self) -> Vec3 {
			Vec3 {
				x: 2.0 * self.x * self.z + 2.0 * self.y * self.w,
				y: 2.0 * self.y * self.x - 2.0 * self.x * self.w,
				z: 1.0 - (2.0 * self.x * self.x + 2.0 * self.y * self.y),
			}
		}

		pub fn backward(&self) -> Vec3 {
			Vec3 {
				x: -2.0 * self.x * self.z - 2.0 * self.y * self.w,
				y: -2.0 * self.y * self.x + 2.0 * self.x * self.w,
				z: -1.0 + (2.0 * self.x * self.x + 2.0 * self.y * self.y),
			}
		}

		pub fn up(&self) -> Vec3 {
			Vec3 {
				x: 2.0 * self.x * self.y - 2.0 * self.z * self.w,
				y: 1.0 - (2.0 * self.x * self.x + 2.0 * self.z * self.z),
				z: 2.0 * self.y * self.z + 2.0 * self.x * self.w,
			}
		}

		pub fn down(&self) -> Vec3 {
			Vec3 {
				x: -2.0 * self.x * self.y + 2.0 * self.z * self.w,
				y: -1.0 + (2.0 * self.x * self.x + 2.0 * self.z * self.z),
				z: -2.0 * self.y * self.z - 2.0 * self.x * self.w,
			}
		}

		pub fn right(&self) -> Vec3 {
			Vec3 {
				x: -1.0 + (2.0 * self.y * self.y + 2.0 * self.z * self.z),
				y: -2.0 * self.x * self.y - 2.0 * self.z * self.w,
				z: -2.0 * self.x * self.z + 2.0 * self.y * self.w,
			}
		}

		pub fn left(&self) -> Vec3 {
			Vec3 {
				x: 1.0 - (2.0 * self.y * self.y + 2.0 * self.z * self.z),
				y: 2.0 * self.x * self.y + 2.0 * self.z * self.w,
				z: 2.0 * self.x * self.z - 2.0 * self.y * self.w,
			}
		}

	}

	impl Add for Quaternion {
		type Output = Quaternion;

		fn add(self, r: Quaternion) -> Quaternion {
			Quaternion {
				x: self.x + r.x,
				y: self.y + r.y,
				z: self.z + r.z,
				w: self.w + r.w,
			}
		}
	}

	impl Sub for Quaternion {
		type Output = Quaternion;

		fn sub(self, r: Quaternion) -> Quaternion {
			Quaternion {
				x: self.x - r.x,
				y: self.y - r.y,
				z: self.z - r.z,
				w: self.w - r.w,
			}
		}
	}

	impl Mul<Quaternion> for Quaternion {
		type Output = Quaternion;

		fn mul(self, r: Quaternion) -> Quaternion {
			Quaternion {
				x: self.w * r.x + self.x * r.w + self.y * r.z - self.z * r.y,
				y: self.w * r.y + self.y * r.w + self.z * r.x - self.x * r.z,
				z: self.w * r.z + self.z * r.w + self.x * r.y - self.y * r.x,
				w: self.w * r.w - self.x * r.x - self.y * r.y - self.z * r.z,
			}
		}
	}

	impl Mul<f32> for Quaternion {
		type Output = Quaternion;

		fn mul(self, r: f32) -> Quaternion {
			Quaternion { x: self.x * r, y: self.y * r, z: self.z * r, w: self.w * r }
		}
	}

	// ____________________________________________________________________________________________
	// Vec3
	#[derive(Copy, Clone)]
	pub struct Vec3 {
		pub x: f32,
		pub y: f32,
		pub z: f32,
	}

	impl Vec3 {
		pub fn new() -> Vec3 {
			Vec3 { x: 0.0, y: 0.0, z: 0.0 }
		}

		pub fn set(&mut self, x: f32, y: f32, z: f32) -> Vec3 {
			self.x = x;
			self.y = y;
			self.z = z;
			*self
		}

		fn dot(&self, r: Vec3) -> f32  {
			self.x * r.x + self.y * r.y + self.z * r.z
		}

		fn cross(&self, r: Vec3) -> Vec3  {
			Vec3 {
				x: (self.y * r.z) - (self.z * r.y),
				y: (self.z * r.x) - (self.x * r.z),
				z: (self.x * r.y) - (self.y * r.x),
			}
		}

		fn rotate(&mut self, axis: Vec3, angle: f32) -> Vec3 {
			// Rodrigues' Rotation Formula
			// v(rot) = v cos(t) + (axis X v) sin(t) + axis ( axis . v ) (1 - cos(t))
			// v(rot) = a + b + c

			let t = angle * PI / 180.0;
			let sin_t = t.sin();
			let cos_t = t.cos();

			// a = v cos(t)
			let ax = self.x * cos_t;
			let ay = self.y * cos_t;
			let az = self.z * cos_t;

			// b = (axis X v) sin(t)
			let bx = ((axis.y * self.z) - (axis.z * self.y)) * sin_t;
			let by = ((axis.z * self.x) - (axis.x * self.z)) * sin_t;
			let bz = ((axis.x * self.y) - (axis.y * self.x)) * sin_t;

			// c = axis ( axis . v ) (1 - cos(t))
			let scale = self.dot(axis) * (1.0 - cos_t);
			let cx = axis.x * scale;
			let cy = axis.y * scale;
			let cz = axis.z * scale;

			// v(rot) = a + b + c
			self.x = ax + bx + cx;
			self.x = ay + by + cy;
			self.x = az + bz + cz;
			*self
		}

		pub fn normalized(&self) -> Vec3 {
			let inv_length = 1.0 / self.length();
			let (x, y, z) = (self.x, self.y, self.z);
			Vec3 {
				x: x * inv_length,
				y: y * inv_length,
				z: z * inv_length,
			}
		}

		pub fn normalize(&mut self) -> Vec3 {
			let inv_length = 1.0 / self.length();
			let (x, y, z) = (self.x, self.y, self.z);
			self.set(
					x * inv_length,
					y * inv_length,
					z * inv_length,
			);

			*self
		}

		pub fn length(&self) -> f32 {
			(self.x * self.x + self.y * self.y + self.z * self.z).sqrt()
		}

		pub fn length_squared(&self) -> f32 {
			self.x * self.x + self.y * self.y + self.z * self.z
		}

		pub fn distance(v1: Vec3, v2: Vec3) -> f32 {
			(v1 - v2).length()
		}

		pub fn distance_squared(v1: Vec3, v2: Vec3) -> f32 {
			(v1 - v2).length_squared()
		}

		pub fn lerp(v1: Vec3, v2: Vec3, amount: f32) -> Vec3 {
			let diff = 1.0 - amount;
			Vec3 {
				x: diff * v1.x + amount * v2.x,
				y: diff * v1.y + amount * v2.y,
				z: diff * v1.z + amount * v2.z,
			}
		}

		// TODO: Swizzling and create Vec2 struct
	}

	impl Add for Vec3 {
		type Output = Vec3;

		fn add(self, r: Vec3) -> Vec3 {
			Vec3 { x: self.x + r.x, y: self.y + r.y, z: self.z + r.z }
		}
	}

	impl Sub for Vec3 {
		type Output = Vec3;

		fn sub(self, r: Vec3) -> Vec3 {
			Vec3 { x: self.x - r.x, y: self.y - r.y, z: self.z - r.z }
		}
	}

	impl Mul<f32> for Vec3 {
		type Output = Vec3;

		fn mul(self, r: f32) -> Vec3 {
			Vec3 {  x: self.x * r, y: self.y * r, z: self.z * r }
		}
	}

	impl Div<f32> for Vec3 {
		type Output = Vec3;

		fn div(self, r: f32) -> Vec3 {
			let inv = if r != 0.0 { 1.0 / r } else { f32::MAX };
			Vec3 { x: self.x * inv, y: self.y * inv, z: self.z * inv }
		}
	}

	impl Neg for Vec3 {
		type Output = Vec3;

		fn neg(self) -> Vec3 {
			Vec3 { x: -self.x, y: -self.y, z: -self.z }
		}
	}

	// ____________________________________________________________________________________________
	// Vec4
	pub struct Vec4 {
		pub x: f32,
		pub y: f32,
		pub z: f32,
		pub w: f32,
	}

	impl Vec4 {
		pub fn new() -> Vec4 {
			Vec4 { x: 0.0, y: 0.0, z: 0.0, w: 0.0 }
		}
	}
}

// TODO: put Graphics stuff in module graphics

// Textures
// ________________________________________________________________________________________________

pub struct Texture {
	id: GLuint,
}

impl Texture {
	pub fn new() -> Texture {
		Texture {
			id: 0,
		}
	}

	pub fn begin(&self) {
		unsafe {
			gl::BindTexture(gl::TEXTURE_2D, self.id);
		}
	}

	pub fn end(&self) {
		unsafe {
			gl::BindTexture(gl::TEXTURE_2D, 0);
		}
	}

	pub fn load(&mut self, file_path: &str) {
		if file_path.to_lowercase().ends_with(".bmp") {
			self.load_bmp(file_path);
		} else if file_path.to_lowercase().ends_with(".dds") {
			self.load_dds(file_path);
		} else {
			println!("Not a correct image format!");
		}
	}

	// Loading bmp manually for educational purposes use dds
	fn load_bmp(&mut self, file_path: &str) -> io::Result<()> {
		let mut file = try!(File::open(file_path));
		let mut header: [u8; 54] = [0; 54];

		// Header
		try!(file.read(&mut header));

		if (header[0] != 66) || (header[1] != 77) {
			return Err(Error::new(ErrorKind::Other, "Not a bmp file!"));
 		}

		let mut image_size: usize = 0;
		let mut width = 0;
		let mut height = 0;

		unsafe {
			let raw_width = [header[0x12], header[0x13], header[0x14], header[0x15]];
			width = std::mem::transmute::<[u8; 4], i32>(raw_width);
			let raw_height = [header[0x16], header[0x17], header[0x18], header[0x19]];
			height = std::mem::transmute::<[u8; 4], i32>(raw_height);
			let raw_image_size = [header[0x22], header[0x23], header[0x24], header[0x25]];
			image_size = std::mem::transmute::<[u8; 4], u32>(raw_image_size) as usize;
		}

		if image_size == 0 {
			image_size = (width * height * 3) as usize;
		}

		// Data
		let mut data = vec![0; image_size];
		try!(file.read(&mut data)); // Read from where header ended

		// Give data to OpenGL and create texture
		unsafe {
			gl::GenTextures(1, &mut self.id);
			gl::BindTexture(gl::TEXTURE_2D, self.id);

			gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::REPEAT as i32);
    		gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::REPEAT as i32);
			gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as i32);
			gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR as i32);

			gl::TexImage2D(gl::TEXTURE_2D, 0, gl::RGB as i32, width, height,
				0, gl::BGR, gl::UNSIGNED_BYTE, std::mem::transmute(&data[0]));

			gl::GenerateMipmap(gl::TEXTURE_2D);

			data.clear();
			gl::BindTexture(gl::TEXTURE_2D, 0);
		}

		Ok(())
	}

	// Compress png images with AMDCompress(CLI) to dds
	// Only use compressed images (DXT1 = BC1), (DXT3 = BC2), (DXT5 = BC3)
	// For sprites compress to DXT5 for alpha (gradient) channel
	fn load_dds(&mut self, file_path: &str) -> io::Result<()> {
		let mut file = try!(File::open(file_path));
		let mut header: [u8; 128] = [0; 128];

		// Header
		try!(file.read(&mut header));
		if (header[0] != 0x44) || (header[1] != 0x44) ||
			(header[2] != 0x53) || (header[3] != 0x20) {
			return Err(Error::new(ErrorKind::Other, "Not a dds file!"));
 		}

		let raw_height = [header[12], header[13], header[14], header[15]];
		let mut height = unsafe { std::mem::transmute::<[u8; 4], i32>(raw_height) };

		let raw_width = [header[16], header[17], header[18], header[19]];
		let mut width = unsafe { std::mem::transmute::<[u8; 4], i32>(raw_width) };

		let raw_linear_size = [header[20], header[21], header[22], header[23]];
		let linear_size = unsafe { std::mem::transmute::<[u8; 4], u32>(raw_linear_size) };

		let raw_mipmap_count = [header[28], header[29], header[30], header[31]];
		let mipmap_count = unsafe { std::mem::transmute::<[u8; 4], u32>(raw_mipmap_count) };

		let raw_four_cc = [header[84], header[85], header[86], header[87]];
		let four_cc = unsafe { std::mem::transmute::<[u8; 4], u32>(raw_four_cc) };

		// Data
		let image_size = if mipmap_count > 1 { linear_size * 2 } else { linear_size } as usize;
		let mut data = vec![0; image_size];
		try!(file.read(&mut data)); // Read from where header ended

		const FOURCC_DXT1: u32 = 0x31545844;
		const FOURCC_DXT3: u32 = 0x33545844;
		const FOURCC_DXT5: u32 = 0x35545844;
		const COMPRESSED_RGBA_S3TC_DXT1_EXT: u32 = 0x83F1;
		const COMPRESSED_RGBA_S3TC_DXT3_EXT: u32= 0x83F2;
		const COMPRESSED_RGBA_S3TC_DXT5_EXT: u32= 0x83F3;

		let mut format: u32 = 0;
		let mut block_size = 16;

		match four_cc {
			FOURCC_DXT1 => { format = COMPRESSED_RGBA_S3TC_DXT1_EXT; block_size = 8; },
			FOURCC_DXT3 => { format = COMPRESSED_RGBA_S3TC_DXT3_EXT; },
			FOURCC_DXT5 => { format = COMPRESSED_RGBA_S3TC_DXT5_EXT; },
			_ => return Err(Error::new(ErrorKind::Other, "No DXTn specified.")),
		};

		// Give data to OpenGL and create texture
		unsafe {
			gl::GenTextures(1, &mut self.id);
			gl::BindTexture(gl::TEXTURE_2D, self.id);
			gl::PixelStorei(gl::UNPACK_ALIGNMENT,1);

			gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::CLAMP_TO_EDGE as i32);	// REPEAT
			gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::CLAMP_TO_EDGE as i32);	// REPEAT
			gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as i32);
			gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR as i32);
		}

		// Load mipmaps

		let mut level = 0;
		let mut offset = 0;

		while level < mipmap_count && width > 0 && height > 0 {
			let size = ((width+3)/4)*((height+3)/4)*block_size;
			unsafe {
				gl::CompressedTexImage2D(gl::TEXTURE_2D, level as i32, format, width, height,
					0, size, std::mem::transmute(&data[offset as usize]));
			}

			offset += size;
			width /= 2;
			height /= 2;

			level += 1;
		}

		data.clear();
		unsafe {
			gl::BindTexture(gl::TEXTURE_2D, 0);
		}

		Ok(())
	}
}

// Shaders
// ________________________________________________________________________________________________
pub struct Uniform<'a> {
	id: GLint,
	name: &'a str,
}

impl<'a> Uniform<'a> {
	fn new(name: &'a str) -> Uniform<'a>  {
		Uniform { id: 0, name: name }
	}
}

impl<'a> Default for Uniform<'a>  {
	fn default() -> Uniform<'a>  {
		Uniform { id: 0, name: "" }
	}
}

pub struct InternalShader {
	id: GLuint,
}

impl InternalShader {
	pub fn new() -> InternalShader {
		unsafe {
			let id = gl::CreateProgram();
			InternalShader {
				id: id,
			 }
		}
	}

	pub fn compile(&self) {
		unsafe {
			gl::LinkProgram(self.id);
			//gl::ValidateProgram(self.id);
		}
	}

	pub fn vertex_shader(&self, file_path: &str) {
		let code = &*InternalShader::read_code(file_path);
		self.add_shader(code, gl::VERTEX_SHADER);
	}

	pub fn fragment_shader(&self, file_path: &str) {
		let code = &*InternalShader::read_code(file_path);
		self.add_shader(code, gl::FRAGMENT_SHADER); // TODO: read file and return code
	}

	pub fn geometry_shader(&self, file_path: &str) {
		let code = &*InternalShader::read_code(file_path);
		self.add_shader(code, gl::GEOMETRY_SHADER);
	}

	fn add_shader(&self, code: &str, ty: GLenum) {
		unsafe {
			let shader = gl::CreateShader(ty);
			let c_str = CString::new(code.as_bytes()).unwrap();
			gl::ShaderSource(shader, 1, &c_str.as_ptr(), ptr::null());
			gl::CompileShader(shader);
			gl::AttachShader(self.id, shader);
			gl::DeleteShader(shader);
		}
	}

	fn read_code(file_path: &str) -> String {
		let mut file = File::open(file_path).unwrap();
		let mut contents: Vec<u8> = Vec::new();
		file.read_to_end(&mut contents).unwrap();

		String::from_utf8(contents).unwrap()
	}

	pub fn add_uniform(&mut self, uniform: &mut Uniform) {
		unsafe {
			let c_str = CString::new(uniform.name.as_bytes()).unwrap();
			uniform.id = gl::GetUniformLocation(self.id, c_str.as_ptr());
			if uniform.id == -1 {
				panic!("Could not find uniform {}", uniform.name);
			}
		}
	}

	pub fn set_bool(&self, uniform: &Uniform, value: bool) {
		unsafe {
			gl::Uniform1i(uniform.id, match value { true => 1, false => 0 });
		}
	}

	pub fn set_i32(&self, uniform: &Uniform, value: i32) {
		unsafe {
			gl::Uniform1i(uniform.id, value);
		}
	}

	pub fn set_f32(&self, uniform: &Uniform, value: f32) {
		unsafe {
			gl::Uniform1f(uniform.id, value);
		}
	}

	pub fn set_vec4(&self, uniform: &Uniform, value: Vec4) {
		unsafe {
			gl::Uniform4f(uniform.id, value.x, value.y, value.z, value.w);
		}
	}

	pub fn set_mat4x4(&self, uniform: &Uniform, value: &Mat4x4) {
		unsafe {
			gl::UniformMatrix4fv(uniform.id, 1,
				gl::TRUE, std::mem::transmute(value));
		}
	}

	pub fn begin(&self) {
		unsafe {
			gl::UseProgram(self.id);
		}
	}

	pub fn end(&self) {
		unsafe {
			gl::UseProgram(0);
		}
	}
}

impl Drop for InternalShader {
	fn drop(&mut self) {
		unsafe {
			gl::DeleteProgram(self.id);
		};
	}
}

pub trait Shader {
	fn init(&mut self);
	fn begin(&self);
	fn end(&self);
	fn update_uniforms(&self, dt: f32);
}

pub struct BasicShader<'a> {
	shader: InternalShader,
	uniform_transform: Uniform<'a>,
}

impl<'a> BasicShader<'a> {
	pub fn new() -> BasicShader<'a> {
		let shader = InternalShader::new();

		BasicShader {
			shader: shader,
			uniform_transform: Uniform::new("transform"),
		}
	}

	pub fn load_vertex_shader(&self, file_path: &str) {
		self.shader.vertex_shader(file_path);
	}

	pub fn load_fragment_shader(&self, file_path: &str) {
		self.shader.fragment_shader(file_path);
	}
}

impl<'a> Shader for BasicShader<'a> {
	fn init(&mut self) {
		self.shader.compile();
		self.shader.add_uniform(&mut self.uniform_transform);
	}

	fn begin(&self) {
		self.shader.begin();
	}

	fn end(&self) {
		self.shader.end();
	}

	fn update_uniforms(&self, dt: f32) {
		// Unique implementation
		let mut transform = Mat4x4::new().scale(Vec3{ x: 0.5, y: 0.5, z: 0.5 });
		let mut orientation = Quaternion::new().rotate(Vec3{ x: 0.0, y: 0.0, z: 1.0 }, 45.0);
		transform = (orientation.matrix() * transform).translate(Vec3{ x: 0.5, y: -0.5, z: 0.0 });
		self.shader.set_mat4x4(&self.uniform_transform, &transform);
	}
}

// Try to write everything in a modular way

mod engine {
	mod core {
		pub trait CoreSystem {
			fn init(&self);
			fn update(&self, dt: f32);
			fn shutdown(&self);
		}
	}

	mod messaging {
		use engine::graphics::GraphicsSystem;
		use std::sync::mpsc::{ channel, Sender, Receiver };

		pub struct MessageSystem<'a> {
			sender: Sender<Message>,
			receiver: Receiver<Message>,
			graphics_system: Option<&'a GraphicsSystem<'a>>,
		}

		impl<'a> MessageSystem<'a> {
			pub fn new() -> MessageSystem<'a> {
				let (sender, receiver) = channel();
				MessageSystem {
					sender: sender,
					receiver: receiver,
					graphics_system: None::<&'a GraphicsSystem<'a>>,
				}

			}

			pub fn send(&self, msg: Message) {

			}
		}
		pub enum Message {
			Quit,
			Test,
			TestData{ x: f32, y: f32 },
			None,
		}

		pub trait Messager {
			fn send(&self, msg: Message);
			//fn recv(&self);
			fn handle_message(&self, msg: Message);
		}
	}

	// Test Messaging
	// TODO: test with https://doc.rust-lang.org/std/sync/mpsc/index.html
	// for multithreading (can be single threaded if not using thread::spawn()):
	// use std::sync::mpsc::channel;
    //
    // // Create a simple streaming channel
    // let (tx, rx) = channel();	// Sender, Receiver
    // //thread::spawn(move|| {
    //     tx.send(10).unwrap();
    // //});
    // println!("{}", rx.recv().unwrap());

	mod graphics {
		use engine::core::CoreSystem;
		use engine::messaging::*;

		pub struct GraphicsSystem<'a> {
			message_system: &'a MessageSystem<'a>,	// use: message_system.post_message(msg);
		}

		impl<'a> CoreSystem for GraphicsSystem<'a> {
			fn init(&self) {
				println!("Init");
			}

			fn update(&self, dt: f32) {
				println!("Update");
			}

			fn shutdown(&self) {
				println!("Shutdown");
			}
		}

		impl<'a> Messager for GraphicsSystem<'a> {
			fn send(&self, msg: Message) {
				self.message_system.send(msg);
			}

			fn handle_message(&self, msg: Message) {
				match msg {
					Message::TestData{x, y} => { println!("Data: {}, {}", x, y) },
					Message::Test | Message::Quit => { println!("Quit") },
					_ => {}
				}
			}
		}
	}
}

// Quad
static VERTICES: [GLfloat; 32] = [
	// Positions		Colors			Texture Coordinates
	0.5, 0.5, 0.0,		1.0, 0.0, 0.0,	1.0, 1.0,			// Top Right
	-0.5, 0.5, 0.0,		1.0, 1.0, 0.0,	0.0, 1.0,			// Top Left
	-0.5, -0.5, 0.0,	0.0, 1.0, 0.0,	0.0, 0.0,			// Bottom Left
	0.5, -0.5, 0.0,		0.0, 0.0, 1.0,	1.0, 0.0			// Bottom Right
];

static INDICES: [GLuint; 6] = [
	0, 1, 2,   // First Triangle
	2, 3, 0	   // Second Triangle
];

fn main() {
	// Initialize SDL stuff (later in WindowsSystem)

	let sdl_context = sdl2::init().unwrap();
	let video_subsystem = sdl_context.video().unwrap();
	video_subsystem.gl_set_swap_interval(1); // If vsync
	/*
	if settings.get_vsync() {
			video_subsystem.gl_set_swap_interval(1);
		} else {
			video_subsystem.gl_set_swap_interval(0);
		}
	*/
	let gl_attr = video_subsystem.gl_attr();

	// Not all drivers default to 32bit color, so explicitly set it to 32bit color.
	gl_attr.set_red_size(8);
	gl_attr.set_green_size(8);
	gl_attr.set_blue_size(8);
	gl_attr.set_alpha_size(8);
	gl_attr.set_stencil_size(8);
	gl_attr.set_multisample_samples(8); // 8x MSAA
	gl_attr.set_context_version(3 as u8, 3 as u8); // OpenGL 3.3
	gl_attr.set_context_profile(GLProfile::Core);

	let window = video_subsystem.window("Nitrust Oxide", 800, 600)
		.position_centered()
		.opengl()
		//.resizable()
		//.fullscreen()
		.build()
		.unwrap();

	let ctx = window.gl_create_context().unwrap();
	window.gl_make_current(&ctx).unwrap();

	gl::load_with(|name| video_subsystem.gl_get_proc_address(name) as *const _);
	unsafe {
		gl::Viewport(0, 0, 800, 600);
	}

	// Initialize Rendering


	let mut vao = 0;
	let mut vbo = 0;
	let mut ebo = 0;

	unsafe {
		// Create Vertex Array Object
		gl::GenVertexArrays(1, &mut vao);

		gl::BindVertexArray(vao);

		// Create a Vertex Buffer Object and copy the vertex data to it
		gl::GenBuffers(1, &mut vbo);
		gl::BindBuffer(gl::ARRAY_BUFFER, vbo);
		gl::BufferData(gl::ARRAY_BUFFER,
			(VERTICES.len() * std::mem::size_of::<GLfloat>()) as GLsizeiptr,
			std::mem::transmute(&VERTICES[0]),
			gl::STATIC_DRAW);

		// Create a Element Buffer Object and copy the index data to it
		gl::GenBuffers(1, &mut ebo);
		gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, ebo);
		gl::BufferData(gl::ELEMENT_ARRAY_BUFFER,
			(INDICES.len() * std::mem::size_of::<GLuint>()) as GLsizeiptr,
			std::mem::transmute(&INDICES[0]),
			gl::STATIC_DRAW);

		// Position attribute
		gl::VertexAttribPointer(0, 3, gl::FLOAT, gl::FALSE as GLboolean,
			(8 * std::mem::size_of::<GLfloat>()) as i32, 0 as *const _);
		gl::EnableVertexAttribArray(0);

		// Color attribute
		gl::VertexAttribPointer(1, 3, gl::FLOAT, gl::FALSE as GLboolean,
			(8 * std::mem::size_of::<GLfloat>()) as i32,
			(3 * std::mem::size_of::<GLfloat>()) as *const _);
		gl::EnableVertexAttribArray(1);

		// Texture attribute
		gl::VertexAttribPointer(2, 2, gl::FLOAT, gl::FALSE as GLboolean,
			(8 * std::mem::size_of::<GLfloat>()) as i32,
			(6 * std::mem::size_of::<GLfloat>()) as *const _);
		gl::EnableVertexAttribArray(2);

		gl::BindBuffer(gl::ARRAY_BUFFER, 0);
		//gl::BindVertexArray(0);

		// Uncomment for wireframe mode
		//gl::PolygonMode(gl::FRONT_AND_BACK, gl::LINE);

		// Set state back to filled polygons
		//gl::PolygonMode(gl::FRONT_AND_BACK, gl::FILL);

		// Back Face Culling
		gl::Enable(gl::CULL_FACE);
		gl::CullFace(gl::BACK);
		gl::FrontFace(gl::CCW);

		// Depth testing
		gl::Enable(gl::DEPTH_TEST);

		// Alpha blending
		gl::Enable(gl::BLEND);
		gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);
	}

	let mut texture = Texture::new();
	texture.load("./assets/textures/board_alpha.dds");

	let mut shader = BasicShader::new();
	shader.load_vertex_shader("./assets/shaders/basic_shader.vs.glsl");
	shader.load_fragment_shader("./assets/shaders/basic_shader.fs.glsl");
	shader.init();

	// Initialize input
	let mut event_pump = sdl_context.event_pump().unwrap();

	// Initialize clock
	let mut running = true;
	let mut dt: f32 = 0.0;
	let mut clock = Clock::new(60.0);
	clock.start();

	while running {
		dt = clock.delta();

		// Input
		for event in event_pump.poll_iter() {
			match event {
				Event::Quit {..} |
				Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
					running = false;
				},
				_ => {}
			}
		}

		while clock.accumulating() {
			// Do fixed stuff
			//println!("fps: {}", (1.0/dt));

			clock.accumulate();
		}

		// Do non fixed stuff

		// Rendering
		unsafe {
			gl::ClearColor(0.0, 0.0, 0.0, 1.0);
			gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);


			texture.begin();
			shader.begin();
			shader.update_uniforms(dt);
			gl::BindVertexArray(vao);
			gl::DrawElements(gl::TRIANGLES, 6, gl::UNSIGNED_INT, 0 as *const _);
			shader.end();
			texture.end();

			gl::BindVertexArray(0);
		}

		window.gl_swap_window();
		//println!("fps: {}, ms: {}", (1.0/dt), dt);
	}

	// Shutdown

	// Rendering
	unsafe {
		gl::DeleteBuffers(1, &ebo);
		gl::DeleteBuffers(1, &vbo);
		gl::DeleteVertexArrays(1, &vao);
	}
}
