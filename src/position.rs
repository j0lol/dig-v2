use macroquad::{camera::Camera2D, input::mouse_position, math::Rect, window::{screen_height, screen_width}};
use macroquad::math::{vec2, IVec2, Vec2};
use serde::{Deserialize, Serialize};

use crate::{TILE_SIZE, VIRTUAL_HEIGHT, VIRTUAL_WIDTH};

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChunkPos(pub IVec2);
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct TilePos(pub Vec2);
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct WorldPos(pub Vec2);
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct ScreenPos(pub Vec2);

impl ScreenPos {
    pub fn screen() -> ScreenPos {
        ScreenPos(vec2(VIRTUAL_WIDTH, VIRTUAL_HEIGHT))
    }
    
    pub fn screen_rect() -> Rect {
        Rect::new(0., 0., VIRTUAL_WIDTH, VIRTUAL_HEIGHT)
    }
    
    pub fn mouse() -> Option<ScreenPos> {
        let screen = Rect::new(0., 0., VIRTUAL_WIDTH, VIRTUAL_HEIGHT);
    
        let mouse_pos= mouse_position();
        let mouse_pos = vec2(mouse_pos.0, mouse_pos.1);
        let mouse_pos = real_to_virtual_screen_space(mouse_pos);
    
        screen.contains(mouse_pos).then_some(ScreenPos(mouse_pos))
    }
    pub fn to_world(self, camera: &Camera2D) -> WorldPos {
        WorldPos(
            camera.target + self.0 - Rect::new(0., 0., VIRTUAL_WIDTH, VIRTUAL_HEIGHT).size()/2.
        )
    }
    pub fn to_tile(self, camera: &Camera2D) -> TilePos {
        self.to_world(camera).to_tile()
    }
}
impl WorldPos {
    pub fn snap(self) -> WorldPos {
        TilePos(self.to_tile().0.floor()).to_world()
    }
    pub fn to_tile(self) -> TilePos {
        TilePos(self.0 / TILE_SIZE)
    }
    pub fn to_chunk(self) -> ChunkPos {
        ChunkPos((self.0 / ScreenPos::screen().0).floor().as_ivec2())
    }
}

impl TilePos {
    pub fn to_world(self) -> WorldPos {
        WorldPos(self.0 * TILE_SIZE)
    }
}

fn real_to_virtual_screen_space(pos: Vec2) -> Vec2 {
    let real_screen_size = vec2(screen_width(), screen_height());
    let virtual_screen_size = vec2(VIRTUAL_WIDTH, VIRTUAL_HEIGHT);
    let scale = (real_screen_size / virtual_screen_size)
        .min_element()
        .floor()
        .max(1.0);

    let margin = (real_screen_size - ( virtual_screen_size * scale )) / 2.0;
    return (pos - margin) / scale;
}

pub trait VecExtend {
    fn rect_from_origin(self: Self) -> Rect;
}
impl VecExtend for Vec2 {
    fn rect_from_origin(self) -> Rect {
        Rect::new(0., 0., self.x, self.y)
    }
}

pub trait RectExtend {
    fn from_vecs(top_left: Vec2, size: Vec2) -> Rect;
}

impl RectExtend for Rect {
    fn from_vecs(top_left: Vec2, size: Vec2) -> Rect {
        Rect::new(top_left.x, top_left.y, size.x, size.y)
    }
}