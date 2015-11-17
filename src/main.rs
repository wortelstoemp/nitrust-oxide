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
use std::mem;
use std::ptr;
use std::str;
use std::ffi::CString;

use math::*;
use core::*;

mod core {
	use time::PreciseTime;

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
}

mod math {
	use std::ops::*;
	use std::f32::consts::PI;

	pub struct Mat4x4 {
		m: [f32; 16],
	}

	impl Mat4x4 {
		fn new() -> Mat4x4 {
			Mat4x4 {
				m: [0.0; 16],
			}
		}

		fn diagonal(&mut self, d: f32) -> &Mat4x4 {
			self.m = [
				d, 0.0, 0.0, 0.0,
				0.0, d, 0.0, 0.0,
				0.0, 0.0, d, 0.0,
				0.0, 0.0, 0.0, d,
			];
			self
		}

		fn identity(&mut self) -> &Mat4x4 {
			self.m = [
				1.0, 0.0, 0.0, 0.0,
				0.0, 1.0, 0.0, 0.0,
				0.0, 0.0, 1.0, 0.0,
				0.0, 0.0, 0.0, 1.0,
			];
			self
		}

		fn transpose(&mut self) -> &Mat4x4 {
			let t = self.m;
			self.m = [
				t[0], t[4], t[8], t[12],
				t[1], t[5], t[9], t[13],
				t[2], t[6], t[10], t[14],
				t[3], t[7], t[11], t[15],
			];
			self
		}

		fn translate(&mut self, t: &Vec3) -> &Mat4x4 {
			self.m = [
				1.0, 0.0, 0.0, t.x,
				0.0, 1.0, 0.0, t.y,
				0.0, 0.0, 1.0, t.z,
				0.0, 0.0, 0.0, 1.0,
			];
			self
		}

		fn scale(&mut self, s: &Vec3) -> &Mat4x4 {
			self.m = [
				s.x, 0.0, 0.0, 0.0,
				0.0, s.y, 0.0, 0.0,
				0.0, 0.0, s.z, 0.0,
				0.0, 0.0, 0.0, 1.0,
			];
			self
		}

		fn mirror(&mut self) -> &Mat4x4 {
			self.m = [
				-1.0, 0.0, 0.0, 0.0,
				0.0, -1.0, 0.0, 0.0,
				0.0, 0.0, -1.0, 0.0,
				0.0, 0.0, 0.0, 1.0,
			];
			self
		}

		// TODO: rotate()

		fn ortho(&mut self, left: f32, right: f32, bottom: f32, top: f32, z_near: f32, z_far: f32) -> &Mat4x4 {
			let width = right - left;
			let height = top - bottom;
			let depth = z_far - z_near;

			self.m = [
    			2.0/width, 0.0, 0.0, -(right+left)/width,
    			0.0, 2.0/height, 0.0, -(top+bottom)/height,
    			0.0, 0.0, -2.0/depth, -(z_far + z_near)/depth,
    			0.0, 0.0, 0.0, 0.0,
    		];
			self
		}

		fn perspective(&mut self, fovy: f32, aspect_ratio: f32, z_near: f32, z_far: f32) -> &Mat4x4 {
			let rad: f32 = (fovy / 2.0) * PI / 180.0;
			let y_scale: f32 = 1.0 / rad.tan();
			let x_scale: f32 = y_scale / aspect_ratio;
			let frustum_length: f32 = z_far - z_near;

			self.m = [
				x_scale, 0.0, 0.0, 0.0,
				0.0, y_scale, 0.0, 0.0,
				0.0, 0.0, (z_far + z_near) / frustum_length, (-2.0) * z_near * z_far / frustum_length,
				0.0, 0.0, 1.0, 0.0,
			];
			self
		}

		// TODO: Implement
		fn look_at(&mut self, eye: &Vec3, look: &Vec3, up: &Vec3) -> &Mat4x4 {
			self
		}

		// TODO: Implement
		fn camera(&mut self, position: &Vec3, orientation: &Quaternion) -> &Mat4x4 {
			self
		}
	}

	impl Add for Mat4x4 {
		type Output = Mat4x4;

