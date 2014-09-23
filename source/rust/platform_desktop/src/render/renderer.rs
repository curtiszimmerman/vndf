use std::collections::HashMap;
use std::rc::Rc;

use cgmath::{
	mod,
	Deg,
	FixedArray,
	Matrix,
	Matrix4,
	Point3,
	Vector,
	Vector2,
	Vector3,
	Vector4,
};
use gfx::{
	mod,
	Device,
	DeviceHelper,
	ToSlice,
};

use font::{
	Font,
	Glyph,
};
use images::{
	Image,
	Images,
};
use physics::Body;
use platform::{
	Camera,
	Frame,
	Input,
};
use window::Window;

use super::{
	Graphics,
	Transform,
	Vertex,
};
use super::grid::{
	Grid,
	GridParams,
};


#[shader_param(IconBatch)]
struct IconParams {
	transform: [[f32, ..4], ..4],
	tex      : gfx::shade::TextureParam,
}


static ICON_VERTEX_SHADER: gfx::ShaderSource = shaders! {
	GLSL_150: b"
		#version 150 core

		uniform mat4 transform;

		in vec3 position;
		in vec2 tex_coord;

		out vec2 tex_coord_f;

		void main()
		{
			gl_Position = transform * vec4(position, 1.0);
			tex_coord_f = tex_coord;
		}
	"
};

static ICON_FRAGMENT_SHADER: gfx::ShaderSource = shaders! {
	GLSL_150: b"
		#version 150 core

		uniform sampler2D tex;

		in vec2 tex_coord_f;

		out vec4 out_color;

		void main()
		{
			out_color = texture(tex, tex_coord_f);
		}
	"
};


pub struct Renderer {
	graphics: Graphics,
	window  : Rc<Window>,

	frame: gfx::Frame,

	grid : Grid,
	icons: HashMap<String, Icon>,

	glyphs: HashMap<char, Glyph>,
}

impl Renderer {
	pub fn new(window: Rc<Window>, images: Images, font: Font) -> Renderer {
		let mut graphics = gfx::Graphics::new(window.new_device());

		let frame = gfx::Frame::new(window.width, window.height);

		let grid = Grid::new(&mut graphics);

		let mut glyphs = HashMap::new();
		let mut icons  = HashMap::new();
		for (path, image) in images.into_iter() {
			icons.insert(path, Icon::from_image(&mut graphics, image));
		}
		for (c, glyph) in font.into_iter() {
			if c != ' ' {
				icons.insert(
					c.to_string(),
					Icon::from_glyph(&mut graphics, &glyph)
				);
			}
			glyphs.insert(c, glyph);
		}

		Renderer {
			graphics: graphics,
			window  : window,

			frame: frame,

			grid : grid,
			icons: icons,

			glyphs: glyphs,
		}
	}

	pub fn render(&mut self, frame: &Frame) {
		let projection = self.perspective();

		self.graphics.clear(
			gfx::ClearData {
				color  : [0.0, 0.0, 0.0, 1.0],
				depth  : 0.0,
				stencil: 0,
			},
			gfx::Color,
			&self.frame
		);

		self.draw_grid(&frame.camera, projection);

		for body in frame.ships.iter() {
			self.draw_craft(
				body,
				&frame.camera,
				"images/spaceship.png"
			);
		}

		for body in frame.missiles.iter() {
			self.draw_craft(
				body,
				&frame.camera,
				"images/missile.png"
			);
		}

		self.draw_ui_overlay(frame.input);

		self.graphics.end_frame();
		self.window.swap_buffers();
	}

	fn draw_grid(&mut self, camera: &Camera, projection: Transform) {
		let grid_camera = Camera {
			center: Vector3::new(
				camera.center[0] % 200.0,
				camera.center[1] % 200.0,
				camera.center[2],
			),

			perspective: camera.perspective,
			distance   : camera.distance,
		};

		let view = camera_to_transform(&grid_camera);

		let params = GridParams {
			transform: projection.mul(&view).into_fixed(),
		};

		self.graphics.draw(
			&self.grid.batch,
			&params,
			&self.frame
		);
	}

