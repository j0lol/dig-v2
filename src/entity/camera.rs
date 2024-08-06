use macroquad::prelude::*;
use bevy_ecs::prelude::*;
use crate::physics2::Collider;
use crate::position::ScreenPos;
use crate::VIRTUAL_HEIGHT;
use crate::VIRTUAL_WIDTH;

use super::player::PlayerTag;

#[derive(Component)]
pub struct GameCamera(pub Camera2D);

pub(super) fn init_camera(mut commands: Commands) {
    let render_target = render_target(VIRTUAL_WIDTH as u32, VIRTUAL_HEIGHT as u32);
    render_target.texture.set_filter(FilterMode::Nearest);
    let mut render_target_cam =
        Camera2D::from_display_rect(Rect::new(0., 0., VIRTUAL_WIDTH, VIRTUAL_HEIGHT));
    render_target_cam.render_target = Some(render_target);
    commands.spawn(GameCamera(render_target_cam));
}

pub fn setup_camera(camera: Query<&GameCamera>) {
    let camera = camera.single();
    set_camera(&camera.0);
    // set_default_camera();
    clear_background(LIGHTGRAY);
}

pub(super) fn refocus_camera(mut camera: Query<&mut GameCamera>, player: Query<&Collider, With<PlayerTag>>) {
    let mut camera = camera.single_mut();
    let player = player.single();
    camera.0.target = player.pos;
}

pub fn letterbox_camera(camera: Query<&GameCamera>) {
    let scale: f32 = f32::max(f32::min(
        screen_width() / VIRTUAL_WIDTH,
        screen_height() / VIRTUAL_HEIGHT,
    ).floor(), 1.0);
    
    set_default_camera();
    clear_background(BLACK); // Will be the letterbox color
    
    let camera = camera.single();
    // Draw 'render_target' to window screen, properly scaled and letterboxed
    draw_texture_ex(
        &camera.0.render_target.as_ref().unwrap().texture,
        ((screen_width() - (VIRTUAL_WIDTH * scale)) * 0.5).floor(),
        ((screen_height() - (VIRTUAL_HEIGHT * scale)) * 0.5).floor(),
        WHITE,
        DrawTextureParams {
            dest_size: Some(ScreenPos::screen().0 * scale),
            flip_y: true, // Must flip y otherwise 'render_target' will be upside down
            ..Default::default()
        },
    );
}