		fn add(self, o: Mat4x4) -> Mat4x4 {
			let mut res = Mat4x4::new();
			res.m = [
				self.m[0] + o.m[0],	self.m[1] + o.m[1], self.m[2] + o.m[2], self.m[3] + o.m[3],
				self.m[4] + o.m[4], self.m[5] + o.m[5], self.m[6] + o.m[6], self.m[7] + o.m[7],
				self.m[8] + o.m[8], self.m[9] + o.m[9], self.m[10] + o.m[10], self.m[11] + o.m[11],
				self.m[12] + o.m[12], self.m[13] + o.m[13], self.m[14] + o.m[14], self.m[15] + o.m[15],
			];
			res
		}
	}

	impl Sub for Mat4x4 {
		type Output = Mat4x4;

		fn sub(self, o: Mat4x4) -> Mat4x4 {
			let mut res = Mat4x4::new();
			res.m = [
				self.m[0] - o.m[0],	self.m[1] - o.m[1], self.m[2] - o.m[2], self.m[3] - o.m[3],
				self.m[4] - o.m[4], self.m[5] - o.m[5], self.m[6] - o.m[6], self.m[7] - o.m[7],
				self.m[8] - o.m[8], self.m[9] - o.m[9], self.m[10] - o.m[10], self.m[11] - o.m[11],
				self.m[12] - o.m[12], self.m[13] - o.m[13], self.m[14] - o.m[14], self.m[15] - o.m[15],
			];
			res
		}
	}

	// ____________________________________________________________________________________________
	// Quaternion
	pub struct Quaternion {
		pub x: f32,
		pub y: f32,
		pub z: f32,
		pub w: f32,
	}

	impl Quaternion {
		fn new() -> Quaternion {
			Quaternion { x: 0.0, y: 0.0, z: 0.0, w: 1.0, }
		}

		fn set(&mut self, x: f32, y: f32, z: f32, w: f32) -> &Quaternion {
			self.x = x;
			self.y = y;
			self.z = z;
			self.w = w;
			self
		}

		fn from_axis(&mut self, axis: &Vec3, angle: f32) -> &Quaternion {
			let half_rad: f32 = angle * PI / 360.0;
			let half_sin: f32 = half_rad.sin();
			let half_cos: f32 = half_rad.cos();

			self.x = axis.x * half_sin;
			self.y = axis.y * half_sin;
			self.z = axis.z * half_sin;
			self.w = half_cos;

			self
		}

		fn from_euler(&mut self, angles: &Vec3) -> &Quaternion {
			let rx = angles.x * PI / 180.0;
			let ry = angles.y * PI / 180.0;
			let rz = angles.z * PI / 180.0;

			let sin_pitch = (rx * 0.5).sin();
			let cos_pitch = (rx * 0.5).cos();
			let sin_yaw = (ry * 0.5).sin();
			let cos_yaw = (ry * 0.5).cos();
			let sin_roll = (rz * 0.5).sin();
			let cos_roll = (rz * 0.5).cos();
			let cos_yaw_cos_roll = cos_yaw * cos_roll;
			let cos_yaw_sin_roll = cos_yaw * sin_roll;
			let sin_yaw_sin_roll = sin_yaw * sin_roll;
			let sin_yaw_cos_roll = sin_yaw * cos_roll;

			self.x = -sin_pitch * cos_yaw_cos_roll - cos_pitch * sin_yaw_sin_roll;
	    	self.y = -cos_pitch * sin_yaw_cos_roll + sin_pitch * cos_yaw_sin_roll;
	    	self.z = -cos_pitch * cos_yaw_sin_roll - sin_pitch * sin_yaw_cos_roll;
	    	self.w =  cos_pitch * cos_yaw_cos_roll - sin_pitch * sin_yaw_sin_roll;

			self
		}

		fn rotate(&mut self, axis: &Vec3, angle: f32) -> &Quaternion {
			let half_rad: f32 = angle * PI / 360.0;
			let half_sin: f32 = half_rad.sin();
			let half_cos: f32 = half_rad.cos();

			let rx = -axis.x * half_sin;
			let ry = -axis.y * half_sin;
			let rz = -axis.z * half_sin;
			let rw = half_cos;

			let (x, y, z, w) = (self.x, self.y, self.z, self.w);

			self.set(
				rw * x + rx * w + ry * z - rz * y,
				rw * y + ry * w + rz * x - rx * z,
				rw * z + rz * w + rx * y - ry * x,
			 	rw * w - rx * x - ry * y - rz * z
			);

			self
		}

