/*
MIT License

@ 2019-2021 Fedor Logachev <not.fl3@gmail.com>

Permission is hereby granted, free of charge, to any person obtaining a
copy of this software and associated documentation files (the "Software"),
to deal in the Software without restriction, including without limitation
the rights to use, copy, modify, merge, publish, distribute, sublicense,
and/or sell copies of the Software, and to permit persons to whom the
Software is furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in
all copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL
THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING
FROM, OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER
DEALINGS IN THE SOFTWARE.
*/

use base64::{prelude::BASE64_STANDARD, Engine};
use macroquad::math::{vec2, Vec2};
use macroquad::math::Rect;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use crate::entity::tile_map::ChunkMap;



#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash, Serialize, Deserialize)]
pub struct Actor(usize);

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash, Serialize, Deserialize)]
pub struct Solid(usize);



#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Collider {
    collidable: bool,
    squished: bool,
    pos: Vec2,
    width: i32,
    height: i32,
    x_remainder: f32,
    y_remainder: f32,
    squishers: HashSet<Solid>,
    descent: bool,
    seen_wood: bool,
}

impl Collider {
    pub fn rect(&self) -> Rect {
        Rect::new(
            self.pos.x,
            self.pos.y,
            self.width as f32,
            self.height as f32,
        )
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct World {
    pub map: ChunkMap,
    pub solids: Vec<(Solid, Collider)>,
    pub actors: Vec<(Actor, Collider)>,
}

impl World {
    pub fn new() -> World {
        World {
            map: ChunkMap::default(),
            actors: vec![],
            solids: vec![],
        }
    }
    
    pub fn save(&self) {
        println!("Save");
        let data = bincode::serialize(&self).expect("Serde Bincode failure");
        let data = BASE64_STANDARD.encode(data);
        let storage = &mut quad_storage::STORAGE.lock().expect("Storage lock fail");
        storage.set("World", &data);
        storage.set("foo", &BASE64_STANDARD.encode(b"bar"));
    }
    
    pub fn load() -> Option<World> {
        println!("Load");
        let storage = &mut quad_storage::STORAGE.lock().expect("Storage lock fail");
       
        // if storage.get("foo").is_none() {
        //     return None;
        // }
        
        // assert_eq!("bar", String::from_utf8(BASE64_STANDARD.decode(storage.get("foo").unwrap()).unwrap()).unwrap());
        
        let data = storage.get("World")?;
        let data = BASE64_STANDARD.decode(data).expect("Base64 Decode failure");
        let data = &data[..];
        let world: World = bincode::deserialize(data).unwrap();
        
        Some(world)
    }
    

    // pub fn add_static_tiled_layer(
    //     &mut self,
    //     static_colliders: Grid<Tile>,
    //     tile_width: f32,
    //     tile_height: f32,
    //     width: usize,
    //     tag: u8,
    // ) {
    //     self.map.push(StaticTiledLayer {
    //         static_colliders,
    //         tile_width,
    //         tile_height,
    //         width,
    //         tag,
    //     });
    // }

    pub fn add_actor(&mut self, pos: Vec2, width: i32, height: i32) -> Actor {
        let actor = Actor(self.actors.len());

        let mut descent = false;
        let mut seen_wood = false;
        let tile = self.collide_solids(pos, width, height);
        if tile == CollisionResult::JumpThrough {
            descent = true;
            seen_wood = true;
        }
        self.actors.push((
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
        ));

        actor
    }

    pub fn add_solid(&mut self, pos: Vec2, width: i32, height: i32) -> Solid {
        let solid = Solid(self.solids.len());

        self.solids.push((
            solid,
            Collider {
                collidable: true,
                squished: false,
                pos,
                width,
                height,
                x_remainder: 0.,
                y_remainder: 0.,
                squishers: HashSet::new(),
                descent: false,
                seen_wood: false,
            },
        ));

        solid
    }

    pub fn set_actor_position(&mut self, actor: Actor, pos: Vec2) {
        let collider = &mut self.actors[actor.0].1;

        collider.x_remainder = 0.0;
        collider.y_remainder = 0.0;
        collider.pos = pos;
    }

    pub fn descent(&mut self, actor: Actor) {
        let collider = &mut self.actors[actor.0].1;
        collider.descent = true;
    }

    pub fn move_v(&mut self, actor: Actor, dy: f32) -> bool {
        use CollisionResult::*;
        
        let id = actor.0;
        let mut collider = self.actors[id].1.clone();

        collider.y_remainder += dy;

        let mut move_ = collider.y_remainder.round() as i32;
        if move_ != 0 {
            collider.y_remainder -= move_ as f32;
            let sign = move_.signum();

            while move_ != 0 {
                let tile = self.collide_solids(
                    collider.pos + vec2(0., sign as f32),
                    collider.width,
                    collider.height,
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
                    self.actors[id].1 = collider;

                    return false;
                }
            }
        }

        // Final check, if we are out of woods after the move - reset wood flags
        let tile = self.collide_solids(collider.pos, collider.width, collider.height);
        if tile != JumpThrough {
            collider.seen_wood = false;
            collider.descent = false;
        }

        self.actors[id].1 = collider;
        true
    }

    pub fn move_h(&mut self, actor: Actor, dx: f32) -> bool {
        use CollisionResult::*;
        let id = actor.0;
        let mut collider = self.actors[id].1.clone();
        collider.x_remainder += dx;

        let mut move_ = collider.x_remainder.round() as i32;
        if move_ != 0 {
            collider.x_remainder -= move_ as f32;
            let sign = move_.signum();

            while move_ != 0 {
                let tile = self.collide_solids(
                    collider.pos + vec2(sign as f32, 0.),
                    collider.width,
                    collider.height,
                );
                if tile == JumpThrough {
                    collider.descent = true;
                    collider.seen_wood = true;
                }
                if tile == Empty || tile == JumpThrough {
                    collider.pos.x += sign as f32;
                    move_ -= sign;
                } else {
                    self.actors[id].1 = collider;
                    return false;
                }
            }
        }
        self.actors[id].1 = collider;
        true
    }

    pub fn solid_move(&mut self, solid: Solid, dx: f32, dy: f32) {
        let collider = &mut self.solids[solid.0].1;

        collider.x_remainder += dx;
        collider.y_remainder += dy;
        let move_x = collider.x_remainder.round() as i32;
        let move_y = collider.y_remainder.round() as i32;

        let mut riding_actors = vec![];
        let mut pushing_actors = vec![];

        let riding_rect = Rect::new(
            collider.pos.x,
            collider.pos.y - 1.0,
            collider.width as f32,
            1.0,
        );
        let pushing_rect = Rect::new(
            collider.pos.x + move_x as f32,
            collider.pos.y,
            collider.width as f32,
            collider.height as f32,
        );

        for (actor, actor_collider) in &mut self.actors {
            let rider_rect = Rect::new(
                actor_collider.pos.x,
                actor_collider.pos.y + actor_collider.height as f32 - 1.0,
                actor_collider.width as f32,
                1.0,
            );

            if riding_rect.overlaps(&rider_rect) {
                riding_actors.push(*actor);
            } else if pushing_rect.overlaps(&actor_collider.rect())
                && !actor_collider.squished
            {
                pushing_actors.push(*actor);
            }

            if !pushing_rect.overlaps(&actor_collider.rect()) {
                actor_collider.squishers.remove(&solid);
                if actor_collider.squishers.is_empty() {
                    actor_collider.squished = false;
                }
            }
        }

        self.solids[solid.0].1.collidable = false;
        for actor in riding_actors {
            self.move_h(actor, move_x as f32);
        }
        for actor in pushing_actors {
            let squished = !self.move_h(actor, move_x as f32);
            if squished {
                self.actors[actor.0].1.squished = true;
                self.actors[actor.0].1.squishers.insert(solid);
            }
        }
        self.solids[solid.0].1.collidable = true;

        let collider = &mut self.solids[solid.0].1;
        if move_x != 0 {
            collider.x_remainder -= move_x as f32;
            collider.pos.x += move_x as f32;
        }
        if move_y != 0 {
            collider.y_remainder -= move_y as f32;
            collider.pos.y += move_y as f32;
        }
    }

    pub fn solid_at(&self, pos: Vec2) -> bool {
        self.tag_at(pos, 1)
    }

    pub fn tag_at(&self, _pos: Vec2, _tag: u8) -> bool {
        true
        // FIXME
        // for StaticTiledLayer {
        //     tile_width,
        //     tile_height,
        //     width,
        //     static_colliders,
        //     tag: layer_tag,
        // } in &self.static_tiled_layers
        // {
        //     let y = (pos.y / tile_width) as i32;
        //     let x = (pos.x / tile_height) as i32;
        //     let ix = y * (width as i32) + x;
        // 
        //     if ix >= 0
        //         && ix < static_colliders.array.len() as i32
        //         && static_colliders.array[ix as usize] != Tile::Empty
        //     {
        //         return *layer_tag == tag;
        //     }
        // }
        // 
        // self.solids
        //     .iter()
        //     .any(|solid| solid.1.collidable && solid.1.rect().contains(pos))
    }

    pub fn collide_solids(&mut self, pos: Vec2, width: i32, height: i32) -> CollisionResult {
        let tile = self.collide_tag(1, pos, width, height);
        if tile != CollisionResult::Empty {
            return tile;
        }

        self.solids
            .iter()
            .find(|solid| {
                solid.1.collidable
                    && solid.1.rect().overlaps(&Rect::new(
                    pos.x,
                    pos.y,
                    width as f32,
                    height as f32,
                ))
            })
            .map_or(CollisionResult::Empty, |_| CollisionResult::Collider)
    }

    pub fn collide_tag(&mut self, _tag: u8, pos: Vec2, width: i32, height: i32) -> CollisionResult {
        self.map.collide(Rect::new(pos.x, pos.y, width as f32, height as f32))
    }

    pub fn squished(&self, actor: Actor) -> bool {
        self.actors[actor.0].1.squished
    }

    pub fn actor_pos(&self, actor: Actor) -> Vec2 {
        self.actors[actor.0].1.pos
    }

    pub fn solid_pos(&self, solid: Solid) -> Vec2 {
        self.solids[solid.0].1.pos
    }

    pub fn collide_check(&mut self, collider: Actor, pos: Vec2) -> bool {
        let (_, collider) = self.actors[collider.0].clone();

        let tile = self.collide_solids(pos, collider.width, collider.height);

        use CollisionResult::*;
        if collider.descent {
            tile == Solid || tile == Collider
        } else {
            tile == Solid || tile == Collider || tile == JumpThrough
        }
    }
}

mod test {
    
    use super::*;
    
    #[test]
    fn store_world() -> Result<(), Box<dyn std::error::Error>> {
        let world = World::new();
        
        let data = bincode::serialize(&world)?;
        let data = BASE64_STANDARD.encode(data);
        let data = BASE64_STANDARD.decode(data)?;
        let _: World = bincode::deserialize(&data[..])?;
        
        Ok(())
    }
}