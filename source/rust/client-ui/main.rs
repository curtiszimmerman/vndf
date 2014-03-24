extern crate collections;

extern crate freetype;
extern crate gl;
extern crate glfw;
extern crate stb_image;

extern crate common;

use camera::Camera;
use core::Core;
use entities::Entities;
use ui::{Renderer, Textures, Window};


mod args;
mod camera;
mod components;
mod core;
mod entities;
mod font;
mod images;
mod input;
mod ui;


#[link(name = "stb-image", kind = "static")]
extern {}


fn main() {
	let screen_width  = 800;
	let screen_height = 600;

	let mut core = Core::start(args::get_server_address());

	let     window   = Window::create(screen_width, screen_height);
	let mut textures = Textures::init(window);

	images::load(&mut textures);
	font::load(&mut textures);

	let renderer = Renderer::init(window, textures);

	let mut entities = Entities::new();

	entities.update_asteroid(999, 0.0, 0.0);

	let mut cam = Camera::new();

	let self_id = core.get_self_id();

	while !window.should_close() {
		core.update_positions(entities);
		input::apply(
			&window.glfw_window,
			cam);

		match entities.positions.find(&self_id) {
			Some(ship_position) => {
				cam.x = ship_position.x;
				cam.y = ship_position.y;
				cam.z = ship_position.z;
			},

			_ => ()
		}

		renderer.render(
			window,
			cam,
			&entities.positions,
			&entities.visuals);

		glfw::poll_events();
	}
}
