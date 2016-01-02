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