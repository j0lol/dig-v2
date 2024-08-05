use std::sync::OnceLock;
use macroquad::prelude::*;
use player::{jetpack_decay_curve, Jumping};
use position::{RectExtend, ScreenPos};
use crate::physics::*;
use crate::player::Player;
use crate::ui::draw_ui;

pub mod player;
pub mod grid;
pub mod physics;
pub mod tile_map;
pub mod ui;
pub mod position;
pub mod tile;
pub mod input;


pub const VIRTUAL_WIDTH: f32 = 256.0;
pub const VIRTUAL_HEIGHT: f32 = 224.0;
pub const TILE_SIZE: f32 = 16.0;
pub const SMOOTH_CAMERA: bool = false;
pub const SAVE_TIMER: f32 = 10.0;

static TILE_SET: OnceLock<Texture2D> = OnceLock::new();

#[cfg(target_family = "wasm")]
const IS_WASM: bool = true;

#[cfg(not(target_family = "wasm"))]
const IS_WASM: bool = false;

#[macroquad::main("Game")]
async fn main() {
    
    let mut show_f3 = false;
    
    if IS_WASM {
        show_mouse(false);
    }
    
    // Set up camera & screen
    let render_target = render_target(VIRTUAL_WIDTH as u32, VIRTUAL_HEIGHT as u32);
    render_target.texture.set_filter(FilterMode::Nearest);
    let mut render_target_cam =
        Camera2D::from_display_rect(Rect::new(0., 0., VIRTUAL_WIDTH, VIRTUAL_HEIGHT));
    render_target_cam.render_target = Some(render_target.clone());
    
    let mut font = load_ttf_font("./assets/m5x7.ttf")
        .await
        .unwrap();

    font.set_filter(FilterMode::Nearest);

    // load assets
    TILE_SET.set(load_texture("assets/tileset.png").await.unwrap()).unwrap();
    
    // init objects
    let mut world = World::load().unwrap_or(World::new());
    let mut player = Player::new(&mut world);
    
    let mut save_timer = 10.0;

    world.save();
    loop {
        
        if save_timer < 0.0 {
            world.save();
            save_timer = SAVE_TIMER;
        } else {
            save_timer -= get_frame_time()
        }
        
        if is_key_down(KeyCode::LeftShift) && is_key_pressed(KeyCode::S) {
            world.save();
        }
        if is_key_down(KeyCode::LeftShift) && is_key_pressed(KeyCode::L) {
            match World::load() {
                Some(w) =>{world = w}
                None => {}
            }
        }

        // Get required scaling value
        let scale: f32 = f32::max(f32::min(
            screen_width() / VIRTUAL_WIDTH,
            screen_height() / VIRTUAL_HEIGHT,
        ).floor(), 1.0);

        // ------------------------------------------------------------------------
        // Begin drawing the virtual screen to 'render_target'
        // ------------------------------------------------------------------------
        // render_target_cam.target = world.actor_pos(player.collider);
        set_camera(&render_target_cam);
        
        clear_background(LIGHTGRAY);

        world.map.update(
            Rect::from_vecs(player.position(&world).0, player.size),
            ScreenPos::mouse().map(|pos| {
                pos.to_world(&render_target_cam)
            }
        ),
        &player
        );
        world.map.draw(ScreenPos::screen_rect());
        player.draw(&world);
        player.update(&mut world);
        
        // draw_cursor(virtual_mouse_pos, &tile_set);

        draw_ui(&render_target_cam, &player, &world, &font, save_timer);


        render_target_cam.target = player.position(&world).0;

        // ------------------------------------------------------------------------
        // Begin drawing the window screen
        // ------------------------------------------------------------------------
        set_default_camera();

        clear_background(BLACK); // Will be the letterbox color

        // Draw 'render_target' to window screen, properly scaled and letterboxed
        draw_texture_ex(
            &render_target.texture,
            ((screen_width() - (VIRTUAL_WIDTH * scale)) * 0.5).floor(),
            ((screen_height() - (VIRTUAL_HEIGHT * scale)) * 0.5).floor(),
            WHITE,
            DrawTextureParams {
                dest_size: Some(ScreenPos::screen().0 * scale),
                flip_y: true, // Must flip y otherwise 'render_target' will be upside down
                ..Default::default()
            },
        );
        
        if is_key_pressed(KeyCode::Key3) {
            show_f3 = !show_f3;
        }
        if show_f3 {
            f3(&mut world, &mut player, &render_target_cam, &font);
        }

        next_frame().await;
    }
}



