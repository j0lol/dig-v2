use std::collections::HashSet;
use macroquad::math::{ivec2, vec2, Rect, Vec2};
use bevy_ecs::component::Component;
use serde::{Deserialize, Serialize};

use crate::{entity::tile_map::ChunkMap, position::RectExtend};

// pub struct PhysicsPlugin;

// impl Plugin for PhysicsPlugin {
//     fn build(&self, app: &mut App) {
//         // add things to your app here
//     }
// }
#[derive(Debug, PartialEq, Clone, Copy, Serialize, Deserialize)]
pub enum CollisionResult {
    Empty,
    Solid,
    JumpThrough,
    Collider,
}

impl CollisionResult {
    pub(crate) fn or(self, other: CollisionResult) -> CollisionResult {
        use CollisionResult::*;
        match (self, other) {
            (Empty, Empty) => Empty,
            (JumpThrough, JumpThrough) => JumpThrough,
            (JumpThrough, Empty) => JumpThrough,
            (Empty, JumpThrough) => JumpThrough,
            _ => Solid,
        }
    }
}

#[derive(Component)]
pub struct Actor;

#[derive(Component)]
pub struct Solid;

#[allow(unused)]
#[derive(Component)]
pub struct Collider {
    collidable: bool,
    squished: bool,
    pub pos: Vec2,
    pub width: i32,
    pub height: i32,
    x_remainder: f32,
    y_remainder: f32,
    squishers: HashSet<Solid>,
    descent: bool,
    seen_wood: bool,
}

pub fn add_actor(pos: Vec2, width: i32, height: i32, map: &mut ChunkMap) -> (Actor, Collider) {
    let actor = Actor;

    let mut descent = false;
    let mut seen_wood = false;
    
    let tile = map.collide(Rect::new(pos.x, pos.y, width as f32, height as f32));
    
    // let tile = map.collide_solids(pos, width, height);
    if tile == CollisionResult::JumpThrough {
        descent = true;
        seen_wood = true;
    }
    (
        actor,
        Collider {
            collidable: true,
            squished: false,
            pos,
            width,
            height,
            x_remainder: 0.,
            y_remainder: 0.,
            squishers: HashSet::new(),
            descent,
            seen_wood,
        },
    )
}



pub fn move_v(map: &mut ChunkMap, collider: &mut Collider, dy: f32) -> bool {
    use CollisionResult::*;
    
    collider.y_remainder += dy;

    let mut move_ = collider.y_remainder.round() as i32;
    if move_ != 0 {
        collider.y_remainder -= move_ as f32;
        let sign = move_.signum();

        while move_ != 0 {
            let tile = map.collide(
                Rect::from_vecs(
                    collider.pos + vec2(0., sign as f32),
                    ivec2(collider.width, collider.height).as_vec2()
                )
            );

            // collider wants to go down and collided with jumpthrough tile
            if tile == JumpThrough && collider.descent {
                collider.seen_wood = true;
            }
            // collider wants to go up and encoutered jumpthrough obstace
            if tile == JumpThrough && sign < 0 {
                collider.seen_wood = true;
                collider.descent = true;
            }
            if tile == Empty || (tile == JumpThrough && collider.descent) {
                collider.pos.y += sign as f32;
                move_ -= sign;
            } else {
                return false;
            }
        }
    }

    // Final check, if we are out of woods after the move - reset wood flags
    let tile = map.collide(Rect::from_vecs(collider.pos, ivec2(collider.width, collider.height).as_vec2()));
    if tile != JumpThrough {
        collider.seen_wood = false;
        collider.descent = false;
    }
    true
}

pub fn move_h(map: &mut ChunkMap, collider: &mut Collider, dx: f32) -> bool {
    use CollisionResult::*;
    
    collider.x_remainder += dx;

    let mut move_ = collider.x_remainder.round() as i32;
    if move_ != 0 {
        collider.x_remainder -= move_ as f32;
        let sign = move_.signum();

        while move_ != 0 {
            let tile = map.collide(Rect::from_vecs(
                collider.pos + vec2(sign as f32, 0.),
                ivec2( collider.width, collider.height ).as_vec2()
            ));
            if tile == JumpThrough {
                collider.descent = true;
                collider.seen_wood = true;
            }
            if tile == Empty || tile == JumpThrough {
                collider.pos.x += sign as f32;
                move_ -= sign;
            } else {
                return false;
            }
        }
    }
    true
}