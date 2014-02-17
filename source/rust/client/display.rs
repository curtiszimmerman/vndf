use std::f64;
use std::iter::Iterator;
use std::libc;
use std::mem;
use std::ptr;

use gl;
use glfw;
use glfw::Window;

use camera::Camera;
use texture::Texture;


pub struct PosMap {
	cap: libc::size_t,
	elems: *mut PosMapEntry
}

pub struct PosMapEntry {
	isOccupied: libc::c_int,
	value     : Position
}

pub struct Position {
	x: f32,
	y: f32
}

struct PosMapIter {
	map: PosMap,
	i  : int
}


impl PosMap {
	pub fn new(size: libc::size_t) -> PosMap {
		unsafe {
			let map = PosMap {
				cap  : size,
				elems: libc::malloc(size * mem::size_of::<PosMapEntry>() as u64) as *mut PosMapEntry };

			ptr::zero_memory(map.elems, size as uint);

			map
		}
	}

	pub fn add(&mut self, id: int, pos: Position) {
		unsafe {
			(*self.elems.offset(id)).isOccupied = 1;
			(*self.elems.offset(id)).value      = pos;
		}
	}

	pub fn remove(&mut self, id: int) {
		unsafe {
			(*self.elems.offset(id)).isOccupied = 0;
		}
	}

	pub fn iter(&self) -> ~Iterator<Position> {
		let iter = ~PosMapIter {
			map: *self,
			i  : 0 };

		iter as ~Iterator<Position>
	}
}

impl Iterator<Position> for PosMapIter {
	fn next(&mut self) -> Option<Position> {
		unsafe {
			while (*self.map.elems.offset(self.i)).isOccupied == 0 {
				if self.i as u64 >= self.map.cap {
					return None
				}

				self.i += 1
			}

			let r = if (self.i as u64) < self.map.cap {
				Some((*self.map.elems.offset(self.i)).value)
			}
			else {
				None
			};

			self.i += 1;

			r
		}
	}
}


pub fn init(screen_width: u32, screen_height: u32) -> Window {
	match glfw::init() {
		Err(_) => fail!("Failed to initialize GLFW."),
		_      => ()
	}

	let window = create_window(screen_width, screen_height);
	init_gl(screen_width, screen_height);

	window
}

fn create_window(width: u32, height: u32) -> Window {
	let window_opt = Window::create(
		width, height,
		"Von Neumann Defense Force",
		glfw::Windowed);

	let window = match window_opt {
		Some(window) => window,
		None         => fail!("Failed to create window.")
	};

	window.make_context_current();

	window
}

fn init_gl(screen_width: u32, screen_height: u32) {
	gl::load_with(glfw::get_proc_address);

	gl::Enable(gl::TEXTURE_2D);

	gl::Enable(gl::BLEND);
	gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);

	gl::LoadIdentity();

	// I'm not a 100% sure what this does, but it has to do with using textures
	// that are not power of two. Before I added this call, glTexture2D wouldn't
	// work correctly on an 11x11 texture, causing memory access errors and not
	// displaying it correctly.
	gl::PixelStorei(gl::UNPACK_ALIGNMENT, 1);

	let z_near = 0.1;
	let fov_angle_y = 45.0;
	let half_height =
		f64::tan( fov_angle_y / 360.0 * f64::consts::PI ) * z_near;
	let half_width = half_height * screen_width as f64 / screen_height as f64;
	gl::Frustum(
		-half_width, half_width,
		-half_height, half_height,
		z_near, 1000.0);
}

pub fn render(
	window   : &Window,
	camera   : Camera,
	positions: PosMap,
	texture  : Texture) {

	gl::Clear(gl::COLOR_BUFFER_BIT);

	gl::PushMatrix();

	gl::Translatef(0.0f32, 0.0f32, -500.0f32);
	gl::Rotatef(camera.v, 1.0f32, 0.0f32, 0.0f32);
	gl::Rotatef(camera.h, 0.0f32, 1.0f32, 0.0f32);

	gl::BindTexture(
		gl::TEXTURE_2D,
		texture.name);

	gl::Color4f(1.0f32, 1.0f32, 1.0f32, 1.0f32);

	for position in positions.iter() {
		gl::PushMatrix();

		gl::Translatef(
			position.x - texture.width as f32 / 2f32,
			position.y - texture.height as f32 / 2f32,
			0.0f32);

		gl::Begin(gl::TRIANGLE_STRIP);
			gl::TexCoord2f(1.0f32, 0.0f32);
			gl::Vertex3f(
				texture.width as f32,
				texture.height as f32,
				0.0f32);

			gl::TexCoord2f(1.0f32, 1.0f32);
			gl::Vertex3f(texture.width as f32, 0.0f32, 0.0f32);

			gl::TexCoord2f(0.0f32, 0.0f32);
			gl::Vertex3f(0.0f32, texture.height as f32, 0.0f32);

			gl::TexCoord2f(0.0f32, 1.0f32);
			gl::Vertex3f(0.0f32, 0.0f32, 0.0f32);
		gl::End();

		gl::PopMatrix();
	}

	gl::PopMatrix();
	window.swap_buffers();
}
