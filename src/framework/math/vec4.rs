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