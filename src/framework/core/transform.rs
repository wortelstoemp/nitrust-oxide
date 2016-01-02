use framework::core::Camera;
use framework::math::{Mat4x4, Quaternion, Vec3};

pub struct Transform {
	pub position: Vec3,
	pub scale: Vec3,
	pub orientation: Quaternion,
}

impl Transform {
	pub fn move_towards(&mut self, direction: &Vec3, amount: f32) {
	   self.position = &self.position + &(&direction.normalized() * amount);
	}

	pub fn rotate(&mut self, axis: &Vec3, angle: f32) {
		self.orientation.rotate(axis, angle);
	}

	pub fn model(&self) -> Mat4x4 {
	   &Mat4x4::translation(&self.position) *
	   &Quaternion::matrix(&self.orientation) *
	   Mat4x4::scale(&self.scale)
	}

	pub fn mvp(&self, camera: &Camera) -> Mat4x4 {
       &camera.view_projection * &self.model()
	}
}