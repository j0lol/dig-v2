use macroquad::prelude::*;
use crate::physics::{Actor, World};
use crate::position::WorldPos;
use crate::tile::TileId;
use crate::ui::draw_from_tile_set;
use crate::{TILE_SET, TILE_SIZE, VIRTUAL_HEIGHT, VIRTUAL_WIDTH};

const MAX_SPEED: f32 = 300.0;
const WALK_SPEED: f32 = 120.0;

const GRAVITY: f32 = 500.0;

const JUMP_IMPULSE: f32 = 1000.0;
pub const JETPACK_IMPULSE: f32 = GRAVITY + 300.0;
pub const JETPACK_TIME: f32 = 0.75;

const PLAYER_W: f32 = TILE_SIZE - 6.0;
const PLAYER_H: f32 = TILE_SIZE;

#[derive(Default, Copy, Clone, Debug)]
pub enum Facing {
    Left = 0,
    #[default]
    Forward = 1,
    Right = 2,
}

#[derive(Copy, Clone, Debug)]
pub enum Jumping {
    Not,
    Jumping,
    Jetpacking(f32)
}

pub struct Player {
    pub collider: Actor,
    pub speed: Vec2,
    pub facing: Facing,
    pub size: Vec2,
    pub jumping: Jumping,
    pub selected_item: u8,
    pub inventory: Box<[TileId; 4]>
}

impl Player {
    pub fn new(world: &mut World) -> Player {
        let position = vec2(VIRTUAL_WIDTH/2., VIRTUAL_HEIGHT/2.);

        Player {
            collider: world.add_actor(position, PLAYER_W as i32, PLAYER_H as i32),
            size: vec2(PLAYER_W, PLAYER_H),
            speed: vec2(0., 0.),
            facing: Facing::Forward,
            jumping: Jumping::Not,
            selected_item: 0,
            inventory: Box::new([TileId::Dirt, TileId::WoodPlanks, TileId::Dirt, TileId::WoodPlanks])
        }
    }
    pub fn update(&mut self, world: &mut World) {
        self.selected_item = self.selected_item.overflowing_add_signed((mouse_wheel().1 as i8).saturating_mul(64)).0;
        
        if is_key_down(KeyCode::X) {
            world.set_actor_position(self.collider,  vec2(VIRTUAL_WIDTH/2., VIRTUAL_HEIGHT/2.));
        }
        let pos = world.actor_pos(self.collider);

        let on_ground = world.collide_check(self.collider, pos + vec2(0., 1.));
        let on_ceil = world.collide_check(self.collider, pos - vec2(0., 1.));

        if on_ground {
            self.speed.y = 0.;
            self.jumping = Jumping::Not;
        } else {
            self.speed.y += GRAVITY * get_frame_time();
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
            self.speed.x = -WALK_SPEED;
        } else if right && !left {
            self.facing = Facing::Right;
            self.speed.x = WALK_SPEED;
        }
        
        match self.jumping {
            Jumping::Not => {
                if is_key_down(KeyCode::Space) && on_ground {
                    self.speed.y = -180.;
                    self.jumping = Jumping::Jumping;
                }
            },
            Jumping::Jumping => {
                if is_key_pressed(KeyCode::Space) {
                    self.speed.y -= JUMP_IMPULSE * get_frame_time();
                    self.jumping = Jumping::Jetpacking(JETPACK_TIME);
                }
            },
            Jumping::Jetpacking(t) if t <= 0.0 => { }
            Jumping::Jetpacking(time_left) => {
                if is_key_down(KeyCode::Space) {
                    self.speed.y -= jetpack_decay_curve(time_left);
                    self.jumping = Jumping::Jetpacking(time_left - get_frame_time());
                }
            },
        }
        
        self.speed.y = self.speed.y.clamp(-MAX_SPEED, MAX_SPEED);
        self.speed.x = self.speed.x.clamp(-MAX_SPEED, MAX_SPEED);
        
        world.move_v(self.collider, self.speed.y * get_frame_time());
        world.move_h(self.collider, self.speed.x * get_frame_time());

        let chunk_in = self.position(world).to_chunk();
        if world.map.focus != chunk_in {
            println!("CHUNK LOAD");
            println!("{} -> {}", world.map.focus.0, chunk_in.0);
            world.map.focus = chunk_in;
        }
    }

    pub fn draw(&self, world: &World) {
        let position = self.position(world);
        draw_from_tile_set(11 + (self.facing as u32), position.0 + vec2(-2.0, -1.0));
    }
    
    pub fn get_inventory_item(&self) -> TileId {
        self.inventory[self.get_inventory_index()]
    }
    pub fn get_inventory_index(&self) -> usize {
        (self.selected_item as usize / (u8::max_value() as f32 * 0.25) as usize).min(3)
    }
    
    pub fn position(&self, world: &World) -> WorldPos {
        WorldPos(world.actor_pos(self.collider))
    }

}

pub fn jetpack_decay_curve(_time_left: f32) -> f32 {
    JETPACK_IMPULSE * get_frame_time()
}