		fn normalize(&mut self) -> &Quaternion {
			let inv_length = 1.0 / self.length();
			let (x, y, z, w) = (self.x, self.y, self.z, self.w);
			self.set(
					x * inv_length,
					y * inv_length,
					z * inv_length,
					w * inv_length
			);

			self
		}

		fn length(&self) -> f32 {
			(self.x * self.x + self.y * self.y + self.z * self.z + self.w * self.w).sqrt()
		}

		fn length_squared(&self) -> f32 {
			self.x * self.x + self.y * self.y + self.z * self.z + self.w * self.w
		}

		fn dot(&self, q: &Quaternion) -> f32  {
			self.x * q.x + self.y * q.y + self.z * q.z + self.w * q.w
		}

		fn conjugate(&mut self) -> &Quaternion {
			let (x, y, z, w) = (self.x, self.y, self.z, self.w);
			self.x = -x;
			self.y = -y;
			self.z = -z;
			self.w = w;
			self
		}

		fn inverse(&mut self) -> &Quaternion {
			let inv_length_squared = 1.0 / (self.x * self.x + self.y * self.y + self.z * self.z + self.w * self.w);
			let (x, y, z, w) = (self.x, self.y, self.z, self.w);
			self.set(
					-x * inv_length_squared,
					-y * inv_length_squared,
					-z * inv_length_squared,
					w * inv_length_squared
			);

			self
		}

		/*

		*/
		fn slerp(q1: &Quaternion, q2: &Quaternion, amount: f32) -> Quaternion {
			let epsilon = 1000.0;
			let mut cos = q1.dot(q2);
			let mut res = Quaternion{ x: q2.x, y: q2.y, z: q2.z, w: q2.w };

			if cos < 0.0 {
				cos = -cos;
				res = Quaternion{ x: -(q2.x), y: -(q2.y), z: -(q2.z), w: -(q2.w) };
			}

			if cos.abs() >= (1.0 - epsilon) {
				return res;
				// return destination.subtr(q1).mult(amount).add(q1).normalize();
			}

			let sin = (1.0 - cos * cos).sqrt();
			let angle = sin.atan2(cos);
			let inv_sin =  1.0 / sin;

			let src_factor = ((1.0 - amount) * angle).sin() * inv_sin;
			let dest_factor = (amount * angle).sin() * inv_sin;

			res // q1.mult(sourceFactor).add(destination.mult(destinationFactor));
		}

		fn nlerp(q1: &Quaternion, q2: &Quaternion, amount: f32) -> Quaternion {
			let mut res = Quaternion {x: q2.x, y: q2.y, z: q2.z, w: q2.w };

			if q1.dot(q2) < 0.0 {
				res = Quaternion {x: -(q2.x), y: -(q2.y), z: -(q2.z), w: -(q2.w)};
			}

			res // (((res - q1) * amount) + q1).normalize()
		}

