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

use macroquad::math::{vec2, Rect, Vec2};

use std::collections::HashSet;
use macroquad::prelude::{ivec2, uvec2};
use crate::grid::Grid;
use crate::{VIRTUAL_HEIGHT, VIRTUAL_WIDTH};
use crate::tile_map::ChunkMap;

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Tile {
    Empty,
    Solid,
    JumpThrough,
    Collider,
}

impl Tile {
    pub(crate) fn or(self, other: Tile) -> Tile {
        match (self, other) {
            (Tile::Empty, Tile::Empty) => Tile::Empty,
            (Tile::JumpThrough, Tile::JumpThrough) => Tile::JumpThrough,
            (Tile::JumpThrough, Tile::Empty) => Tile::JumpThrough,
            (Tile::Empty, Tile::JumpThrough) => Tile::JumpThrough,
            _ => Tile::Solid,
        }
    }
}
#[derive(Debug, Clone)]
pub struct StaticTiledLayer {
    pub static_colliders: Grid<Tile>,
    tile_width: f32,
    tile_height: f32,
    pub(crate) width: usize,
    tag: u8,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub struct Actor(usize);

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub struct Solid(usize);



#[derive(Clone, Debug)]
struct Collider {
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

#[derive(Debug, Clone)]
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
        if tile == Tile::JumpThrough {
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
                if tile == Tile::JumpThrough && collider.descent {
                    collider.seen_wood = true;
                }
                // collider wants to go up and encoutered jumpthrough obstace
                if tile == Tile::JumpThrough && sign < 0 {
                    collider.seen_wood = true;
                    collider.descent = true;
                }
                if tile == Tile::Empty || (tile == Tile::JumpThrough && collider.descent) {
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
        if tile != Tile::JumpThrough {
            collider.seen_wood = false;
            collider.descent = false;
        }

        self.actors[id].1 = collider;
        true
    }

    pub fn move_h(&mut self, actor: Actor, dx: f32) -> bool {
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
                if tile == Tile::JumpThrough {
                    collider.descent = true;
                    collider.seen_wood = true;
                }
                if tile == Tile::Empty || tile == Tile::JumpThrough {
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

    pub fn tag_at(&self, pos: Vec2, tag: u8) -> bool {
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

    pub fn collide_solids(&mut self, pos: Vec2, width: i32, height: i32) -> Tile {
        let tile = self.collide_tag(1, pos, width, height);
        if tile != Tile::Empty {
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
            .map_or(Tile::Empty, |_| Tile::Collider)
    }

    pub fn collide_tag(&mut self, tag: u8, pos: Vec2, width: i32, height: i32) -> Tile {
        self.map.collide(Rect::new(pos.x, pos.y, width as f32, height as f32))
        // {
        //     let map = &self.map;
        //     let layer_width = map.chunk_size.x;
        //     let layer_height = map.chunk_size.y;
        //     let check = |pos: Vec2| {
        //         let y = (pos.y / map.tile_size.x as f32) as i32;
        //         let x = (pos.x / map.tile_size.y as f32) as i32;
        //         let ix = y * (layer_width as i32) + x;
        //         if y >= 0
        //             && y < layer_height as i32
        //             && x >= 0
        //             && x < layer_width as i32
        //             && ix >= 0
        //             && ix < (map.chunk_size.x * map.chunk_size.y) as i32
        //             && map.tag == tag
        //             && map.focused().0[uvec2(x as u32, y as u32)] != Tile::Empty
        //         {
        //             return map.focused().0[uvec2(x as u32, y as u32)];
        //         }
        //         Tile::Empty
        //     };
        // 
        //     let tile = check(pos)
        //         .or(check(pos + vec2(width as f32 - 1.0, 0.0)))
        //         .or(check(pos + vec2(width as f32 - 1.0, height as f32 - 1.0)))
        //         .or(check(pos + vec2(0.0, height as f32 - 1.0)));
        // 
        //     if tile != Tile::Empty {
        //         return tile;
        //     }
        // 
        //     if width > map.tile_size.x as i32 {
        //         let mut x = pos.x;
        // 
        //         while {
        //             x += map.tile_size.x as f32;
        //             x < pos.x + width as f32 - 1.
        //         } {
        //             let tile =
        //                 check(vec2(x, pos.y)).or(check(vec2(x, pos.y + height as f32 - 1.0)));
        //             if tile != Tile::Empty {
        //                 return tile;
        //             }
        //         }
        //     }
        // 
        //     if height > map.tile_size.y as i32 {
        //         let mut y = pos.y;
        // 
        //         while {
        //             y += map.tile_size.y as f32;
        //             y < pos.y + height as f32 - 1.
        //         } {
        //             let tile = check(vec2(pos.x, y)).or(check(vec2(pos.x + width as f32 - 1., y)));
        //             if tile != Tile::Empty {
        //                 return tile;
        //             }
        //         }
        //     }
        // }
        // Tile::Empty
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

        if collider.descent {
            tile == Tile::Solid || tile == Tile::Collider
        } else {
            tile == Tile::Solid || tile == Tile::Collider || tile == Tile::JumpThrough
        }
    }
}