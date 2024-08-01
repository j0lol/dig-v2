use macroquad::prelude::*;
use crate::physics::{Actor, World};
use crate::{TILE_SIZE, VIRTUAL_HEIGHT, VIRTUAL_WIDTH};
use crate::tile_map::global_coordinate_to_chunk;

#[derive(Default, Copy, Clone, Debug)]
pub enum Facing {
    Left = 0,
    #[default]
    Forward = 1,
    Right = 2,
}
pub struct Player {
    pub collider: Actor,
    pub speed: Vec2,
    pub facing: Facing,
    pub size: Vec2
}

impl Player {

    pub fn new(world: &mut World) -> Player {
        let position = vec2(VIRTUAL_WIDTH/2., VIRTUAL_HEIGHT/2.);

        Player {
            collider: world.add_actor(position, TILE_SIZE as i32 - 2, TILE_SIZE as i32),
            size: vec2(TILE_SIZE - 2., TILE_SIZE),
            speed: vec2(0., 0.),
            facing: Facing::Forward,
        }
    }
    pub fn position(&self, world: &World) -> Vec2 {
        world.actor_pos(self.collider)
    }

    pub fn update(&mut self, world: &mut World) {

        if is_key_down(KeyCode::X) {
            world.set_actor_position(self.collider,  vec2(VIRTUAL_WIDTH/2., VIRTUAL_HEIGHT/2.));
        }
        let pos = world.actor_pos(self.collider);

        let on_ground = world.collide_check(self.collider, pos + vec2(0., 1.));
        let on_ceil = world.collide_check(self.collider, pos - vec2(0., 1.));

        if on_ground {
            self.speed.y = 0.;
        } else {
            self.speed.y += 500. * get_frame_time();
        }
        if on_ceil {
            self.speed.y = self.speed.y.abs() / 2.;
        }

        let left = is_key_down(KeyCode::A);
        let right = is_key_down(KeyCode::D);

        self.facing = Facing::Forward;
        self.speed.x = 0.;

        if left && !right {
            self.facing = Facing::Left;
            self.speed.x = -100.0;
        } else if right && !left {
            self.facing = Facing::Right;
            self.speed.x = 100.0;
        }
        if is_key_down(KeyCode::Space) && on_ground {
            self.speed.y = -180.;
        }
        
        if is_key_down(KeyCode::F) {
            self.speed.y -= 1000. * get_frame_time();

        }
        
        world.move_h(self.collider, self.speed.x * get_frame_time());
        world.move_v(self.collider, self.speed.y * get_frame_time());

        let chunk_in = global_coordinate_to_chunk(self.position(world));
        if world.map.focus != chunk_in {
            println!("CHUNK LOAD");
            println!("{} -> {}", world.map.focus, chunk_in);
            world.map.focus = chunk_in;
        }

    }

    pub fn draw(&self, world: &World, tile_set: &Texture2D) {

        let position = self.position(world);

        draw_texture_ex(
            tile_set,
            position.x,
            position.y - 1.0,
            WHITE,
            DrawTextureParams {
            dest_size: None,
            source: Some(
                Rect::new(
                    (3. + (self.facing as i32) as f32)*TILE_SIZE,
                    1.*TILE_SIZE, TILE_SIZE, TILE_SIZE)),
            rotation: 0.0,
            flip_x: false,
            flip_y: false,
            pivot: None,
        });
    }

    pub fn get_chunk(&self, world: &World) -> IVec2 {
        (self.position(world) / vec2(VIRTUAL_WIDTH, VIRTUAL_HEIGHT)).floor().as_ivec2()
    }
}