// Copyright 2015 Tom Quareme @wortelstoemp
extern crate gl;
extern crate sdl2;
extern crate time;

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

use core::Clock;

fn main() {
	// Initialize SDL stuff (later in WindowsSystem)
	let sdl_context = sdl2::init().unwrap();
	let video_subsystem = sdl_context.video().unwrap();
	let window = video_subsystem.window("Nitrust Oxide", 800, 600)
		.position_centered()
		.opengl()
		//.resizable()
		//.fullscreen()
		.build()
		.unwrap();

	// Initialize clock
	let mut running = true;
	let mut dt: f32 = 0.0;
	let mut clock = Clock::new(60.0);
	clock.start();

	while running {
		dt = clock.delta();

		// Do non fixed stuff

		while clock.accumulating() {
			// Do fixed stuff
		 	//println!("fps: {}", (1.0/dt));

			clock.accumulate();
		}
		// println!("fps: {}", (1.0/dt));

		// Do non fixed stuff

	}
}
