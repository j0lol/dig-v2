use std::collections::HashMap;
use itertools::Itertools;
use macroquad::prelude::*;
use serde::{Deserialize, Serialize};
use crate::grid::Grid;
use crate::player::Player;
use crate::position::{ChunkPos, RectExtend, ScreenPos, WorldPos};
use crate::tile::TileId;
use crate::ui::draw_from_tile_set;
use crate::{CollisionResult, TILE_SET, TILE_SIZE};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Chunk(pub Grid<TileId>);

impl Chunk {
    pub fn draw(&self, offset: Vec2) {
        // let tile_set = TILE_SET.get().unwrap();
        self.0.for_each_immut(|point, tile| {
            if let Some(tile_index) = tile.val().sprite {
                draw_from_tile_set(tile_index, point.as_vec2() * 16. + offset);
                
                
                // let tileset_width = tile_set.width() / TILE_SIZE;
                
                // let sprite_rect =  Rect::from_vecs(
                //     vec2(
                //         (tile_index as f32 % tileset_width).floor() * TILE_SIZE,
                //         (tile_index as f32 / tileset_width).floor() * TILE_SIZE,
                //     ), Vec2::splat(TILE_SIZE));
                
                
                // draw_texture_ex(
                //     tile_set,
                //     point.x as f32 * 16. + offset.x,
                //     point.y as f32 * 16. + offset.y,
                //     WHITE,
                //     DrawTextureParams {
                //         source: Some(sprite_rect),
                //         ..Default::default()
                //     }
                // );
            }
        });
    }
    pub fn dbg_draw(&self, offset: Vec2) {

        self.0.for_each_immut(|point, tile| {
            let tile = tile.val().collision_result();
            
            let color = if tile == CollisionResult::Solid { PURPLE } else { SKYBLUE } ;
            draw_rectangle(
                (point.x as f32 * 2.) + offset.x,
                (point.y as f32 * 2.) + offset.y,
                2.,
                2.,
                color
            )
        });
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ChunkMap {
    store: HashMap<ChunkPos, Chunk>,
    pub focus: ChunkPos,
    pub tile_size: UVec2,
    pub chunk_size: UVec2,
    pub tag: u8,
}

impl ChunkMap {
    pub(crate) fn update(&mut self, rect: Rect, world_mouse_pos: Option<WorldPos>, player: &Player) {
        use MouseButton::*;
        use TileId::*;
        if is_mouse_button_down(Left) ^ is_mouse_button_down(Right) {
            let tile = if is_mouse_button_down(Left) {
                Air
            } else {
                player.get_inventory_item()
            };
            if let Some(world_mouse_pos) = world_mouse_pos {
                self.place_tile(rect, world_mouse_pos, tile);
            }
        }
    }
    

    pub(crate) fn draw(&mut self, virtual_screen: Rect) {
        for pos in [-1, -1, 0, 0, 1, 1].into_iter().permutations(2).unique() {
            // let mut chunk_map = chunk_map.clone();
            let pos = ivec2(pos[0] + self.focus.0.x, pos[1] + self.focus.0.y);
            let chunk = self.get(ChunkPos(pos));

            chunk.draw(virtual_screen.size() * pos.as_vec2());
        }
    }
    
    pub fn get(&mut self, chunk_index: ChunkPos) -> &Chunk {
       self.get_mut(chunk_index)
    }
    
    pub fn get_mut(&mut self, chunk_index: ChunkPos) -> &mut Chunk {
        let chunk_size = ScreenPos::screen().0 / TILE_SIZE;
        self.store.entry(chunk_index).or_insert_with(|| {
            match chunk_index.0.y {
                0 => {
                    Chunk(Grid::new_filled(
                        chunk_size.x as usize,
                        chunk_size.y as usize,
                        |point| if point.y > 7 { TileId::Dirt } else { TileId::Air },
                        TileId::default()
                    ))
                }
                1..=i32::MAX => {
                    Chunk(Grid::new_filled(
                        chunk_size.x as usize,
                        chunk_size.y as usize,
                        |_| TileId::Dirt,
                        TileId::default()
                    ))
                }
                i32::MIN..=-1 => {
                    Chunk(Grid::new_filled(
                        chunk_size.x as usize,
                        chunk_size.y as usize,
                        |_| TileId::Air,
                        TileId::default()
                    ))
                }
            }
        })
    }

    fn place_tile(&mut self, rect: Rect, pos: WorldPos, tile: TileId) {
        
        let tile_rect = Rect::from_vecs(pos.snap().0, Vec2::splat(16.0));
        
        if rect.intersect(tile_rect).is_some_and(|rect| rect.size().min_element() != 0.0) {
            draw_rectangle(tile_rect.x, tile_rect.y, tile_rect.w, tile_rect.h, RED);
            return;
        }
        
        let chunk_inside = pos.to_chunk();
        let screen_size = (self.chunk_size * self.tile_size).as_vec2();

        let position_in_chunk = wrap_around_vec_in_rect(
            Rect::new(0., 0., screen_size.x, screen_size.y),
            pos.0
        );
        let chunk_inside = self.get_mut(chunk_inside);

        let pos = WorldPos(position_in_chunk).to_tile().0.as_uvec2();

        chunk_inside.0[pos] = tile;
    }

}


impl ChunkMap {
    pub fn new() -> ChunkMap {
        let chunk_size = (ScreenPos::screen().0 / TILE_SIZE).as_uvec2();
        let chunks = HashMap::<ChunkPos, Chunk>::new();

        ChunkMap {
            store: chunks, 
            focus: ChunkPos(ivec2(0,0)),
            tile_size: UVec2::splat(TILE_SIZE as u32),
            chunk_size,
            tag: 0
        }
    }
    
    pub fn focused(&mut self) -> &Chunk {
        self.get(self.focus)
    }
    pub fn focused_mut(&mut self) -> &mut Chunk {
        self.get_mut(self.focus)
    }
    pub fn around_focus(&mut self) -> NineOf<Chunk> {
        self.chunks_around(self.focus)
    }
    
    pub fn chunks_around(&mut self, point: ChunkPos) -> NineOf<Chunk> {
        let chunks =  [-1, -1, 0, 0, 1, 1].into_iter().permutations(2).unique().map(|pos| {
            let pos = ivec2(pos[0] + point.0.x, pos[1] + point.0.y);
            let chunk = self.get(ChunkPos(pos));

            ((pos.x as i8, pos.y as i8), chunk.to_owned())
        }).collect_vec();
        
        NineOf {
            items: chunks
        }
    }
    
    pub fn collide(&mut self, collider: Rect) -> CollisionResult {
        let tile_size = self.tile_size.as_vec2();


        let mut check = |pos: Vec2| {
            let chunk_inside = WorldPos(pos).to_chunk();
            let screen_size = (self.chunk_size * self.tile_size).as_vec2();

            let position_in_chunk = wrap_around_vec_in_rect(
                Rect::new(0., 0., screen_size.x, screen_size.y),
                pos
            );
            let chunk_inside = self.get(chunk_inside);

            let tile = chunk_inside.0.get(
                position_in_chunk.floor().as_uvec2() / 16
            ).unwrap().to_owned().val().collision_result();

            tile
        };

        let tile = check(collider.point())
            .or(check(collider.point() + vec2(collider.w - 1.0, 0.0)))
            .or(check(collider.point() + vec2(collider.w - 1.0, collider.h - 1.0)))
            .or(check(collider.point() + vec2(0.0, collider.h - 1.0)));

        if tile != CollisionResult::Empty {
            return tile;
        }
        

        if collider.w > tile_size.x {
            let mut x = collider.x;
        
            while {
                x += tile_size.x;
                x < collider.x + collider.w - 1.
            } {
                let tile =
                    check(vec2(x, collider.y)).or(check(vec2(x, collider.y + collider.h - 1.0)));
                if tile != CollisionResult::Empty {
                    return tile;
                }
            }
        }
        
        if collider.h > tile_size.y {
            let mut y = collider.y;
        
            while {
                y += tile_size.y;
                y < collider.y + collider.h - 1.
            } {
                let tile = check(vec2(collider.x, y)).or(check(vec2(collider.x + collider.w - 1., y)));
                if tile != CollisionResult::Empty {
                    return tile;
                }
            }
        }

        CollisionResult::Empty
    }
}

impl Default for ChunkMap {
    fn default() -> Self {
        ChunkMap::new()
    }
}

pub struct NineOf<T> {
    pub items: Vec<((i8, i8), T)>,
}

pub fn wrap_around_vec_in_rect(rect: Rect, vec: Vec2) -> Vec2 {
    vec.rem_euclid(rect.size())
}