		fn matrix(&self) -> Mat4x4 {
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

		fn forward(&self) -> Vec3 {
			Vec3 {
				x: 2.0 * self.x * self.z + 2.0 * self.y * self.w,
				y: 2.0 * self.y * self.x - 2.0 * self.x * self.w,
				z: 1.0 - (2.0 * self.x * self.x + 2.0 * self.y * self.y),
			}
		}

		fn backward(&self) -> Vec3 {
			Vec3 {
				x: -2.0 * self.x * self.z - 2.0 * self.y * self.w,
				y: -2.0 * self.y * self.x + 2.0 * self.x * self.w,
				z: -1.0 + (2.0 * self.x * self.x + 2.0 * self.y * self.y),
			}
		}

		fn up(&self) -> Vec3 {
			Vec3 {
				x: 2.0 * self.x * self.y - 2.0 * self.z * self.w,
				y: 1.0 - (2.0 * self.x * self.x + 2.0 * self.z * self.z),
				z: 2.0 * self.y * self.z + 2.0 * self.x * self.w,
			}
		}

		fn down(&self) -> Vec3 {
			Vec3 {
				x: -2.0 * self.x * self.y + 2.0 * self.z * self.w,
				y: -1.0 + (2.0 * self.x * self.x + 2.0 * self.z * self.z),
				z: -2.0 * self.y * self.z - 2.0 * self.x * self.w,
			}
		}

		fn right(&self) -> Vec3 {
			Vec3 {
				x: -1.0 + (2.0 * self.y * self.y + 2.0 * self.z * self.z),
				y: -2.0 * self.x * self.y - 2.0 * self.z * self.w,
				z: -2.0 * self.x * self.z + 2.0 * self.y * self.w,
			}
		}

		fn left(&self) -> Vec3 {
			Vec3 {
				x: 1.0 - (2.0 * self.y * self.y + 2.0 * self.z * self.z),
				y: 2.0 * self.x * self.y + 2.0 * self.z * self.w,
				z: 2.0 * self.x * self.z - 2.0 * self.y * self.w,
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

		fn mul(self, f: f32) -> Quaternion {
			Quaternion { x: self.x * f, y: self.y * f, z: self.z * f, w: self.w * f }
    	}
	}

	// ____________________________________________________________________________________________
	// Vec3
	pub struct Vec3 {
		pub x: f32,
		pub y: f32,
		pub z: f32,
	}

	impl Vec3 {
		pub fn new() -> Vec3 {
			Vec3 { x: 0.0, y: 0.0, z: 0.0, }
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


// InternalShader object
pub struct InternalShader<'a> {
	program: GLuint,
	uniforms: HashMap<&'a str, GLint>,
}

impl<'a> InternalShader<'a> {
	pub fn new() -> InternalShader<'a> {
		unsafe {
			let program = gl::CreateProgram();
			InternalShader {
				program: program,
				uniforms: HashMap::new(),
			 }
		}
	}

	pub fn compile(&self) {
		unsafe {
			gl::LinkProgram(self.program);
			//gl::ValidateProgram(self.program);
		}
	}

	pub fn vertex_shader(&self, file: &str) {
		let code =
		"#version 330 core\n\
	 	layout (location = 0) in vec3 position;\n\
	 	void main() {\n\
	 	   gl_Position = vec4(position.x, position.y, position.z, 1.0);\n\
	 	}";
		self.add_shader(code, gl::VERTEX_SHADER); // TODO: read file and return code
	}

	pub fn fragment_shader(&self, file: &str) {
		let code =
		"#version 330 core\n\
	 	out vec4 out_color;\n\
	 	uniform vec4 color;\n\
	 	void main() {\n\
	 	   out_color = color;\n\
	 	}";
		self.add_shader(code, gl::FRAGMENT_SHADER); // TODO: read file and return code
	}

	pub fn geometry_shader(&self, file: &str) {
		let code = "";
		self.add_shader(code, gl::GEOMETRY_SHADER); // TODO: read file and return code
	}

	fn add_shader(&self, code: &str, ty: GLenum) {
		unsafe {
			let shader = gl::CreateShader(ty);
			let c_str = CString::new(code.as_bytes()).unwrap();
			gl::ShaderSource(shader, 1, &c_str.as_ptr(), ptr::null());
			gl::CompileShader(shader);
			gl::AttachShader(self.program, shader);
			gl::DeleteShader(shader);
		}
	}

	pub fn add_uniform(&mut self, uniform: &'a str) {
		unsafe {
			let c_str = CString::new(uniform.as_bytes()).unwrap();
			let uniform_location = gl::GetUniformLocation(self.program, c_str.as_ptr());

			if uniform_location == -1 {
				panic!("Could not find uniform {}", uniform);
			}

			self.uniforms.insert(uniform, uniform_location);
		}
	}

	pub fn set_uniform4f(&self, uniform: &'a str, value: &Vec4)
	{
		unsafe {
			gl::Uniform4f(*self.uniforms.get(uniform).unwrap(), value.x, value.y, value.z, value.w);
		}
	}

	pub fn begin(&self) {
		unsafe {
			gl::UseProgram(self.program);
		}
	}

	pub fn end(&self) {
		unsafe {
			gl::UseProgram(0);
		}
	}

	pub fn delete(&self) {
		unsafe {
			gl::DeleteProgram(self.program);
		}
	}
}

pub trait Shader {
	fn init(&mut self);
	fn begin(&self);
	fn end(&self);
	fn delete(&self);
	fn update_uniforms(&self, dt: f32);
	fn set_uniform4f(&self, uniform: &str, value: &Vec4);
}

pub struct BasicShader<'a> {
	shader: InternalShader<'a>,
}

impl<'a> BasicShader<'a> {
	fn new(vertex_shader_file: &str, fragment_shader_file: &str) -> BasicShader<'a> {
		let mut shader = InternalShader::new();
		shader.vertex_shader(vertex_shader_file);
		shader.fragment_shader(fragment_shader_file);

		BasicShader {
			shader: shader,
		}
	}
}

impl<'a> Shader for BasicShader<'a> {
	fn init(&mut self) {
		self.shader.compile();
		self.shader.add_uniform("color");
	}

	fn begin(&self) {
		self.shader.begin();
	}

	fn end(&self) {
		self.shader.end();
	}

	fn delete(&self) {
		self.shader.delete();
	}

	fn update_uniforms(&self, dt: f32) {
		self.set_uniform4f("color", &Vec4{ x: 1.0, y: 1.0, z: 0.0, w: 1.0});
	}

	fn set_uniform4f(&self, uniform: &str, value: &Vec4) {
		self.shader.set_uniform4f(uniform, value);
	}
}

// Vertex data
static VERTEX_DATA: [GLfloat; 12] = [
	0.5, 0.5, 0.0,		// Top Right
	0.5, -0.5, 0.0,		// Bottom Right
	-0.5, -0.5, 0.0,	// Bottom Left
	-0.5, 0.5, 0.0		// Top Left
];

static INDICES: [GLuint; 6] = [
	0, 1, 3,   // First Triangle
	1, 2, 3	   // Second Triangle
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
	let mut shader = BasicShader::new("basic.vs.glsl", "basic.fs.glsl"); // TODO: change to real files
	shader.init();

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
			(VERTEX_DATA.len() * mem::size_of::<GLfloat>()) as GLsizeiptr,
			mem::transmute(&VERTEX_DATA[0]),
			gl::STATIC_DRAW);

		// Create a Element Buffer Object and copy the index data to it
		gl::GenBuffers(1, &mut ebo);
		gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, ebo);
		gl::BufferData(gl::ELEMENT_ARRAY_BUFFER,
			(INDICES.len() * mem::size_of::<GLuint>()) as GLsizeiptr,
			mem::transmute(&INDICES[0]),
			gl::STATIC_DRAW);

