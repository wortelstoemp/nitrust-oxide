use std::f32;
use std::f32::consts::PI;
use std::ops::*;

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

	pub fn dot(&self, r: Vec3) -> f32  {
		self.x * r.x + self.y * r.y + self.z * r.z
	}

	pub fn cross(&self, r: Vec3) -> Vec3  {
		Vec3 {
			x: (self.y * r.z) - (self.z * r.y),
			y: (self.z * r.x) - (self.x * r.z),
			z: (self.x * r.y) - (self.y * r.x),
		}
	}

	pub fn rotate(&mut self, axis: Vec3, angle: f32) -> Vec3 {
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