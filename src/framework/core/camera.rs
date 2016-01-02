use framework::math::{Mat4x4, Quaternion, Vec3};
use framework::core::Transform;

pub struct Camera {
	pub view_projection: Mat4x4,
}

impl Camera {
    pub fn new_ortho(transform: &Transform,
	width: u32, height: u32, z_near: f32, z_far: f32) -> Camera {
	   Camera {
	       view_projection:
		      Mat4x4::ortho(0.0, width as f32, 0.0, height as f32, z_near, z_far) * 
			  Mat4x4::camera(transform.position, transform.orientation),
		}
    }

    pub fn new_perspective(transform: &Transform, 
	fovy: f32, width: u32, height: u32, z_near: f32, z_far: f32) -> Camera {
       Camera {
	       view_projection:
		      Mat4x4::perspective(fovy, width as f32 / height as f32, z_near, z_far) *
			  Mat4x4::camera(transform.position, transform.orientation),
		}
	}
}