		gl::VertexAttribPointer(0, 3, gl::FLOAT, gl::FALSE as GLboolean, (3 * mem::size_of::<GLfloat>()) as i32, 0 as *const _);
		gl::EnableVertexAttribArray(0);

		gl::BindBuffer(gl::ARRAY_BUFFER, 0);
		gl::BindVertexArray(0);

		// Uncomment for wireframe mode
		//gl::PolygonMode(gl::FRONT_AND_BACK, gl::LINE);

		// Set state back to filled polygons
		//gl::PolygonMode(gl::FRONT_AND_BACK, gl::FILL);
	}

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
		// println!("fps: {}", (1.0/dt));

		// Do non fixed stuff

		// Rendering
		unsafe {
			gl::ClearColor(0.2, 0.3, 0.3, 1.0);
			gl::Clear(gl::COLOR_BUFFER_BIT);

			gl::BindVertexArray(vao);

			shader.begin();
			shader.update_uniforms(dt);
			gl::DrawElements(gl::TRIANGLES, 6, gl::UNSIGNED_INT, 0 as *const _);
			shader.end();

			gl::BindVertexArray(0);
		}

		window.gl_swap_window();
	}

	// Shutdown

	// Rendering
	unsafe {
		shader.delete();
		gl::DeleteBuffers(1, &ebo);
		gl::DeleteBuffers(1, &vbo);
		gl::DeleteVertexArrays(1, &vao);
	}
}
