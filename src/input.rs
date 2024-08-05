use std::sync::{LazyLock, Mutex};

use macroquad::{input::mouse_wheel, math::{vec2, Vec2}};

const THRESHOLD: f32 = 16.0;
static SCROLL_STEP: LazyLock<Mutex<Vec2>> = LazyLock::new(|| Mutex::new(Vec2::splat(0.0)));


pub fn get_scroll_stepped() -> Vec2 {
    let scr = mouse_wheel();
    let scr = vec2(scr.0, scr.1);
    
    scr
}
// wrapping i8 
// wrapping add with sign
// match from -128 to -64
// -63 to ...
// 