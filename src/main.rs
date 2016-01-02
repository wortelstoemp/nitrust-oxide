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
use std::io;
use std::io::{ Error, ErrorKind };
use std::io::prelude::*;
use std::fs::File;
use std::ptr;
use std::str;
use std::ffi::CString;

mod framework;
use framework::math::{Mat4x4, Quaternion, Vec3, Vec4};
use framework::graphics::{Texture, InternalShader, Shader, Uniform};
use framework::core::{Camera, Clock, Transform};


// Shaders
// ________________________________________________________________________________________________
pub struct BasicShader<'a> {
	shader: InternalShader,
	uniform_transform: Uniform<'a>,
}

impl<'a> BasicShader<'a> {
	pub fn new() -> BasicShader<'a> {
		let shader = InternalShader::new();

		BasicShader {
			shader: shader,
			uniform_transform: Uniform::new("transform"),
		}
	}

	pub fn load_vertex_shader(&self, file_path: &str) {
		self.shader.vertex_shader(file_path);
	}

	pub fn load_fragment_shader(&self, file_path: &str) {
		self.shader.fragment_shader(file_path);
	}
}

impl<'a> Shader for BasicShader<'a> {
	fn init(&mut self) {
		self.shader.compile();
		self.shader.add_uniform(&mut self.uniform_transform);
	}

	fn begin(&self) {
		self.shader.begin();
	}

	fn end(&self) {
		self.shader.end();
	}

	fn update_uniforms(&self, transform: &Transform, camera: &Camera, dt: f32) {
		// Unique implementation
		self.shader.set_mat4x4(&(self.uniform_transform), &transform.mvp(&camera)); // TODO:  &transform.mvp(&camera)
	}
}

// Try to write everything in a modular way

mod engine {
	mod core {
		pub trait CoreSystem {
			fn init(&self);
			fn update(&self, dt: f32);
			fn shutdown(&self);
		}
	}

	mod messaging {
		use engine::graphics::GraphicsSystem;
		use std::sync::mpsc::{ channel, Sender, Receiver };

		pub struct MessageSystem<'a> {
			sender: Sender<Message>,
			receiver: Receiver<Message>,
			graphics_system: Option<&'a GraphicsSystem<'a>>,
		}

		impl<'a> MessageSystem<'a> {
			pub fn new() -> MessageSystem<'a> {
				let (sender, receiver) = channel();
				MessageSystem {
					sender: sender,
					receiver: receiver,
					graphics_system: None::<&'a GraphicsSystem<'a>>,
				}

			}

			pub fn send(&self, msg: Message) {
				
			}
		}
		pub enum Message {
			Quit,
			Test,
			TestData{ x: f32, y: f32 },
			None,
		}

		pub trait Messenger {
			fn send(&self, msg: Message);
			//fn recv(&self);
			fn handle_message(&self, msg: Message);
		}
	}

	// Test Messaging
	// TODO: test with https://doc.rust-lang.org/std/sync/mpsc/index.html
	// for multithreading (can be single threaded if not using thread::spawn()):
	// use std::sync::mpsc::channel;
    //
    // // Create a simple streaming channel
    // let (tx, rx) = channel();	// Sender, Receiver
    // //thread::spawn(move|| {
    //     tx.send(10).unwrap();
    // //});
    // println!("{}", rx.recv().unwrap());

	mod graphics {
		use engine::core::CoreSystem;
		use engine::messaging::*;

		pub struct GraphicsSystem<'a> {
			message_system: &'a MessageSystem<'a>,	// use: message_system.post_message(msg);
		}

		impl<'a> CoreSystem for GraphicsSystem<'a> {
			fn init(&self) {
				println!("Init");
			}

			fn update(&self, dt: f32) {
				println!("Update");
			}

			fn shutdown(&self) {
				println!("Shutdown");
			}
		}

		impl<'a> Messenger for GraphicsSystem<'a> {
			fn send(&self, msg: Message) {
				self.message_system.send(msg);
			}

			fn handle_message(&self, msg: Message) {
				match msg {
					Message::TestData{x, y} => { println!("Data: {}, {}", x, y) },
					Message::Test | Message::Quit => { println!("Quit") },
					_ => {}
				}
			}
		}
	}
}

// Quad
static VERTICES: [GLfloat; 32] = [
	// Positions		Colors			Texture Coordinates
	0.5, 0.5, 0.0,		1.0, 1.0, 1.0,	1.0, 1.0,			// Top Right
	-0.5, 0.5, 0.0,		1.0, 1.0, 1.0,	0.0, 1.0,			// Top Left
	-0.5, -0.5, 0.0,	1.0, 1.0, 1.0,	0.0, 0.0,			// Bottom Left
	0.5, -0.5, 0.0,		1.0, 1.0, 1.0,	1.0, 0.0			// Bottom Right
];

