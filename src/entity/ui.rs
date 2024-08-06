use crate::physics2::Collider;
use std::f32::consts::PI;
use bevy_ecs::prelude::*;
use macroquad::prelude::*;
use crate::{draw_bordered_rect, DEFAULT_FONT, SAVE_TIMER, TILE_SET, TILE_SIZE, VIRTUAL_HEIGHT, VIRTUAL_WIDTH};
use crate::entity::player::{JETPACK_TIME, Jumping, Player};
use crate::position::{RectExtend, WorldPos};

use super::camera::GameCamera;
use super::player::PlayerTag;
use super::tile_map::SaveTimer;

const UI_WIDTH: f32 = 87.0;

pub const COLOR_HIGHLIGHT: u32 = 0xf93f8d;
pub const COLOR_BASE: u32 = 0x550b39;
pub const COLOR_SURFACE: u32 = 0x6d1345;
pub const COLOR_BORDER: u32 = 0x3e042d;



pub(super) fn init_ui(mut commands: Commands) {
    commands.spawn(Ui);
}

#[derive(Component, Default)]
pub struct Ui;

pub(super) fn draw_ui(
    mut v_camera: Query<&GameCamera>, 
    mut v_player: Query<(&Player, &Collider), With<PlayerTag>>,
    save_timer: Res<SaveTimer>
) {
    let camera2d = &v_camera.get_single_mut().unwrap().0;
    let (player, collider) = v_player.get_single_mut().unwrap();
    
    let font = &*DEFAULT_FONT;
    
    let margin = 2.0;

    let base_ui_rect = Rect::new(
        camera2d.screen_to_world(vec2(0., 0.)).x,
        camera2d.screen_to_world(vec2(0., 0.)).y,
        VIRTUAL_WIDTH,
        VIRTUAL_HEIGHT
    );

    let hotbar_rect = Rect::new(
        base_ui_rect.left() + margin,
        base_ui_rect.bottom() - 24.0 - margin,
        UI_WIDTH,
        24.0
    );
    
    draw_jump_velocity_bar(base_ui_rect, margin, hotbar_rect.h, player);


    draw_bordered_rect(
        hotbar_rect,
        Color::from_hex(COLOR_BORDER),
        Color::from_hex(COLOR_BASE)
    );
    
    for i in 1..=4 {
        player.get_inventory_index();
        let border_color =  if player.get_inventory_index() == i-1 {
            COLOR_HIGHLIGHT
        } else {
            COLOR_BORDER
        };
        
        draw_bordered_rect(
            Rect::from_vecs(hotbar_rect.point() + 3. + vec2((i-1) as f32 * 21. , 0.), Vec2::splat(18.)),
            Color::from_hex(border_color),
            Color::from_hex(COLOR_BORDER)
        );
        draw_from_tile_set(player.inventory[i-1].val().sprite.unwrap(),
        hotbar_rect.point() + 3. + vec2((i-1) as f32 * 21. , 0.) + vec2(1., 1.)
        )
    }
    
    let player_position = WorldPos(collider.pos).to_tile().0;
    
    draw_text_ex(
        &format!("[{:.1}, {:.1}]", player_position.x, player_position.y), 
        base_ui_rect.x + 2.0, 
        base_ui_rect.bottom() - 36.0,
        TextParams {
        font: Some(font),
        font_size: 16,
        color: Color::from_hex(COLOR_HIGHLIGHT),
        ..Default::default()
    });
    
    if (SAVE_TIMER - 2.0..SAVE_TIMER).contains(&save_timer.0) {
        let mut color = Color::from_hex(COLOR_HIGHLIGHT);
        color.a = ease_out(SAVE_TIMER - save_timer.0, 2.0);
        draw_text_ex(
            "Saved.", 
            base_ui_rect.right() - 32.0, 
            base_ui_rect.bottom() - 2.0,
            TextParams {
            font: Some(font),
            font_size: 16,
            color,
            ..Default::default()
        });
    }


}

fn draw_jump_velocity_bar(base_ui_rect: Rect, margin: f32, hotbar_height: f32, player: &Player) {

    draw_bordered_rect(
        Rect::new(
            base_ui_rect.left() + margin,
            base_ui_rect.bottom() - 2.0 - (margin * 3.0) - hotbar_height,
            UI_WIDTH,
            2.0
        ),
        Color::from_hex(0x3e042d),
        Color::from_hex(0x550b39)
    );

    let speed = match player.jumping {
        Jumping::Not => 0.0,
        Jumping::Jumping => (-player.speed.y / 150.).max(0.0).min(1.0),
        Jumping::Jetpacking(time_left) => (time_left/JETPACK_TIME).max(0.0).min(1.0)
    };
    if speed > 0.0 {
        draw_bordered_rect(
            Rect::new(
                base_ui_rect.left() + margin,
                base_ui_rect.bottom() - 2.0 - (margin * 3.0) - hotbar_height,
                UI_WIDTH * speed,
                2.0
            ),
            Color::from_hex(COLOR_BORDER),
            Color::from_hex(COLOR_HIGHLIGHT)
        );
    }
}


fn ease_out(x: f32, t: f32) -> f32 {
    let n = (1.0 / t) * x;
    ((PI*n).cos() + 1.0) / 2.0
}


pub fn draw_from_tile_set(tile_index: u32, position: Vec2) {
    
    let tile_set = &*TILE_SET;
    let tileset_width = tile_set.width() / TILE_SIZE;
    
    let sprite_rect =  Rect::from_vecs(
        vec2(
            (tile_index as f32 % tileset_width).floor() * TILE_SIZE,
            (tile_index as f32 / tileset_width).floor() * TILE_SIZE,
        ), Vec2::splat(TILE_SIZE));
    
    draw_texture_ex(
        tile_set,
        position.x,
        position.y,
        WHITE,
        DrawTextureParams {
            source: Some(sprite_rect),
            ..Default::default()
        }
    );
}