fn f3(world: &mut World, player: &mut Player, camera: &Camera2D, font: &Font) {

    let mouse_position = ScreenPos::mouse();

    // left 
    draw_f3_text(
        &format!("Position: {:?}", world.actor_pos(player.collider)),
        false, 1, 0., WHITE, &font
    );
    
    draw_f3_text(
        &format!("Speed: {:?}", player.speed),
        false, 2, 0., WHITE, &font
    );
    
    draw_f3_text(
        &format!("PlayerJump: {:?}", player.jumping),
        false, 3, 0., WHITE, &font
    );
    
    draw_f3_text(
        &format!("Player in chunk: {:?}", player.position(&world).to_chunk()),
        false, 4, 0., WHITE, &font
    );
    
    draw_f3_text(
        &format!("Mouse Pos: {:?}", mouse_position),
        false, 5, 0., WHITE, &font
    );
    if let Some(mouse_position) = mouse_position {
        draw_f3_text(
            &format!("Mouse Pos World: {:?}",mouse_position.to_world(camera).0),
            false, 5, 200.0, WHITE, &font
        );

    }

    if world.collide_check(player.collider, player.position(world).0 - vec2(0., 1.)) {
        draw_f3_text(
            "U", 
            false, 6, 0., Color::from_hex(0x00FF00), &font
        );
    }
    if world.collide_check(player.collider, player.position(world).0 + vec2(0., 1.)) {
        draw_f3_text(
            "D", 
            false, 6, 12.*1., Color::from_hex(0xFF0000), &font
        );
    }
    if world.collide_check(player.collider, player.position(world).0 - vec2(1., 0.)) {
        draw_f3_text(
            "L", 
            false, 6, 12.*2., Color::from_hex(0xFFFF00), &font
        );
    }
    if world.collide_check(player.collider, player.position(world).0 + vec2(1., 0.)) {
        draw_f3_text(
            "R", 
            false, 6, 12.*3., Color::from_hex(0xFF00FF), &font
        );
    }
    
    if let Jumping::Jetpacking(time_left) = player.jumping {
        if time_left > 0.0 {
            draw_f3_text(
                &format!("Jetpack Impulse: {:.2}", jetpack_decay_curve(time_left)),
                false, 7, 0., WHITE, &font
            );
            
        }
    }

    for (pos, chunk) in world.map.around_focus().items {
        let offset = vec2(0., 400.)
            + ( vec2(pos.0 as f32, pos.1 as f32)
            * (Rect::new(0., 0., VIRTUAL_WIDTH, VIRTUAL_HEIGHT).size() / 8.)
        )
            + (
            2. * Rect::new(0., 0., VIRTUAL_WIDTH, VIRTUAL_HEIGHT).size() / 8.
        );
        chunk.dbg_draw(offset);
    }
}


#[test]
fn wraparound() {
    use crate::tile_map::wrap_around_vec_in_rect;
    
    let rect = Rect::new(0., 0., 10., 10.);
    
    assert_eq!(wrap_around_vec_in_rect(rect, vec2(0., 0.)), vec2(0., 0.));
    assert_eq!(wrap_around_vec_in_rect(rect, vec2(9., 9.)), vec2(9., 9.));
    assert_eq!(wrap_around_vec_in_rect(rect, vec2(-1., -1.)), vec2(9., 9.));
    assert_eq!(wrap_around_vec_in_rect(rect, vec2(10., 10.)), vec2(0., 0.));
}

fn draw_f3_text(text: &str, right: bool, line: u8, offset: f32, color: Color, font: &Font) { 
    let font_size = 16;
    let font_scale = 1.0;
    
    let offset: f32 = if right {
        screen_width() - (measure_text(text, Some(font), font_size, font_scale).width) * 1.5 - offset
    } else { 0. + offset};

    draw_text_ex(
        text,
        offset,
        font_size as f32 * line as f32, 
        TextParams {
            font_size,
            font_scale,
            color,
            ..Default::default()
        }
    );
}

enum DrawRectType {
    Filled,
    Lines(f32)
}

fn draw_rect(rect: Rect, draw_type: DrawRectType, params: DrawRectangleParams) {
    match draw_type {
        DrawRectType::Filled => {
            draw_rectangle_ex(rect.x, rect.y, rect.w, rect.h, params);
        }
        DrawRectType::Lines(line_thickness) => {
            draw_rectangle_lines_ex(rect.x, rect.y, rect.w, rect.h, line_thickness, params)
        }
    }
}

fn draw_bordered_rect(rect: Rect, border_color: Color, fill_color: Color) {
    draw_rectangle_ex(rect.x, rect.y, rect.w, rect.h, DrawRectangleParams {
        color: fill_color,
        ..Default::default()
    });
    draw_rectangle_lines_ex(rect.x - 1.0, rect.y - 1.0, rect.w + 2.0, rect.h + 2.0, 1.0, DrawRectangleParams {
        color: border_color,
        ..Default::default()
    });
}

struct Timer {
    seconds: u32,
    on_tick: Box<dyn Fn(u32) -> ()>
}