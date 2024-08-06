use bevy_ecs::schedule::IntoSystemConfigs;
use camera::init_camera;
use camera::letterbox_camera;
use camera::refocus_camera;
use camera::setup_camera;
use cursor::draw_cursor;
use cursor::init_cursor;
use cursor::update_cursor;
use player::draw_player;
use player::move_player;
use tile_map::draw_map;
use tile_map::init_map;
use tile_map::timed_save;
use ui::draw_ui;
use ui::init_ui;

use crate::app::ScheduleLabel_::*;
use crate::app::Plugin;

pub mod player;
pub mod tile_map;
pub mod camera;
pub mod ui;
pub mod cursor;

pub struct EnitityPlugin;

impl Plugin for EnitityPlugin {
    fn build(&self, app: &mut crate::app::App) {
        app
            .add_systems(Startup, (init_camera, init_map, init_cursor, init_ui))
            .add_systems(Update, (
                (timed_save, update_cursor),
                (draw_map, draw_cursor, draw_ui).chain(),
                (draw_player, move_player, refocus_camera).chain(),
                ).chain()
            )
            .add_systems(PreUpdate, setup_camera)
            .add_systems(PostUpdate, letterbox_camera);
    }
}