use std::f32::consts::E;

use macroquad::prelude::*;
use macroquad::telemetry::capture_frame;
use player::{jetpack_decay_curve, Jumping, JETPACK_IMPULSE, JETPACK_TIME};
use tile_map::global_coordinate_to_chunk;

use crate::physics::*;
use crate::player::Player;
use crate::tile_map::wrap_around_vec_in_rect;

pub mod player;
pub mod grid;
pub mod physics;
pub mod tile_map;

pub const VIRTUAL_WIDTH: f32 = 256.0;
pub const VIRTUAL_HEIGHT: f32 = 224.0;
pub const TILE_SIZE: f32 = 16.0;
pub const SMOOTH_CAMERA: bool = false;

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
    let tile_set: Texture2D = load_texture("assets/tileset.png").await.unwrap();
    
    // init objects
    let mut world = World::new();
    let mut player = Player::new(&mut world);

    loop {

        // Get required scaling value
        let scale: f32 = f32::max(f32::min(
            screen_width() / VIRTUAL_WIDTH,
            screen_height() / VIRTUAL_HEIGHT,
        ).floor(), 1.0);

        // Mouse position in the virtual screen
        let virtual_mouse_pos = Vec2 {
            x: (mouse_position().0 - (screen_width() - (VIRTUAL_WIDTH * scale)) * 0.5) / scale,
            y: (mouse_position().1 - (screen_height() - (VIRTUAL_HEIGHT * scale)) * 0.5) / scale,
        };
        let virtual_mouse_pos = render_target_cam.target + virtual_mouse_pos - Rect::new(0., 0., VIRTUAL_WIDTH, VIRTUAL_HEIGHT).size()/2.;


        // ------------------------------------------------------------------------
        // Begin drawing the virtual screen to 'render_target'
        // ------------------------------------------------------------------------
        // render_target_cam.target = world.actor_pos(player.collider);
        set_camera(&render_target_cam);
        
        clear_background(LIGHTGRAY);

        world.map.update(Rect::new(player.position(&world).x, player.position(&world).y, player.size.x, player.size.y), virtual_mouse_pos);
        world.map.draw(&tile_set, Rect::new(0., 0., VIRTUAL_WIDTH, VIRTUAL_HEIGHT));
        player.draw(&world, &tile_set);
        player.update(&mut world);
        
        draw_cursor(virtual_mouse_pos);

        render_target_cam.target = player.position(&world);

        // ------------------------------------------------------------------------
        // Begin drawing the window screen
        // ------------------------------------------------------------------------
        set_default_camera();

        clear_background(BLACK); // Will be the letterbox color

        // Draw 'render_target' to window screen, properly scaled and letterboxed
        draw_texture_ex(
            &render_target.texture,
            (screen_width() - (VIRTUAL_WIDTH * scale)) * 0.5,
            (screen_height() - (VIRTUAL_HEIGHT * scale)) * 0.5,
            WHITE,
            DrawTextureParams {
                dest_size: Some(Rect::new(0., 0., VIRTUAL_WIDTH, VIRTUAL_HEIGHT).size() * scale),
                flip_y: true, // Must flip y otherwise 'render_target' will be upside down
                ..Default::default()
            },
        );
        
        if is_key_pressed(KeyCode::Key3) {
            show_f3 = !show_f3;
        }
        if show_f3 {
            f3(&mut world, &mut player, &render_target_cam, virtual_mouse_pos, &font);
        }

        next_frame().await;
    }
}