	fn draw_craft(&mut self, body: &Body, camera: &Camera, icon_id: &str) {
		let icon = self.icons[icon_id.to_string()];
		let screen_position = self.perspective()
			.mul(&camera_to_transform(camera))
			.mul_v(&Vector4::new(
				body.position[0] as f32,
				body.position[1] as f32,
				body.position[2] as f32,
				1.0,
			));

		let transform = self.ortho()
			.mul(&Matrix4::from_translation(&Vector3::new(
				screen_position.x / screen_position.w * self.window.width as f32,
				screen_position.y / screen_position.w * self.window.height as f32,
				0.0,
			)));

		self.draw_icon(&icon, &transform);

		let mut text_position = icon.size + icon.offset;
		self.draw_text(
			format!("pos: {:i} / {:i} / {:i}",
				body.position.x as int,
				body.position.y as int,
				body.position.z as int,
			)
			.as_slice(),
			&transform.mul(&Matrix4::from_translation(&text_position.extend(0.0))),
		);

		text_position = text_position - Vector2::new(0.0, 15.0);
		self.draw_text(
			format!("vel: {:i} / {:i} / {:i}",
				body.velocity.x as int,
				body.velocity.y as int,
				body.velocity.z as int,
			).as_slice(),
			&transform.mul(&Matrix4::from_translation(&text_position.extend(0.0))),
		);
	}

	fn draw_ui_overlay(&mut self, input: Input) {
		let projection = self.ortho();

		let left   = -(self.window.width as f32) / 2.0;
		let right  = -left;
		let bottom = -(self.window.height as f32) / 2.0;


		self.draw_text(
			"Move camera with WASD; change zoom with R and F",
			&projection.mul(&Matrix4::from_translation(&Vector2::new(left + 20.0, bottom + 60.0).extend(0.0)))
		);
		self.draw_text(
			"Change attitude with the cursor keys, toggle thrust with Space",
			&projection.mul(&Matrix4::from_translation(&Vector2::new(left + 20.0, bottom + 40.0).extend(0.0))),
		);
		self.draw_text(
			"Shoot missiles with Enter",
			&projection.mul(&Matrix4::from_translation(&Vector2::new(left + 20.0, bottom + 20.0).extend(0.0))),
		);

		self.draw_text(
			format!("{}", input.attitude).as_slice(),
			&projection.mul(&Matrix4::from_translation(&Vector2::new(right - 100.0, bottom + 40.0).extend(0.0))),
		);
		self.draw_text(
			if input.thrust { "Thrust ON" } else { "Thrust OFF" },
			&projection.mul(&Matrix4::from_translation(&Vector2::new(right - 100.0, bottom + 20.0).extend(0.0))),
		);
	}

	fn draw_text(&mut self, text: &str, transform: &Transform) {
		let mut total_advance = Vector2::zero();

		for c in text.chars() {
			let (offset, advance) = {
				let ref glyph = self.glyphs[c];
				(glyph.offset, glyph.advance)
			};

			if c != ' ' {
				let icon = self.icons[c.to_string()];

				self.draw_icon(
					&icon,
					&transform.mul(&Matrix4::from_translation(&(offset.extend(0.0) + total_advance.extend(0.0)))),
				);
			}

			total_advance = total_advance + advance;
		}
	}

	fn draw_icon(&mut self, icon: &Icon, transform: &Transform) {
		let params = IconParams {
			transform: transform.mul(&Matrix4::from_translation(&icon.offset.extend(0.0))).into_fixed(),
			tex      : icon.param,
		};

		self.graphics.draw(
			&icon.batch,
			&params,
			&self.frame
		);
	}

