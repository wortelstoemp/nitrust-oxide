use framework::collision::{AABB, Sphere};
use framework::math::Vec3;

pub fn intersects_AABB(a: &AABB, b: &AABB) -> bool {
	if (a.center.x - b.center.x).abs() > (a.size.x + b.size.x) {
		return false;
	}
	
	if (a.center.y - b.center.y).abs() > (a.size.y + b.size.y) {
		return false;
	}
	
	if (a.center.z - b.center.z).abs() > (a.size.z + b.size.z) {
		return false;
	}

	true
}

pub fn intersects_Sphere(a: &Sphere, b: &Sphere) -> bool {
	let radius_sum = a.radius + b.radius;
	let radius_sum_squared = radius_sum * radius_sum;
	let center_distance_squared = Vec3::distance_squared(&b.center, &a.center);
	
	center_distance_squared <= radius_sum_squared
}