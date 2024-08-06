
use bevy_ecs::prelude::*;
use macroquad::prelude::*;

use crate::{physics2::Collider, position::{RectExtend as _, ScreenPos}, tile::TileId, TILE_SET, TILE_SIZE};

use super::{camera::GameCamera, player::{Player, PlayerTag}, tile_map::ChunkMap, ui::{COLOR_BORDER, COLOR_HIGHLIGHT}};


// struct CursorPlugin;

// impl Plugin for CursorPlugin {
//     fn build(&self, app: &mut crate::app::App) {
//         app
//         ;
//     }
// }

#[derive(Component)]
pub struct Cursor {
    position: Option<ScreenPos>
}
impl Cursor {
    pub fn on_screen(&self) -> bool {
        self.position.is_some()
    }
}

pub(super) fn init_cursor(mut commands: Commands) {
    let pos = ScreenPos::mouse();
    
    commands.spawn(Cursor {
        position: pos
    });
}

pub(super) fn update_cursor(mut cursor: Query<&mut Cursor>, camera: Query<&GameCamera>, player: Query<(&Player, &Collider), With<PlayerTag>>, mut map: Query<&mut ChunkMap>) {
    let mut cursor = cursor.single_mut();
    let cursor = cursor.as_mut();
    let (player, collider) = player.single();
    let camera = camera.single();
    let mut map = map.single_mut();
    
    let pos = ScreenPos::mouse();
    
    *cursor = Cursor {
        position: pos
    };
    
    if let Some(pos) = cursor.position {
        use MouseButton::*;
        use TileId::*;
        
        let pos = pos.to_world(&camera.0);
        if is_mouse_button_down(Left) ^ is_mouse_button_down(Right) {
            let tile = if is_mouse_button_down(Left) {
                Air
            } else {
                player.get_inventory_item()
            };
            let player_collider = Rect::from_vecs(collider.pos, ivec2(collider.width, collider.height).as_vec2());
            map.place_tile(player_collider, pos, tile);
        }
    }
    
    
}

pub(super) fn draw_cursor(cursor: Query<&Cursor>, camera: Query<&GameCamera>) {
    let cursor = cursor.single();
    let camera = camera.single();
    let tile_set = &*TILE_SET;
    
    if let Some(pos) = cursor.position {
        let pos = pos.to_world(&camera.0);
        if cfg!(target_family = "wasm") {
            draw_texture_ex(
                &tile_set,
                pos.0.x.floor(),
                pos.0.y.floor(),
                WHITE,
                DrawTextureParams {
                    source: Some(
                        Rect::new(
                            5.*TILE_SIZE,
                            0.*TILE_SIZE, TILE_SIZE, TILE_SIZE)),
                    ..Default::default()
                });
        }
        
        let tile_pos = pos.snap().0;
        
        draw_line(
            tile_pos.x  + 1.,
            tile_pos.y + TILE_SIZE,
            tile_pos.x + 1. + TILE_SIZE,
            tile_pos.y + TILE_SIZE,
            1.,
            Color::from_hex(COLOR_BORDER),
        );
        draw_line(
            tile_pos.x + TILE_SIZE,
            tile_pos.y + 1.,
            tile_pos.x + TILE_SIZE,
            tile_pos.y + 1. + TILE_SIZE,
            2.,
            Color::from_hex(COLOR_BORDER),
        );
        draw_rectangle_lines(tile_pos.x, tile_pos.y, TILE_SIZE, TILE_SIZE, 2., Color::from_hex(COLOR_HIGHLIGHT));
    
        // draw_triangle(mouse_pos, mouse_pos + vec2(0., 3.), mouse_pos + vec2(3., 3.), RED);
    }
}