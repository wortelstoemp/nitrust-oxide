use std::f32::consts::PI;
use std::ops::*;

use framework::math::Mat4x4;
use framework::math::Vec3;

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

	pub fn matrix(q: Quaternion) -> Mat4x4 {
		let xx2 = 2.0 * q.x * q.x;
		let xy2 = 2.0 * q.x * q.y;
		let xz2 = 2.0 * q.x * q.z;
		let xw2 = 2.0 * q.x * q.w;
		let yy2 = 2.0 * q.y * q.y;
		let yz2 = 2.0 * q.y * q.z;
		let yw2 = 2.0 * q.y * q.w;
		let zz2 = 2.0 * q.z * q.z;
		let zw2 = 2.0 * q.z * q.w;

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