fn draw_cursor(mouse_pos: Vec2) {
    let tile_pos: Vec2 = (mouse_pos / 16.).floor() * 16. ;
    draw_line(
        tile_pos.x  + 1.,
        tile_pos.y + TILE_SIZE,
        tile_pos.x + 1. + TILE_SIZE,
        tile_pos.y + TILE_SIZE,
        1.,
        Color::from_hex(0x3e042d),
    );
    draw_line(
        tile_pos.x + TILE_SIZE,
        tile_pos.y + 1.,
        tile_pos.x + TILE_SIZE,
        tile_pos.y + 1. + TILE_SIZE,
        2.,
        Color::from_hex(0x3e042d),
    );
    draw_rectangle_lines(tile_pos.x, tile_pos.y, TILE_SIZE, TILE_SIZE, 2., Color::from_hex(0xf93f8d));
    draw_triangle(mouse_pos, mouse_pos + vec2(0., 3.), mouse_pos + vec2(3., 3.), RED);
}

fn f3(world: &mut World, player: &mut Player, camera: &Camera2D, mouse_position: Vec2, font: &Font) {
   
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
        &format!("Player in chunk: {:?}", global_coordinate_to_chunk(player.position(&world))),
        false, 4, 0., WHITE, &font
    );
    
    draw_f3_text(
        &format!("Mouse Pos: {:?}", mouse_position.as_ivec2()),
        false, 5, 0., WHITE, &font
    );
    
    if world.collide_check(player.collider, player.position(world) - vec2(0., 1.)) {
        draw_f3_text(
            "U", 
            false, 6, 0., Color::from_hex(0x00FF00), &font
        );
    }
    if world.collide_check(player.collider, player.position(world) + vec2(0., 1.)) {
        draw_f3_text(
            "D", 
            false, 6, 12.*1., Color::from_hex(0xFF0000), &font
        );
    }
    if world.collide_check(player.collider, player.position(world) - vec2(1., 0.)) {
        draw_f3_text(
            "L", 
            false, 6, 12.*2., Color::from_hex(0xFFFF00), &font
        );
    }
    if world.collide_check(player.collider, player.position(world) + vec2(1., 0.)) {
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
    
    // right
    // draw_f3_text(
    //     &format!("Cam Pos: {:?}", camera.target),
    //     true, 1, WHITE, &font
    // );
    
    
    

    // if !Rect::new(0., 0., VIRTUAL_WIDTH, VIRTUAL_HEIGHT).overlaps(&Rect::new(world.actor_pos(player.collider).x, world.actor_pos(player.collider).y, TILE_SIZE - 2., TILE_SIZE)) {
    //     draw_text("OOB", screen_width()-30., 60., 18., Color::from_hex(0xFF0000));
    // }

    // draw_text(&format!("INCHUNK POS: {}", wrap_around_vec_in_rect(
    //     Rect::new(0., 0., VIRTUAL_WIDTH, VIRTUAL_HEIGHT),
    //     world.actor_pos(player.collider)
    // )), screen_width()-500., 12.*7., 18., Color::from_hex(0xFFFFFF));
    
    // if world.collide_check(player.collider, player.position - vec2(0., 1.)) {
    //     draw_text("U", 12.*0., 32., 18., Color::from_hex(0x00FF00));
    // }
    // if world.collide_check(player.collider, player.position + vec2(0., 1.)) {
    //     draw_text("D", 12.*1., 32., 18., Color::from_hex(0xFF0000));
    // }
    // if world.collide_check(player.collider, player.position - vec2(1., 0.)) {
    //     draw_text("L", 12.*2., 32., 18., Color::from_hex(0xFFFF00));
    // }
    // if world.collide_check(player.collider, player.position + vec2(1., 0.)) {
    //     draw_text("R", 12.*3., 32., 18., Color::from_hex(0xFF00FF));
    // }
    // draw_text(&format!("{:?}", player.facing), 0., 42., 18., Color::from_hex(0xF0B357 << player.facing as u32));


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
    
    // let text_dims = measure_text(text, Some(font), font_size, font_scale);
    // let mut text_box = Rect::new(
    //     offset, 
    //     (font_size * line as u16) as f32 - text_dims.offset_y, 
    //     text_dims.width * 1.5, 
    //     text_dims.height * 1.3
    // );
    
    // draw_rectangle_lines(text_box.x, text_box.y, text_box.w, text_box.h, 5.0, MAGENTA);
    
    
 
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