	fn ortho(&self) -> Transform {
		cgmath::ortho(
			-(self.window.width  as f32) / 2.0,
			  self.window.width  as f32  / 2.0,
			-(self.window.height as f32) / 2.0,
			  self.window.height as f32  / 2.0,
			-1.0, 1.0,
		)
	}

	fn perspective(&self) -> Transform {
		cgmath::perspective(
			Deg { s: 45.0f32 },
			self.window.width as f32 / self.window.height as f32,
			0.01, 100000.0,
		)
	}
}


struct Icon {
	batch : IconBatch,
	param : gfx::shade::TextureParam,
	size  : Vector2<f32>,
	offset: Vector2<f32>,
}

impl Icon {
	fn from_glyph(graphics: &mut Graphics, glyph: &Glyph) -> Icon {
		let data = Vec::from_fn(
			glyph.data.len() * 4,
			|i| {
				if (i + 1) % 4 == 0 {
					glyph.data[i / 4]
				}
				else {
					255
				}
			}
		);

		Icon::new(
			graphics,
			glyph.size[0],
			glyph.size[1],
			data.as_slice(),
			false,
		)
	}

	fn from_image(graphics: &mut Graphics, image: Image) -> Icon {
		Icon::new(
			graphics,
			image.width as f32,
			image.height as f32,
			image.data.as_slice(),
			true,
		)
	}

	fn new(
		graphics: &mut Graphics,
		width   : f32,
		height  : f32,
		data    : &[u8],
		center  : bool,
	) -> Icon {
		let vertices = [
			Vertex::for_icon([   0.0,    0.0, 0.0 ], [ 0.0, 1.0 ]),
			Vertex::for_icon([ width,    0.0, 0.0 ], [ 1.0, 1.0 ]),
			Vertex::for_icon([   0.0, height, 0.0 ], [ 0.0, 0.0 ]),
			Vertex::for_icon([ width, height, 0.0 ], [ 1.0, 0.0 ]),
		];

		let mesh  = graphics.device.create_mesh(vertices);
		let slice = mesh.to_slice(gfx::TriangleStrip);

		let program = graphics.device
			.link_program(
				ICON_VERTEX_SHADER.clone(),
				ICON_FRAGMENT_SHADER.clone()
			)
			.unwrap_or_else(|error| fail!("error linking program: {}", error));

		let texture_info = gfx::tex::TextureInfo {
			width : width as u16,
			height: height as u16,
			depth : 1,
			levels: -1,
			kind  : gfx::tex::Texture2D,
			format: gfx::tex::RGBA8,
		};

		let texture = graphics.device.create_texture(texture_info).unwrap();
		graphics.device.update_texture(
			&texture,
			&texture_info.to_image_info(),
			data
		)
		.unwrap();

		let sampler = graphics.device.create_sampler(
			gfx::tex::SamplerInfo::new(
				gfx::tex::Bilinear,
				gfx::tex::Clamp
			)
		);

		let batch = graphics
			.make_batch(
				&program,
				&mesh,
				slice,
				&gfx::DrawState::new().blend(gfx::BlendAlpha)
			)
			.unwrap();

		let size   = Vector2::new(width as f32, height as f32);
		let offset = if center { size.mul_s(-0.5) } else { Vector2::zero() };

		Icon {
			batch : batch,
			param : (texture, Some(sampler)),
			size  : size,
			offset: offset,
		}
	}
}


fn camera_to_transform(camera: &Camera) -> Transform {
	let (phi, theta) = camera.perspective;

	let x = camera.distance * theta.s.sin() * phi.s.cos();
	let y = camera.distance * theta.s.sin() * phi.s.sin();
	let z = camera.distance * theta.s.cos();

	Matrix4::look_at(
		&Point3::new(
			(camera.center[0] + x) as f32,
			(camera.center[1] + y) as f32,
			z as f32,
		),
		&Point3::new(
			camera.center[0] as f32,
			camera.center[1] as f32,
			camera.center[2] as f32,
		),
		&Vector3::new(0.0, 0.0, 1.0),
	)
}
