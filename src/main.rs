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

			if self.delta > self.fixed {    // > 0.25 alternative?
				self.delta = self.fixed;    // = 0.25 alternative?
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

// struct TestStructA {
// 	x: i32,
// 	y: i32,
// }
//
// struct TestStructB {
// 	t: Option<TestStructA>,
// 	z: i32,
// }
//
// impl TestStructB {
// 	fn new() -> TestStructB {
// 		TestStructB {
// 			t: None,
// 			z: 0,
// 		}
// 	}
// }

fn main() {
	// let mut testyStr: TestStructB = TestStructB::new();
	// testyStr.t = Some(TestStructA {x: 0, y: 0});
	// Initialize SDL stuff (later in WindowsSystem)

	let sdl_context = sdl2::init().unwrap();
	let video_subsystem = sdl_context.video().unwrap();
	video_subsystem.gl_set_swap_interval(1); // If vsync

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
		unsafe {
            gl::ClearColor(0.3, 0.3, 0.3, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT);
        }

		window.gl_swap_window();
	}
}
