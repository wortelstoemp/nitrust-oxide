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

mod math {
	pub struct Vec2 {
		x: f32,
		y: f32
	}

	impl Vec2 {
		pub fn new() -> Vec2 {
			Vec2 { x: 0.0, y: 0.0 }
		}
	}
}

// Shader object
struct Shader<'a> {
	program: GLuint,
	uniforms: HashMap<&'a str, i32>,
}

impl<'a> Shader<'a> {
	pub fn new(vertex_path: &str, fragment_path: &str) -> Shader<'a> {
		unsafe {
			let program = gl::CreateProgram();
			Shader {
				program: program,
				uniforms: HashMap::new(),
			 }
		}
	}

	pub fn create_vertex_shader(&self, file: &str) {
		self.add_shader(file, gl::VERTEX_SHADER);
	}

	pub fn create_fragment_shader(&self, file: &str) {
		self.add_shader(file, gl::FRAGMENT_SHADER);
	}

	pub fn create_geometry_shader(&self, file: &str) {
		self.add_shader(file, gl::GEOMETRY_SHADER);
	}

	pub fn add_shader(&self, src: &str, ty: GLuint) {
		unsafe {
			let shader = gl::CreateShader(ty);
			let c_str = CString::new(src.as_bytes()).unwrap();
			gl::ShaderSource(shader, 1, &c_str.as_ptr(), ptr::null());
			gl::CompileShader(shader);
			gl::AttachShader(self.program, shader);
			gl::DeleteShader(shader);
		}
	}

	pub fn compile(&self) {
		unsafe {
			gl::LinkProgram(self.program);
			gl::ValidateProgram(self.program);
		}
	}

	pub fn enable(&self) {
		unsafe {
			gl::UseProgram(self.program);
		}
	}

	pub fn disable(&self) {
		unsafe {
			gl::UseProgram(0);
		}
	}

	pub fn dispose(&self) {
		unsafe {
			gl::DeleteProgram(self.program);
		}
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
static VS_SRC: &'static str =
   "#version 330 core\n\
	layout (location = 0) in vec3 position;\n\
	out vec4 vertexColor;\n\
	void main() {\n\
	   gl_Position = vec4(position.x, position.y, position.z, 1.0);\n\
	   vertexColor = vec4(1.0f, 1.0f, 0.0f, 1.0f);\n\
	}";

static FS_SRC: &'static str =
   "#version 330 core\n\
	in vec4 vertexColor;\n\
	out vec4 out_color;\n\
	void main() {\n\
	   out_color = vertexColor;\n\
	}";

fn compile_shader(src: &str, ty: GLenum) -> GLuint {
	let shader;
	unsafe {
		shader = gl::CreateShader(ty);
		// Attempt to compile the shader
		let c_str = CString::new(src.as_bytes()).unwrap();
		gl::ShaderSource(shader, 1, &c_str.as_ptr(), ptr::null());
		gl::CompileShader(shader);

		// Get the compile status
		let mut status = gl::FALSE as GLint;
		gl::GetShaderiv(shader, gl::COMPILE_STATUS, &mut status);

		// Fail on error
		if status != (gl::TRUE as GLint) {
			let mut len = 0;
			gl::GetShaderiv(shader, gl::INFO_LOG_LENGTH, &mut len);
			let mut buf = Vec::with_capacity(len as usize);
			buf.set_len((len as usize) - 1); // subtract 1 to skip the trailing null character
			gl::GetShaderInfoLog(shader, len, ptr::null_mut(), buf.as_mut_ptr() as *mut GLchar);
			panic!("{}", str::from_utf8(&buf).ok().expect("ShaderInfoLog not valid utf8"));
		}
	}
	shader
}

fn link_program(vs: GLuint, fs: GLuint) -> GLuint { unsafe {
	let program = gl::CreateProgram();
	gl::AttachShader(program, vs);
	gl::AttachShader(program, fs);
	gl::LinkProgram(program);
	// Get the link status
	let mut status = gl::FALSE as GLint;
	gl::GetProgramiv(program, gl::LINK_STATUS, &mut status);

	// Fail on error
	if status != (gl::TRUE as GLint) {
		let mut len: GLint = 0;
		gl::GetProgramiv(program, gl::INFO_LOG_LENGTH, &mut len);
		let mut buf = Vec::with_capacity(len as usize);
		buf.set_len((len as usize) - 1); // subtract 1 to skip the trailing null character
		gl::GetProgramInfoLog(program, len, ptr::null_mut(), buf.as_mut_ptr() as *mut GLchar);
		panic!("{}", str::from_utf8(&buf).ok().expect("ProgramInfoLog not valid utf8"));
	}
	program
} }

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
	let vs = compile_shader(VS_SRC, gl::VERTEX_SHADER);
	let fs = compile_shader(FS_SRC, gl::FRAGMENT_SHADER);
	let program = link_program(vs, fs);

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

			gl::UseProgram(program);
			gl::BindVertexArray(vao);
			//gl::DrawArrays(gl::TRIANGLES, 0, 3);
			gl::DrawElements(gl::TRIANGLES, 6, gl::UNSIGNED_INT, 0 as *const _);
			gl::BindVertexArray(0);
		}

		window.gl_swap_window();
	}

	// Shutdown

	// Rendering
	unsafe {
		gl::DeleteProgram(program);
		gl::DeleteShader(fs);
		gl::DeleteShader(vs);
		gl::DeleteBuffers(1, &ebo);
		gl::DeleteBuffers(1, &vbo);
		gl::DeleteVertexArrays(1, &vao);
	}
}
