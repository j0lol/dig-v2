use bevy_ecs::component::Component;
use bevy_ecs::query::With;
use bevy_ecs::system::Query;
use macroquad::prelude::*;
use crate::physics2::{move_h, move_v, Collider, CollisionResult};
use crate::position::{RectExtend, WorldPos};
use crate::tile::TileId;
use crate::entity::tile_map::ChunkMap;
use crate::entity::ui::draw_from_tile_set;
use crate::{IS_WASM, TILE_SIZE, VIRTUAL_HEIGHT, VIRTUAL_WIDTH};


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

#[derive(Component)]
pub struct PlayerTag;

#[derive(Component, Debug)]
pub struct Player {
    pub speed: Vec2,
    pub facing: Facing,
    pub size: Vec2,
    pub jumping: Jumping,
    pub selected_item: u8,
    pub inventory: Box<[TileId; 4]>
}

pub fn new_player(chunk_map: &mut ChunkMap) -> (Player, Collider, crate::physics2::Actor) {
    let position = vec2(VIRTUAL_WIDTH/2., VIRTUAL_HEIGHT/2.);

    let (actor, collider) = crate::physics2::add_actor(position, PLAYER_W as i32, PLAYER_H as i32, chunk_map);
    (
        Player {
            size: vec2(PLAYER_W, PLAYER_H),
            speed: vec2(0., 0.),
            facing: Facing::Forward,
            jumping: Jumping::Not,
            selected_item: 0,
            inventory: Box::new([TileId::Dirt, TileId::WoodPlanks, TileId::WoodLog, TileId::GenericOre])
        },
        collider,
        actor
    )
    
}
pub fn move_player(mut v_player: Query<(&mut Player, &mut Collider), With<PlayerTag>>, mut v_phys_world: Query<&mut ChunkMap>) {
    let (mut player, mut collider) = v_player.single_mut();
    let mut world = v_phys_world.single_mut();
    let world = world.as_mut();
    // player.update(world);
    
    let scroll_sensitivity = if IS_WASM {
        16
    } else {
        64
    };
    player.selected_item = player.selected_item.overflowing_add_signed((mouse_wheel().1 as i8).saturating_mul(scroll_sensitivity)).0;
    
    if is_key_down(KeyCode::X) {
        // world.set_actor_position(self.collider,  vec2(VIRTUAL_WIDTH/2., VIRTUAL_HEIGHT/2.));
        collider.pos = vec2(VIRTUAL_WIDTH/2., VIRTUAL_HEIGHT/2.);
    }
    let pos = collider.pos;
    let width = ivec2(collider.width, collider.height).as_vec2();

    let on_ground = world.collide(Rect::from_vecs(pos + vec2(0., 1.), width)) != CollisionResult::Empty;
    let on_ceil = world.collide(Rect::from_vecs(pos - vec2(0., 1.), width)) != CollisionResult::Empty;
    // let on_ground = world.collide_check(self.collider, pos + vec2(0., 1.));
    // let on_ceil = world.collide_check(self.collider, pos - vec2(0., 1.));

    if on_ground {
        player.speed.y = 0.;
        player.jumping = Jumping::Not;
    } else {
        player.speed.y += GRAVITY * get_frame_time();
    }
    if on_ceil {
        player.speed.y = player.speed.y.abs() / 2.;
    }

    let left = is_key_down(KeyCode::A);
    let right = is_key_down(KeyCode::D);

    player.facing = Facing::Forward;
    player.speed.x = 0.;

    if left && !right {
        player.facing = Facing::Left;
        player.speed.x = -WALK_SPEED;
    } else if right && !left {
        player.facing = Facing::Right;
        player.speed.x = WALK_SPEED;
    }
    
    match player.jumping {
        Jumping::Not => {
            if is_key_down(KeyCode::Space) && on_ground {
                player.speed.y = -180.;
                player.jumping = Jumping::Jumping;
            }
        },
        Jumping::Jumping => {
            if is_key_pressed(KeyCode::Space) {
                player.speed.y -= JUMP_IMPULSE * get_frame_time();
                player.jumping = Jumping::Jetpacking(JETPACK_TIME);
            }
        },
        Jumping::Jetpacking(t) if t <= 0.0 => { }
        Jumping::Jetpacking(time_left) => {
            if is_key_down(KeyCode::Space) {
                player.speed.y -= jetpack_decay_curve(time_left);
                player.jumping = Jumping::Jetpacking(time_left - get_frame_time());
            }
        },
    }
    
    player.speed.y = player.speed.y.clamp(-MAX_SPEED, MAX_SPEED);
    player.speed.x = player.speed.x.clamp(-MAX_SPEED, MAX_SPEED);
    
    move_v(world,  collider.as_mut(), player.speed.y * get_frame_time());
    move_h(world, collider.as_mut(), player.speed.x * get_frame_time());

    let chunk_in = WorldPos(collider.pos).to_chunk();
    if world.focus != chunk_in {
        println!("CHUNK LOAD");
        println!("{} -> {}", world.focus.0, chunk_in.0);
        world.focus = chunk_in;
    }
}
pub fn draw_player(v_player: Query<(&Player, &Collider), With<PlayerTag>>) {
    let (player, collider) = v_player.get_single().unwrap();
    
    dbg!(player);
    
    let position = collider.pos;
    draw_from_tile_set(11 + (player.facing as u32), position + vec2(-2.0, -1.0));
}


impl Player {
    pub fn get_inventory_item(&self) -> TileId {
        self.inventory[self.get_inventory_index()]
    }
    pub fn get_inventory_index(&self) -> usize {
        (self.selected_item as usize / (u8::max_value() as f32 * 0.25) as usize).min(3)
    }
}

pub fn jetpack_decay_curve(_time_left: f32) -> f32 {
    JETPACK_IMPULSE * get_frame_time()
}