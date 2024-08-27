use std::io::BufWriter;
use std::sync::LazyLock;
use app::App;
use app::ScheduleLabel_::Startup;
use asefile::AsepriteFile;
use bevy_ecs::system::Commands;
use entity::player::new_player;
use entity::player::PlayerTag;
use entity::tile_map::ChunkMap;
use entity::EntityPlugin;
use image::codecs::png::PngEncoder;
use macroquad::prelude::*;

pub mod entity;
pub mod grid;
pub mod physics2;
pub mod position;
pub mod tile;
pub mod app;

pub const VIRTUAL_WIDTH: f32 = 256.0;
pub const VIRTUAL_HEIGHT: f32 = 224.0;
pub const TILE_SIZE: f32 = 16.0;
pub const SMOOTH_CAMERA: bool = false;
pub const SAVE_TIMER: f32 = 10.0;

static TILE_SET: LazyLock<Texture2D> = LazyLock::new(|| {
    let ase = AsepriteFile::read(&include_bytes!("../assets/tileset.ase")[..]).unwrap();
    let image = ase.frame(0).image();

    // Complicated process of writing to an in-mem buf
    let mut c = std::io::Cursor::new(Vec::new());
    image.write_to(&mut c, image::ImageFormat::Png).unwrap();
    let buf = &c.into_inner()[..];

    Texture2D::from_file_with_format(buf, Some(ImageFormat::Png))
});

static DEFAULT_FONT: LazyLock<Font> = LazyLock::new(|| {
    let mut font =  load_ttf_font_from_bytes(include_bytes!("../assets/m5x7.ttf")).unwrap();
    font.set_filter(FilterMode::Nearest);
    font
});

#[cfg(target_family = "wasm")]
const IS_WASM: bool = true;

#[cfg(not(target_family = "wasm"))]
const IS_WASM: bool = false;



#[macroquad::main("Game")]
async fn main() {
    if cfg!(target_family = "wasm") {
        show_mouse(false);
    }
   
    let mut app = App::new();
    
    app
        .add_systems(Startup, init_entities)
        // .add_plugin(physics2::PhysicsPlugin)
        .add_plugin(EntityPlugin);

    app.run().await;
    
}



// fn f3(world: &mut World, player: &mut Player, camera: &Camera2D) {

//     let font = DEFAULT_FONT.get().unwrap();
//     let mouse_position = ScreenPos::mouse();

//     // left 
//     draw_f3_text(
//         &format!("Position: {:?}", world.actor_pos(player.collider)),
//         false, 1, 0., WHITE, &font
//     );
    
//     draw_f3_text(
//         &format!("Speed: {:?}", player.speed),
//         false, 2, 0., WHITE, &font
//     );
    
//     draw_f3_text(
//         &format!("PlayerJump: {:?}", player.jumping),
//         false, 3, 0., WHITE, &font
//     );
    
//     draw_f3_text(
//         &format!("Player in chunk: {:?}", player.position(&world).to_chunk()),
//         false, 4, 0., WHITE, &font
//     );
    
//     draw_f3_text(
//         &format!("Mouse Pos: {:?}", mouse_position),
//         false, 5, 0., WHITE, &font
//     );
//     if let Some(mouse_position) = mouse_position {
//         draw_f3_text(
//             &format!("Mouse Pos World: {:?}",mouse_position.to_world(camera).0),
//             false, 5, 200.0, WHITE, &font
//         );

//     }

//     if world.collide_check(player.collider, player.position(world).0 - vec2(0., 1.)) {
//         draw_f3_text(
//             "U", 
//             false, 6, 0., Color::from_hex(0x00FF00), &font
//         );
//     }
//     if world.collide_check(player.collider, player.position(world).0 + vec2(0., 1.)) {
//         draw_f3_text(
//             "D", 
//             false, 6, 12.*1., Color::from_hex(0xFF0000), &font
//         );
//     }
//     if world.collide_check(player.collider, player.position(world).0 - vec2(1., 0.)) {
//         draw_f3_text(
//             "L", 
//             false, 6, 12.*2., Color::from_hex(0xFFFF00), &font
//         );
//     }
//     if world.collide_check(player.collider, player.position(world).0 + vec2(1., 0.)) {
//         draw_f3_text(
//             "R", 
//             false, 6, 12.*3., Color::from_hex(0xFF00FF), &font
//         );
//     }
    
//     if let Jumping::Jetpacking(time_left) = player.jumping {
//         if time_left > 0.0 {
//             draw_f3_text(
//                 &format!("Jetpack Impulse: {:.2}", jetpack_decay_curve(time_left)),
//                 false, 7, 0., WHITE, &font
//             );
            
//         }
//     }

//     for (pos, chunk) in world.map.around_focus().items {
//         let offset = vec2(0., 400.)
//             + ( vec2(pos.0 as f32, pos.1 as f32)
//             * (Rect::new(0., 0., VIRTUAL_WIDTH, VIRTUAL_HEIGHT).size() / 8.)
//         )
//             + (
//             2. * Rect::new(0., 0., VIRTUAL_WIDTH, VIRTUAL_HEIGHT).size() / 8.
//         );
//         chunk.dbg_draw(offset);
//     }
// }

fn init_entities(mut commands: Commands) {
    let mut chunk_map = ChunkMap::load().unwrap_or(ChunkMap::default());
    let (player , collider, actor) = new_player(&mut chunk_map);
    dbg!(&player);
    commands.spawn(chunk_map);
    commands.spawn((PlayerTag, player, collider, actor));
}

#[test]
fn wraparound() {
    use crate::entity::tile_map::wrap_around_vec_in_rect;
    
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
