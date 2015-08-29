use std::fmt::Write;

use nalgebra::{
    cast,
    Mat4,
    Ortho3,
    Iso3,
    Vec2,
    Vec3,
    ToHomogeneous,
};

use client::interface::Frame;
use client::window::Window;
use client::render::base::{color,Shape};
use client::render::draw::{
    GlyphDrawer,
    ShipDrawer,
    ShapeDrawer,
};
use client::render::camera::{Camera};


pub struct Renderer {
    glyph_drawer: GlyphDrawer,
    ship_drawer : ShipDrawer,
    pub camera  : Camera,
}

impl Renderer {
    pub fn new(window: &Window) -> Renderer {
        let mut graphics = window.create_graphics();
        
        let glyph_drawer = GlyphDrawer::new(&mut graphics, 18);
        let ship_drawer  = ShipDrawer::new(&mut graphics);

        Renderer {
            glyph_drawer: glyph_drawer,
            ship_drawer : ship_drawer,
            camera      : Camera::new(),
        }
    }

    /// get new ortho transform matrix based on window size specified
    fn get_transform(size: (u32,u32)) -> Mat4<f32> {
        Ortho3::new(
            size.0 as f32, size.1 as f32,
            -1.0, 1.0
                ).to_mat()
    }

    /// translates transform, used for camera positioning
    fn translate(transform: Mat4<f32>, pos: [f32;2]) -> Mat4<f32> {
        let translation = Iso3::new(
            Vec3::new(pos[0], pos[1], 0.0),
            Vec3::new(0.0, 0.0, 0.0),
            );

        transform * translation.to_homogeneous()
    }

    pub fn render(
        &mut self,
        output : &[String],
        command: (&str,usize),
        frame  : &Frame,
        window : &Window,
        ) {
        let     window_size = window.get_size();
        let     transform   = Renderer::get_transform(window_size);
        let mut graphics    = window.create_graphics();

        graphics.clear();

        let cam_pos = self.camera.update(&frame.ships,None);
        let world_trans = Renderer::translate(transform,cam_pos);

        // render console output
        for (y, line) in output.iter().enumerate() {
            self.glyph_drawer.draw(
                &line,
                position_cli(0, y, window_size),
                color::Colors::white(),
                false,
                transform,
                &mut graphics,
                );
        }

        let mut command_line = String::new();
        let prompt_ypos = 23;

        write!(&mut command_line, "> {}", command.0)
            .unwrap_or_else(|e| panic!("Error writing to String: {}", e));


        self.glyph_drawer.draw(
            &command_line,
            position_cli(0, prompt_ypos, window_size),
            color::Colors::white(),
            false,
            transform,
            &mut graphics,
            );

        //draw cursor position in prompt
        self.glyph_drawer.draw(
            &"_".to_string(),
            position_cli(command.1 + 2, prompt_ypos, window_size),
            color::Colors::white(),
            false,
            transform,
            &mut graphics,
            );


        for (ship_id, ship) in &frame.ships {
            let ship_position = cast(ship.position);
            let ship_size     = self.ship_drawer.ship_size;
            let pos_offset    = Vec2::new(ship_size.x, 10.0);
            // TODO: This should be based on font size, not hardcoded
            let line_advance  = Vec2::new(0.0, -20.0);

            // draw ship velocity line
            let line = Shape::line([0.0,0.0],
                                   [(ship.velocity[0]*30.0) as f32,
                                    (ship.velocity[1]*30.0) as f32],
                                   1.0);
            ShapeDrawer::new(&mut graphics, &line)
                .draw([ship.position[0] as f32,
                       ship.position[1] as f32],
                      [1.0,1.0],
                      color::Colors::red(),
                      world_trans,
                      &mut graphics);


            let mut color = color::Colors::blue();
            if let Some(sid) = frame.ship_id {
                if *ship_id == sid  { color = color::Colors::green_spring(); }
            }
            self.ship_drawer.draw(
                &ship_position,
                color,
                world_trans,
                &mut graphics,
                );

            // draw ship id
            self.glyph_drawer.draw(
                &ship_id.to_string(),
                ship_position - line_advance,
                color::Colors::white(),
                true,
                world_trans,
                &mut graphics,
                );

            // draw ship broadcast
            if let Some(ship_comm) = frame.broadcasts.get(&ship_id) {
                self.glyph_drawer.draw(
                    ship_comm,
                    ship_position + Vec2::new(0.0, -40.0),
                    color::Colors::white(),
                    true,
                    world_trans,
                    &mut graphics,
                    );
            }

            // draw ship position
            let pos = format!("pos: ({}, {})", ship.position[0], ship.position[1]);
            self.glyph_drawer.draw(
                &pos,
                ship_position + pos_offset,
                color::Colors::white(),
                false,
                world_trans,
                &mut graphics,
                );

            // draw ship velocity
            let vel = format!("vel: ({}, {})", ship.velocity[0], ship.velocity[1]);
            self.glyph_drawer.draw(
                &vel,
                ship_position + pos_offset + line_advance,
                color::Colors::white(),
                false,
                world_trans,
                &mut graphics,
                );
        }

        graphics.flush();
    }
}


/// This is used to position CLI text
/// It takes in to account the window sizing
fn position_cli(x: usize, y: usize, window_size: (u32, u32)) -> Vec2<f32> {
        let (width, height) = window_size;

        let pad_x    = 10.0;
        let pad_y    = 30.0;
        let offset_x = 9.0;
        let offset_y = 18.0;

        Vec2::new(
            (-1.0 * ((width as f32 / 2.0) - pad_x)) + offset_x * x as f32,
            ((height as f32 / 2.0) - pad_y) + offset_y * (y as f32 * -1.0),
        )
}