static INDICES: [GLuint; 6] = [
	0, 1, 2,   // First Triangle
	2, 3, 0	   // Second Triangle
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
	gl_attr.set_multisample_samples(4); // 4x MSAA
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
			(VERTICES.len() * std::mem::size_of::<GLfloat>()) as GLsizeiptr,
			std::mem::transmute(&VERTICES[0]),
			gl::STATIC_DRAW);

		// Create a Element Buffer Object and copy the index data to it
		gl::GenBuffers(1, &mut ebo);
		gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, ebo);
		gl::BufferData(gl::ELEMENT_ARRAY_BUFFER,
			(INDICES.len() * std::mem::size_of::<GLuint>()) as GLsizeiptr,
			std::mem::transmute(&INDICES[0]),
			gl::STATIC_DRAW);

		// Position attribute
		gl::VertexAttribPointer(0, 3, gl::FLOAT, gl::FALSE as GLboolean,
			(8 * std::mem::size_of::<GLfloat>()) as i32, 0 as *const _);
		gl::EnableVertexAttribArray(0);

		// Color attribute
		gl::VertexAttribPointer(1, 3, gl::FLOAT, gl::FALSE as GLboolean,
			(8 * std::mem::size_of::<GLfloat>()) as i32,
			(3 * std::mem::size_of::<GLfloat>()) as *const _);
		gl::EnableVertexAttribArray(1);

		// Texture attribute
		gl::VertexAttribPointer(2, 2, gl::FLOAT, gl::FALSE as GLboolean,
			(8 * std::mem::size_of::<GLfloat>()) as i32,
			(6 * std::mem::size_of::<GLfloat>()) as *const _);
		gl::EnableVertexAttribArray(2);

		gl::BindBuffer(gl::ARRAY_BUFFER, 0);
		//gl::BindVertexArray(0);

		// Uncomment for wireframe mode
		//gl::PolygonMode(gl::FRONT_AND_BACK, gl::LINE);

		// Set state back to filled polygons
		//gl::PolygonMode(gl::FRONT_AND_BACK, gl::FILL);

		// Back Face Culling
		//gl::Enable(gl::CULL_FACE);
		gl::CullFace(gl::BACK);
		gl::FrontFace(gl::CCW);

		// Depth testing
		gl::Enable(gl::DEPTH_TEST);

		// Alpha blending
		gl::Enable(gl::BLEND);
		gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);
	}

    let transform = Transform { 
            position: Vec3{ x: 0.0, y: 0.0, z: 0.0 },
            scale: Vec3{ x: 1.0, y: 1.0, z: 1.0 },
            orientation: Quaternion::from_euler(&Vec3{x: 180.0, y: 0.0, z: 0.0}),
    };
    
    let camera_transform = Transform { 
            position: Vec3{ x: 0.0, y: 0.0, z: 3.0 },
            scale: Vec3{ x: 1.0, y: 1.0, z: 1.0 },
            orientation: Quaternion::from_axis(&Vec3{ x: 0.0, y: 1.0, z: 0.0 }, 180.0),
    };
    
    let camera = Camera::new_perspective(&camera_transform, 45.0, 800, 600, 0.1, 100.0);
    
	let mut texture = Texture::new();
	texture.load("./assets/textures/board_alpha.dds");

	let mut shader = BasicShader::new();
	shader.load_vertex_shader("./assets/shaders/basic_shader.vs.glsl");
	shader.load_fragment_shader("./assets/shaders/basic_shader.fs.glsl");
	shader.init();

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

		// Do non fixed stuff

		// Rendering
		unsafe {
			gl::ClearColor(0.0, 0.0, 0.0, 1.0);
			gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);


			texture.begin();
			shader.begin();
			shader.update_uniforms(&transform, &camera, dt);
			gl::BindVertexArray(vao);
			gl::DrawElements(gl::TRIANGLES, 6, gl::UNSIGNED_INT, 0 as *const _);
			shader.end();
			texture.end();

			gl::BindVertexArray(0);
		}

		window.gl_swap_window();
		//println!("fps: {}, ms: {}", (1.0/dt), dt);
	}

	// Shutdown

	// Rendering
	unsafe {
		gl::DeleteBuffers(1, &ebo);
		gl::DeleteBuffers(1, &vbo);
		gl::DeleteVertexArrays(1, &vao);
	}
}
