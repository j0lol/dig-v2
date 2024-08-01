use std::collections::HashMap;
use std::fmt::format;
use itertools::Itertools;
use macroquad::prelude::*;
use crate::grid::Grid;
use crate::physics::{Tile, World};
use crate::{TILE_SIZE, VIRTUAL_HEIGHT, VIRTUAL_WIDTH};
use crate::player::Player;

#[derive(Clone, Debug)]
pub struct Chunk(pub Grid<Tile>);

impl Chunk {
    // pub fn physics_register(&self, world: &mut World) {
    //     world.static_tiled_layers.pop();
    //     world.add_static_tiled_layer(self.0.clone(), TILE_SIZE, TILE_SIZE, (VIRTUAL_WIDTH / TILE_SIZE) as usize, 1);
    // 
    // }
    pub fn draw(&self, tile_set: &Texture2D, offset: Vec2) {
        self.0.for_each_immut(|point, tile| {
            if let Tile::Solid = tile {
                draw_texture_ex(
                    tile_set,
                    point.x as f32 * 16. + offset.x,
                    point.y as f32 * 16. + offset.y,
                    WHITE,
                    DrawTextureParams {
                        dest_size: None,
                        source: Some(
                            Rect::new(
                                1. * TILE_SIZE,
                                1. * TILE_SIZE,
                                TILE_SIZE,
                                TILE_SIZE
                            )),
                        rotation: 0.0,
                        flip_x: false,
                        flip_y: false,
                        pivot: None,
                    });
            }
        });
    }
    pub fn dbg_draw(&self, offset: Vec2) {

        self.0.for_each_immut(|point, tile| {
            let color = if tile == Tile::Solid { PURPLE } else { SKYBLUE } ;
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

#[derive(Clone, Debug)]
pub struct ChunkMap {
    store: HashMap<ChunkIndex, Chunk>,
    pub focus: ChunkIndex,
    pub tile_size: UVec2,
    pub chunk_size: UVec2,
    pub tag: u8,
}

impl ChunkMap {
    pub(crate) fn update(&mut self, rect: Rect, mouse_pos: Vec2) {
        if is_mouse_button_down(MouseButton::Left) ^ is_mouse_button_down(MouseButton::Right) {
            let tile = if is_mouse_button_down(MouseButton::Left) { Tile::Empty } else { Tile::Solid };
            self.place_tile(rect, mouse_pos, tile);
        }
    }
    

    pub(crate) fn draw(&mut self, tile_set: &Texture2D, virtual_screen: Rect) {
        for pos in [-1, -1, 0, 0, 1, 1].into_iter().permutations(2).unique() {
            // let mut chunk_map = chunk_map.clone();
            let pos = ivec2(pos[0] + self.focus.x, pos[1] + self.focus.y);
            let chunk = self.get(pos);

            chunk.draw(tile_set, virtual_screen.size() * pos.as_vec2());
        }
    }
    
    pub fn get(&mut self, chunk_index: ChunkIndex) -> &Chunk {
       self.get_mut(chunk_index)
    }
    
    pub fn get_mut(&mut self, chunk_index: ChunkIndex) -> &mut Chunk {
        self.store.entry(chunk_index).or_insert_with(|| {
            match chunk_index.y {
                0 => {
                    Chunk(Grid::new_filled(
                        (VIRTUAL_WIDTH / TILE_SIZE) as usize,
                        (VIRTUAL_HEIGHT / TILE_SIZE) as usize,
                        |point| if point.y > 7 { Tile::Solid } else { Tile::Empty },
                        Tile::Empty
                    ))
                }
                1..=i32::MAX => {
                    Chunk(Grid::new_filled(
                        (VIRTUAL_WIDTH / TILE_SIZE) as usize,
                        (VIRTUAL_HEIGHT / TILE_SIZE) as usize,
                        |_| Tile::Solid,
                        Tile::Empty
                    ))
                }
                i32::MIN..=-1 => {
                    Chunk(Grid::new_filled(
                        (VIRTUAL_WIDTH / TILE_SIZE) as usize,
                        (VIRTUAL_HEIGHT / TILE_SIZE) as usize,
                        |_| Tile::Empty,
                        Tile::Empty
                    ))
                }
            }
        })
    }

    fn place_tile(&mut self, rect: Rect, world_coord: Vec2, tile: Tile) {
        
        let tile_rect = {
            let snapped = (world_coord / 16.).floor() * 16.;
            
            Rect::new(snapped.x, snapped.y, 16., 16.)
        };
        
        if rect.intersect(tile_rect).is_some_and(|rect| rect.size().min_element() != 0.0) {
            draw_rectangle(tile_rect.x, tile_rect.y, tile_rect.w, tile_rect.h, RED);
            return;
        }
        
        let chunk_inside = global_coordinate_to_chunk(world_coord);
        let screen_size = (self.chunk_size * self.tile_size).as_vec2();

        let position_in_chunk = wrap_around_vec_in_rect(
            Rect::new(0., 0., screen_size.x, screen_size.y),
            world_coord
        );
        let chunk_inside = self.get_mut(chunk_inside);

        let pos = position_in_chunk.floor().as_uvec2() / 16;

        chunk_inside.0[pos] = tile;
    }

}

type ChunkIndex = IVec2;

impl ChunkMap {
    pub fn new() -> ChunkMap {
        let chunks = HashMap::<ChunkIndex, Chunk>::new();
        // chunks.insert(ivec2(0, 0), Chunk::default());

        ChunkMap {
            store: chunks, 
            focus: ivec2(0,0),
            tile_size: UVec2::splat(TILE_SIZE as u32),
            chunk_size: uvec2((VIRTUAL_WIDTH / TILE_SIZE) as u32, (VIRTUAL_HEIGHT / TILE_SIZE) as u32),
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
    
    pub fn chunks_around(&mut self, point: ChunkIndex) -> NineOf<Chunk> {
        let chunks =  [-1, -1, 0, 0, 1, 1].into_iter().permutations(2).unique().map(|pos| {
            let pos = ivec2(pos[0] + point.x, pos[1] + point.y);
            let chunk = self.get(pos);

            ((pos.x as i8, pos.y as i8), chunk.to_owned())
        }).collect_vec();
        
        NineOf {
            items: chunks
        }
    }
    
    pub fn collide(&mut self, collider: Rect) -> Tile {
        let tile_size = self.tile_size.as_vec2();


        let mut check = |pos: Vec2| {
            let chunk_inside = global_coordinate_to_chunk(pos);
            let screen_size = (self.chunk_size * self.tile_size).as_vec2();

            let position_in_chunk = wrap_around_vec_in_rect(
                Rect::new(0., 0., screen_size.x, screen_size.y),
                pos
            );
            let chunk_inside = self.get(chunk_inside);

            let tile = chunk_inside.0.get(
                position_in_chunk.floor().as_uvec2() / 16
            ).unwrap().to_owned();

            tile
        };

        let tile = check(collider.point())
            .or(check(collider.point() + vec2(collider.w - 1.0, 0.0)))
            .or(check(collider.point() + vec2(collider.w - 1.0, collider.h - 1.0)))
            .or(check(collider.point() + vec2(0.0, collider.h - 1.0)));

        if tile != Tile::Empty {
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
                if tile != Tile::Empty {
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
                if tile != Tile::Empty {
                    return tile;
                }
            }
        }

        Tile::Empty
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


fn clamp_vec_in_rect(rect: Rect, vec: Vec2) -> Vec2 {
    let mut ret_vec = vec;
    if vec.x < rect.left() {
        ret_vec.x = rect.left();
    }
    if vec.x >= rect.right() {
        ret_vec.x = rect.right();
    }
    if vec.y < rect.top() {
        ret_vec.y = rect.top();
    }
    if vec.y >= rect.bottom() {
        ret_vec.y = rect.bottom();
    }
    ret_vec
}

pub fn global_coordinate_to_chunk(position: Vec2) -> ChunkIndex {
    (position / vec2(VIRTUAL_WIDTH, VIRTUAL_HEIGHT)).floor().as_ivec2()
}