use macroquad::prelude::*;
use crate::physics::{Actor, World};
use crate::{TILE_SIZE, VIRTUAL_HEIGHT, VIRTUAL_WIDTH};

#[derive(Default, Copy, Clone, Debug)]
pub(crate) enum Facing {
    Left = 0,
    #[default]
    Forward = 1,
    Right = 2,
}
pub struct Player {
    pub(crate) collider: Actor,
    pub(crate) position: Vec2,
    pub(crate) speed: Vec2,
    pub(crate) facing: Facing,
}

impl Player {

    pub fn new(world: &mut World) -> Player {
        let position = vec2(VIRTUAL_WIDTH/2., VIRTUAL_HEIGHT/2.);

        Player {
            collider: world.add_actor(position, TILE_SIZE as i32, TILE_SIZE as i32),
            position,
            speed: vec2(0., 0.),
            facing: Facing::Forward,
        }
    }

    pub fn update(&mut self, world: &mut World) {
        let pos = world.actor_pos(self.collider);
        self.position = pos;

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

        world.move_h(self.collider, self.speed.x * get_frame_time());
        world.move_v(self.collider, self.speed.y * get_frame_time());


    }

    pub fn draw(&self, tile_set: &Texture2D) {

        draw_texture_ex(
            tile_set,
            self.position.x,
            self.position.y,
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
}