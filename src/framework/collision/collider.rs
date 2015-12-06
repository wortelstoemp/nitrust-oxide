use framework::math::Vec3;

pub struct AABB {
	pub center: Vec3,
	pub size: Vec3,
}

pub struct Sphere {
	pub center: Vec3,
	pub radius: f32,
}

// TODO: Implement 2D variants
// pub struct AABB2D;
// pub struct Circle2D;