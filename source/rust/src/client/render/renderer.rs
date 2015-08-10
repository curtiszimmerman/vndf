use std::fmt::Write;

use nalgebra::{
	cast,
};

use client::interface::Frame;
use client::render::base::Graphics;
use client::window::Window;
use client::render::draw::{
	GlyphDrawer,
	ShipDrawer,
};

pub struct Renderer {
	graphics    : Graphics,
	glyph_drawer: GlyphDrawer,
	ship_drawer : ShipDrawer,

	window_size: (u32,u32),
}

impl Renderer {
	pub fn new(window: &Window) -> Renderer {
		let mut graphics = window.create_graphics();
		
		let size = window.get_size();

		let glyph_drawer = GlyphDrawer::new(&mut graphics, size);
		let ship_drawer  = ShipDrawer::new(&mut graphics, size);

		Renderer {
			graphics    : graphics,
			glyph_drawer: glyph_drawer,
			ship_drawer : ship_drawer,
			window_size: size,
		}
	}

	pub fn render(&mut self,
				  output: &[String],
				  command: (&str,usize),
				  frame: &Frame,
				  window: &Window) {
		self.graphics.clear();

		// NOTE: this is probably tmp fix
		// need glutin resize to work properly
		let _size = window.get_size();
		if self.window_size != _size {
			self.window_size = _size;

			// update transforms
			if _size.0 > 1 && _size.1 > 1 {
				self.glyph_drawer.update(_size);
				self.ship_drawer.update(_size);
			}
		}

		for (y, line) in output.iter().enumerate() {
			let _pos = self.position_cli(0, y);
			self.render_text(&line,
							 _pos,
							 false);
		}
		
		let mut command_line = String::new();
		let prompt_ypos = 23;
		
		write!(&mut command_line, "> {}", command.0)
			.unwrap_or_else(|e| panic!("Error writing to String: {}", e));

		
		let _pos = self.position_cli(0, prompt_ypos);
		self.render_text(&command_line,
						 _pos,
						 false);

		//draw cursor position in prompt
		let _pos = self.position_cli(command.1 + 2,prompt_ypos);
		self.render_text(&"_".to_string(),
						 _pos,
						 false);
		

		for (ship_id, ship) in &frame.ships {
			let mut color = [0.0,0.0,1.0];
			if let Some(sid) = frame.ship_id {
				if *ship_id == sid  { color = [0.0,1.0,0.5]; }
			}
			self.ship_drawer.draw(&mut self.graphics,
								  &cast(ship.position),
								  color);

			// draw ship id
			self.render_text(&ship_id.to_string(),
							 [ship.position[0],ship.position[1]+20.0],
							 true);

			// draw ship broadcast
			if let Some(ship_comm) = frame.broadcasts.get(&ship_id) {
				self.render_text(ship_comm,
								 [ship.position[0],ship.position[1]-40.0],
								 true);
			}

			// draw ship position
			let pos = format!("pos: ({}, {})", ship.position[0], ship.position[1]);
			self.render_text(&pos,
							 [ship.position[0]+30.0,ship.position[1]+10.0],
							 false);

			// draw ship velocity
			let vel = format!("vel: ({}, {})", ship.velocity[0], ship.velocity[1]);
			self.render_text(&vel,
							 [ship.position[0]+30.0,ship.position[1]-10.0],
							 false);
		}

		self.graphics.flush();
	}

	// NOTE: glyph size offset is currently hardcoded to 9px
	fn render_text (&mut self, text: &String, pos: [f64;2], center: bool) {
		let glyph_offset = 9;

		let pos_offset = if center {
			// For reasons I don't fully understand, the text doesn't look sharp
			// when the offset is fractional. We're preventing this here by
			// keeping it as an integer up here and only cast below.
			(glyph_offset * text.chars().count()) / 2
		}
		else {
			0
		};
		
		for (x, c) in text.chars().enumerate() {
			self.glyph_drawer.draw_at(
				(pos[0] - pos_offset as f64 + ((x * glyph_offset) as f64)),
				pos[1],
				c,
				[1.0,1.0,1.0],
				&mut self.graphics,
				);
		}
	}

	/// This is used to position CLI text
	/// It takes in to account the window sizing
	fn position_cli (&self, x: usize, y: usize) -> [f64;2] {
		let (width,height) = self.window_size;
		
		let pad_x = 10.0f64;
		let pad_y = 30.0f64;
		let offset_x = 9.0;
		let offset_y = 18.0;

		[(-1.0 * ((width as f64/2.0) - pad_x)) + offset_x * x as f64,
		 ((height as f64/2.0) - pad_y) + offset_y * (y as f64 * -1.0),]
	}
}
