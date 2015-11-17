// Copyright 2015 Tom Quareme @wortelstoemp
// Note that this file currently is a mess
// When I get basic OpenGL 3.3 running this will evolve
// into an earlier renderer I wrote in LWJGL back in the day.

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

use core::Clock;

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

pub struct Vec4 {
	x: f32,
	y: f32,
	z: f32,
	w: f32,
}

impl Vec4 {
	pub fn new() -> Vec4 {
		Vec4 { x: 0.0, y: 0.0, z: 0.0, w: 0.0 }
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
		self.add_shader(VS_code, gl::VERTEX_SHADER); // TODO: read file and return code
	}

	pub fn fragment_shader(&self, file: &str) {
		self.add_shader(FS_code, gl::FRAGMENT_SHADER); // TODO: read file and return code
	}

	pub fn geometry_shader(&self, file: &str) {
		// self.add_shader(GS_code, gl::GEOMETRY_SHADER); // TODO: read file and return code
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
	fn update_uniforms(&self);
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

	fn update_uniforms(&self) {
		// TODO: Implement
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

// Shader sources
static VS_code: &'static str =
   "#version 330 core\n\
	layout (location = 0) in vec3 position;\n\
	void main() {\n\
	   gl_Position = vec4(position.x, position.y, position.z, 1.0);\n\
	}";

static FS_code: &'static str =
   "#version 330 core\n\
	out vec4 out_color;\n\
	uniform vec4 color;\n\
	void main() {\n\
	   out_color = color;\n\
	}";

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
			shader.set_uniform4f("color", &Vec4{ x: 1.0, y: 1.0, z: 0.0, w: 1.0});
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
