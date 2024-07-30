mod player;
mod grid;
mod physics;

use macroquad::prelude::*;
use crate::player::Player;
use physics::*;
use crate::grid::Grid;

const VIRTUAL_WIDTH: f32 = 240.0;
const VIRTUAL_HEIGHT: f32 = 160.0;
const TILE_SIZE: f32 = 16.0;

#[macroquad::main("Letterbox")]
async fn main() {
    // Setup 'render_target', used to hold the rendering result so we can resize it
    let render_target = render_target(VIRTUAL_WIDTH as u32, VIRTUAL_HEIGHT as u32);
    render_target.texture.set_filter(FilterMode::Nearest);

    // Setup camera for the virtual screen, that will render to 'render_target'
    let mut render_target_cam =
        Camera2D::from_display_rect(Rect::new(0., 0., VIRTUAL_WIDTH, VIRTUAL_HEIGHT));
    render_target_cam.render_target = Some(render_target.clone());

    let tile_set: Texture2D = load_texture("assets/tileset.png").await.unwrap();

    let mut tiles: Grid<Tile> = Grid::new_filled(
        (VIRTUAL_WIDTH / TILE_SIZE) as usize,
        (VIRTUAL_HEIGHT / TILE_SIZE) as usize,
        |point| if point.y > 5 { Tile::Solid } else { Tile::Empty },
        Tile::Empty
    );

    let mut world = World::new();
    world.add_static_tiled_layer(tiles.clone(), TILE_SIZE, TILE_SIZE, (VIRTUAL_WIDTH / TILE_SIZE) as usize, 1);
    dbg!(&world.static_tiled_layers);


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

        // ------------------------------------------------------------------------
        // Begin drawing the virtual screen to 'render_target'
        // ------------------------------------------------------------------------
        set_camera(&render_target_cam);

        clear_background(LIGHTGRAY);

        let snapped_vmouse_pos= (virtual_mouse_pos.x as u32 / 16 * 16, virtual_mouse_pos.y as u32 / 16 * 16);
        let tiled_vmouse_pos= (virtual_mouse_pos.x as usize / 16, virtual_mouse_pos.y as usize / 16);

        if is_mouse_button_down(MouseButton::Left) && !is_mouse_button_down(MouseButton::Right) {
            tiles[tiled_vmouse_pos] = Tile::Empty;
            world.static_tiled_layers[0].static_colliders[tiled_vmouse_pos] = Tile::Empty;
            // world.add_static_tiled_layer(tiles.clone().array, TILE_SIZE, TILE_SIZE, (VIRTUAL_WIDTH / TILE_SIZE) as usize, 1);
        } else if is_mouse_button_down(MouseButton::Right) && !is_mouse_button_down(MouseButton::Left) {
            tiles[tiled_vmouse_pos] = Tile::Solid;
            world.static_tiled_layers[0].static_colliders[tiled_vmouse_pos] = Tile::Solid;
        }


        draw_circle(virtual_mouse_pos.x, virtual_mouse_pos.y, 15.0, BLACK);
        draw_rectangle(snapped_vmouse_pos.0 as f32, snapped_vmouse_pos.1 as f32, TILE_SIZE, TILE_SIZE, Color::from_hex(0xFF00FF));

        world.static_tiled_layers[0].static_colliders.for_each(|point, tile| {
            let color = match tile {
                Tile::Solid => Color::from_hex(0xBB0000),
                Tile::Empty => Color::from_hex(0x00BB00),
                _ => Color::from_hex(0xBB00BB)
            };
            draw_rectangle(point.x as f32 * TILE_SIZE, point.y as f32 * TILE_SIZE, 1. * TILE_SIZE, 1. * TILE_SIZE, color);
        });

        tiles.for_each(|point, tile| {
            let color = match tile {
                Tile::Solid => Color::from_hex(0xFF0000),
                _ => Color::from_hex(0x00FF00),
            };
            draw_rectangle_lines(point.x as f32 * TILE_SIZE, point.y as f32 * TILE_SIZE, 1. * TILE_SIZE, 1. * TILE_SIZE, 1.,  color);
        });


        player.update(&mut world);

        player.draw(&tile_set);




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
                dest_size: Some(vec2(VIRTUAL_WIDTH * scale, VIRTUAL_HEIGHT * scale)),
                flip_y: true, // Must flip y otherwise 'render_target' will be upside down
                ..Default::default()
            },
        );

        draw_text(&format!("Position: {:#?}", player.position), 0., 12., 18., Color::from_hex(0xFFFFFF));
        draw_text(&format!("Speed: {:#?}", player.speed), 0., 22., 18., Color::from_hex(0xFFFFFF));
        if world.collide_check(player.collider, player.position - vec2(0., 1.)) {
            draw_text("U", 12.*0., 32., 18., Color::from_hex(0x00FF00));
        }
        if world.collide_check(player.collider, player.position + vec2(0., 1.)) {
            draw_text("D", 12.*1., 32., 18., Color::from_hex(0xFF0000));
        }
        if world.collide_check(player.collider, player.position - vec2(1., 0.)) {
            draw_text("L", 12.*2., 32., 18., Color::from_hex(0xFFFF00));
        }
        if world.collide_check(player.collider, player.position + vec2(1., 0.)) {
            draw_text("R", 12.*3., 32., 18., Color::from_hex(0xFF00FF));
        }
        draw_text(&format!("{:?}", player.facing), 0., 42., 18., Color::from_hex(0xF0B357 << player.facing as u32));
        
        next_frame().await;
    }
}