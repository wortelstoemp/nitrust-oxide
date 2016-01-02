extern crate gl;
extern crate std;

use gl::types::*;

use std::ffi::CString;
use std::fs::File;
use std::io::prelude::*;
use std::ptr;

use framework::math::{Mat4x4, Vec4};
use framework::core::{Camera, Transform};

pub trait Shader {
	fn init(&mut self);
	fn begin(&self);
	fn end(&self);
	fn update_uniforms(&self, transform: &Transform, camera: &Camera, dt: f32);
}

pub struct Uniform<'a> {
	id: GLint,
	name: &'a str,
}

impl<'a> Uniform<'a> {
	pub fn new(name: &'a str) -> Uniform<'a>  {
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