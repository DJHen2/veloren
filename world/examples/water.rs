use common::{terrain::TerrainChunkSize, vol::RectVolSize};
// use self::Mode::*;
use std::{f64, path::PathBuf};
use vek::*;
use veloren_world::{
    sim::{self, MapConfig, MapDebug, WorldOpts, WORLD_SIZE},
    World, CONFIG,
};

const W: usize = 1024;
const H: usize = 1024;

/* enum Mode {
    /// Directional keys affect position of the camera.
    ///
    /// (W A S D move left and right, F B zoom in and out).
    Alt,
    /// Directional keys affect angle of the lens
    ///
    /// (W
    Lens,
    /// Directional keys affect light direction.
    ///
    /// (W A S D move left and right, F B move towards and awaay).
    Light,
}; */

fn main() {
    pretty_env_logger::init();

    let map_file =
        // "map_1575990726223.bin";
        // "map_1575987666972.bin";
        "map_1576046079066.bin";
    let mut _map_file = PathBuf::from("./maps");
    _map_file.push(map_file);

    let world = World::generate(
        5284,
        WorldOpts {
            seed_elements: false,
            // world_file: sim::FileOpts::Load(_map_file),
            world_file: sim::FileOpts::Save,
            ..WorldOpts::default()
        },
    );

    let mut win =
        minifb::Window::new("World Viewer", W, H, minifb::WindowOptions::default()).unwrap();

    let sampler = world.sim();

    let mut focus = Vec3::new(0.0, 0.0, CONFIG.sea_level as f64);
    // Altitude is divided by gain and clamped to [0, 1]; thus, decreasing gain makes
    // smaller differences in altitude appear larger.
    let mut gain = CONFIG.mountain_scale;
    // The Z component during normal calculations is multiplied by gain; thus,
    let mut lgain = 1.0;
    let mut scale = WORLD_SIZE.x as f64 / W as f64;

    // Right-handed coordinate system: light is going left, down, and "backwards" (i.e. on the
    // map, where we translate the y coordinate on the world map to z in the coordinate system,
    // the light comes from -y on the map and points towards +y on the map).  In a right
    // handed coordinate system, the "camera" points towards -z, so positive z is backwards
    // "into" the camera.
    //
    // "In world space the x-axis will be pointing east, the y-axis up and the z-axis will be pointing south"
    let mut light_direction = Vec3::new(-0.8, -1.0, 0.3);

    let mut is_basement = false;
    let mut is_water = true;
    let mut is_shaded = true;
    let mut is_temperature = true;
    let mut is_humidity = true;

    while win.is_open() {
        let config = MapConfig {
            dimensions: Vec2::new(W, H),
            focus,
            gain,
            lgain,
            scale,
            light_direction,

            is_basement,
            is_water,
            is_shaded,
            is_temperature,
            is_humidity,
            is_debug: true,
        };

        let mut buf = vec![0; W * H];
        let MapDebug {
            rivers,
            lakes,
            oceans,
            quads,
        } = config.generate(sampler, |pos, (r, g, b, a)| {
            let i = pos.x;
            let j = pos.y;
            buf[j * W + i] = u32::from_le_bytes([b, g, r, a]);
        });

        let spd = 32.0;
        let lspd = 0.1;
        if win.is_key_down(minifb::Key::P) {
            println!(
                "\
                 Gain / Shade gain: {:?} / {:?}\n\
                 Scale / Focus: {:?} / {:?}\n\
                 Light: {:?}
                 Land(adjacent): (X = temp, Y = humidity): {:?}\n\
                 Rivers: {:?}\n\
                 Lakes: {:?}\n\
                 Oceans: {:?}\n\
                 Total water: {:?}\n\
                 Total land(adjacent): {:?}",
                gain,
                lgain,
                scale,
                focus,
                light_direction,
                quads,
                rivers,
                lakes,
                oceans,
                rivers + lakes + oceans,
                quads.iter().map(|x| x.iter().sum::<u32>()).sum::<u32>()
            );
        }
        if win.get_mouse_down(minifb::MouseButton::Left) {
            if let Some((mx, my)) = win.get_mouse_pos(minifb::MouseMode::Clamp) {
                let pos = (Vec2::<f64>::from(focus) + (Vec2::new(mx as f64, my as f64) * scale))
                    .map(|e| e as i32);
                println!(
                    "Chunk position: {:?}",
                    pos.map2(TerrainChunkSize::RECT_SIZE, |e, f| e * f as i32)
                );
            }
        }
        let is_camera = win.is_key_down(minifb::Key::C);
        if win.is_key_down(minifb::Key::B) {
            is_basement ^= true;
        }
        if win.is_key_down(minifb::Key::H) {
            is_humidity ^= true;
        }
        if win.is_key_down(minifb::Key::T) {
            is_temperature ^= true;
        }
        if win.is_key_down(minifb::Key::O) {
            is_water ^= true;
        }
        if win.is_key_down(minifb::Key::L) {
            is_shaded ^= true;
        }
        if win.is_key_down(minifb::Key::W) {
            if is_camera {
                light_direction.z -= lspd;
            } else {
                focus.y -= spd * scale;
            }
        }
        if win.is_key_down(minifb::Key::A) {
            if is_camera {
                light_direction.x -= lspd;
            } else {
                focus.x -= spd * scale;
            }
        }
        if win.is_key_down(minifb::Key::S) {
            if is_camera {
                light_direction.z += lspd;
            } else {
                focus.y += spd * scale;
            }
        }
        if win.is_key_down(minifb::Key::D) {
            if is_camera {
                light_direction.x += lspd;
            } else {
                focus.x += spd * scale;
            }
        }
        if win.is_key_down(minifb::Key::Q) {
            if is_camera {
                if (lgain * 2.0).is_normal() {
                    lgain *= 2.0;
                }
            } else {
                gain += 64.0;
            }
        }
        if win.is_key_down(minifb::Key::E) {
            if is_camera {
                if (lgain / 2.0).is_normal() {
                    lgain /= 2.0;
                }
            } else {
                gain = (gain - 64.0).max(64.0);
            }
        }
        if win.is_key_down(minifb::Key::R) {
            if is_camera {
                focus.z += spd * scale;
            } else {
                if (scale * 2.0).is_normal() {
                    scale *= 2.0;
                }
                // scale += 1;
            }
        }
        if win.is_key_down(minifb::Key::F) {
            if is_camera {
                focus.z -= spd * scale;
            } else {
                if (scale / 2.0).is_normal() {
                    scale /= 2.0;
                }
                // scale = (scale - 1).max(0);
            }
        }

        win.update_with_buffer(&buf).unwrap();
    }
}
