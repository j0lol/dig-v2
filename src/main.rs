use macroquad::prelude::*;

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


#[macroquad::main("Game")]
async fn main() {
    
    // Set up camera & screen
    let render_target = render_target(VIRTUAL_WIDTH as u32, VIRTUAL_HEIGHT as u32);
    render_target.texture.set_filter(FilterMode::Nearest);
    let mut render_target_cam =
        Camera2D::from_display_rect(Rect::new(0., 0., VIRTUAL_WIDTH, VIRTUAL_HEIGHT));
    render_target_cam.render_target = Some(render_target.clone());

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

        player.draw(&world, &tile_set);
        player.update(&mut world);
        world.map.update(Rect::new(player.position(&world).x, player.position(&world).y, player.size.x, player.size.y), virtual_mouse_pos);
        world.map.draw(&tile_set, Rect::new(0., 0., VIRTUAL_WIDTH, VIRTUAL_HEIGHT));
        
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
        BLACK,
    );
    draw_line(
        tile_pos.x + TILE_SIZE,
        tile_pos.y + 1.,
        tile_pos.x + TILE_SIZE,
        tile_pos.y + 1. + TILE_SIZE,
        2.,
        BLACK,
    );
    draw_rectangle_lines(tile_pos.x, tile_pos.y, TILE_SIZE, TILE_SIZE, 2., Color::from_hex(0xFF00FF));
    draw_triangle(mouse_pos, mouse_pos + vec2(0., 3.), mouse_pos + vec2(3., 3.), RED);
}

fn f3(world: &mut World, player: &mut Player, camera: Camera2D, mouse_position: Vec2) {
    draw_text(&format!("Position: {:#?}", world.actor_pos(player.collider)), 0., 12., 18., Color::from_hex(0xFFFFFF));
    draw_text(&format!("Speed: {:#?}", player.speed), 0., 22., 18., Color::from_hex(0xFFFFFF));
    draw_text(&format!("Cam Pos: {:#?}", camera.target), screen_width()-300., 12., 18., Color::from_hex(0xFFFFFF));
    draw_text(&format!("VMousePos: {:#?}", mouse_position), screen_width()-500., 48., 18., Color::from_hex(0xFFFFFF));

    if !Rect::new(0., 0., VIRTUAL_WIDTH, VIRTUAL_HEIGHT).overlaps(&Rect::new(world.actor_pos(player.collider).x, world.actor_pos(player.collider).y, TILE_SIZE - 2., TILE_SIZE)) {
        draw_text("OOB", screen_width()-30., 60., 18., Color::from_hex(0xFF0000));
    }
    draw_text(&format!("Player in chunk: {:#?}", player.get_chunk(&world)), screen_width()-500., 12.*6., 18., Color::from_hex(0xFFFFFF));

    draw_text(&format!("INCHUNK POS: {}", wrap_around_vec_in_rect(
        Rect::new(0., 0., VIRTUAL_WIDTH, VIRTUAL_HEIGHT),
        world.actor_pos(player.collider)
    )), screen_width()-500., 12.*7., 18., Color::from_hex(0xFFFFFF));

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
    draw_text(&format!("{:?}", player.facing), 0., 42., 18., Color::from_hex(0xF0B357 << player.facing as u32));


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