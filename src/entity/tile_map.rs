use std::collections::HashMap;
use base64::prelude::BASE64_STANDARD;
use base64::Engine as _;
use bevy_ecs::prelude::*;
use itertools::Itertools;
use macroquad::prelude::*;
use serde::{Deserialize, Serialize};
use crate::grid::Grid;
use crate::entity::player::Player;
use crate::physics2::CollisionResult;
use crate::position::{ChunkPos, RectExtend, ScreenPos, WorldPos};
use crate::tile::TileId;
use crate::entity::ui::draw_from_tile_set;
use crate::{SAVE_TIMER, TILE_SIZE};

pub(super) fn init_map(mut commands: Commands) {
    commands.insert_resource(SaveTimer(SAVE_TIMER))
}

pub(super) fn timed_save(mut timer: ResMut<SaveTimer>, mut map: Query<&mut ChunkMap>) {
    let map = map.get_single_mut().unwrap();
    
    if timer.0 < 0.0 {
        map.save();
        timer.0 = SAVE_TIMER;
    } else {
        timer.0 -= get_frame_time()
    }
}

pub(super) fn draw_map(mut map: Query<&mut ChunkMap>) {
    let mut map = map.single_mut();
    let map = map.as_mut();
    
    for pos in [-1, -1, 0, 0, 1, 1].into_iter().permutations(2).unique() {
        // let mut chunk_map = chunk_map.clone();
        let pos = ivec2(pos[0] + map.focus.0.x, pos[1] + map.focus.0.y);
        let chunk = map.get(ChunkPos(pos));

        chunk.draw(ScreenPos::screen_rect().size() * pos.as_vec2());
    }
}


#[derive(Resource)]
pub struct SaveTimer(pub f32);

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Chunk(pub Grid<TileId>);

static SAVE_KEY: &str = "ChunkMap";

impl Chunk {
    pub fn draw(&self, offset: Vec2) {
        self.0.for_each_immut(|point, tile| {
            if let Some(tile_index) = tile.val().sprite {
                draw_from_tile_set(tile_index, point.as_vec2() * 16. + offset);
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

#[derive(Component, Clone, Debug, Serialize, Deserialize)]
pub struct ChunkMap {
    store: HashMap<ChunkPos, Chunk>,
    pub focus: ChunkPos,
    pub tile_size: UVec2,
    pub chunk_size: UVec2,
    pub tag: u8,
}

impl ChunkMap {
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

    pub fn place_tile(&mut self, rect: Rect, pos: WorldPos, tile: TileId) {
        
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
    
    pub fn save(&self) {
        println!("Save");
        let data = bincode::serialize(&self).expect("Serde Bincode failure");
        let data = BASE64_STANDARD.encode(data);
        let storage = &mut quad_storage::STORAGE.lock().expect("Storage lock fail");
        storage.set(SAVE_KEY, &data);
    }
    
    pub fn load() -> Option<ChunkMap> {
        println!("Load");
        let storage = &mut quad_storage::STORAGE.lock().expect("Storage lock fail");
        
        let data = BASE64_STANDARD.decode(storage.get(SAVE_KEY)?).ok()?;
        let world: ChunkMap = bincode::deserialize(&data[..]).ok()?;
        
        Some(world)
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

