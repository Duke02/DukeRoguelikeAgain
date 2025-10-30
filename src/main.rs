mod entities;
mod error;
mod models;
mod systems;

use crate::entities::spawn_goblin;
use crate::models::input::InputState;
use crate::models::stats::Health;
use crate::models::{Player, Position, Renderable};
use crate::systems::{AiSystem, DamageSystem, DeadCollector, InputSystem, SystemFunc};
use doryen_rs::{App, AppOptions, DoryenApi, Engine, UpdateEvent};
use hecs::World;
use std::cell::RefCell;
use std::sync::Arc;
use tracing::log::{Level, LevelFilter};
use tracing_subscriber::field::MakeExt;
use tracing_subscriber::fmt::format;
use tracing_subscriber::fmt::format::FmtSpan;

// // this part makes it possible to compile to wasm32 target
// #[cfg(target_arch = "wasm32")]
// use wasm_bindgen::prelude::*;
// #[cfg(target_arch = "wasm32")]
// #[wasm_bindgen(start)]
// pub fn main_js() -> Result<(), JsValue> {
//     main();
//     Ok(())
// }

/*
Apart from the basic real-time walking, this example shows how screenshots can be captured in-game.
Because it uses UpdateEvent, any combination of keys can be specified to activate it.
*/

const CONSOLE_WIDTH: u32 = 80;
const CONSOLE_HEIGHT: u32 = 45;

// type System = Box<dyn FnMut(&mut World)>;

struct MyRoguelike {
    world: World,
    systems: Vec<Box<dyn SystemFunc>>,
}

impl Engine for MyRoguelike {
    fn init(&mut self, api: &mut dyn DoryenApi) {
        tracing::info!("Initializing Duke Roguelike");
        api.con().register_color("white", (255, 255, 255, 255));
        api.con().register_color("red", (255, 92, 92, 255));
        api.con().register_color("blue", (192, 192, 255, 255));

        let player_entity = (
            Player {},
            Position::new((CONSOLE_WIDTH / 2) as isize, (CONSOLE_HEIGHT / 2) as isize),
            Renderable {
                glyph: '@',
                color: (255, 92, 92, 255),
            },
            Health::new(15),
            InputState::default(),
        );

        tracing::debug!(?player_entity, "Spawning player...");
        self.world.spawn(player_entity);

        tracing::debug!("Spawning goblins...");
        spawn_goblin(
            &mut self.world,
            5,
            (5, 10),
            (CONSOLE_WIDTH as usize - 2, CONSOLE_HEIGHT as usize - 2),
        );

        tracing::info!("Initializing all ECS systems...");
        for system in self.systems.iter_mut() {
            tracing::debug!("Initializing {}...", system.get_name());
            system.init(&mut self.world);
        }
    }
    fn update(&mut self, api: &mut dyn DoryenApi) -> Option<UpdateEvent> {
        // capture the screen
        // if input.key("ControlLeft") && input.key_pressed("KeyS") {
        //     self.screenshot_idx += 1;
        //     return Some(UpdateEvent::Capture(format!(
        //         "screenshot_{:03}.png",
        //         self.screenshot_idx
        //     )));
        // }

        // let api = Arc::new(RefCell::new(api));

        // let world = Arc::new(&mut self.world);

        tracing::trace!("Processing systems...");
        for system in &mut self.systems {
            tracing::trace!("Updating {}...", system.get_name());
            if let Err(e) = system.call(&mut self.world, api) {
                tracing::error!("Got error while running system {e:?}");
            }
        }
        // sleep(Duration::from_millis(250));

        None
    }
    fn render(&mut self, api: &mut dyn DoryenApi) {
        tracing::trace!("Rendering Roguelike...");
        let con = api.con();
        con.clear(
            Some((128, 128, 128, 255)),
            Some((0, 0, 0, 255)),
            Some('.' as u16),
        );
        // con.ascii(self.player_pos.0, self.player_pos.1, '@' as u16);
        // con.fore(self.player_pos.0, self.player_pos.1, (255, 255, 255, 255));

        for (_id, (pos, render)) in self.world.query::<(&Position, &Renderable)>().iter() {
            con.ascii(pos.x as i32, pos.y as i32, render.glyph as u16);
            con.fore(pos.x as i32, pos.y as i32, render.color);
        }
    }
}

impl MyRoguelike {
    pub fn new() -> Self {
        let world = World::new();
        let input_system = InputSystem::default();
        Self {
            world,
            systems: vec![
                Box::new(input_system),
                Box::new(AiSystem::new()),
                Box::new(DamageSystem::default()),
                Box::new(DeadCollector::default()),
            ],
        }
    }
}

fn setup_logger() {
    tracing_subscriber::fmt()
        .with_timer(tracing_subscriber::fmt::time::uptime())
        .with_span_events(FmtSpan::ACTIVE | FmtSpan::CLOSE)
        .with_line_number(true)
        .with_level(true)
        .fmt_fields(
            format::debug_fn(|writer, field, value| {
                if field.to_string() == "message".to_string() {
                    write!(writer, "{value:?}")
                } else {
                    write!(writer, "{field}: `{value:?}`")
                }
            })
            .delimited(", "),
        )
        .with_max_level(tracing::Level::DEBUG)
        .init();
}

fn main() {
    // tracing::subscriber::set_global_default()
    setup_logger();
    // here are all the available options.
    // better practise is to use default values (see other examples)
    let options = AppOptions {
        console_width: CONSOLE_WIDTH,
        console_height: CONSOLE_HEIGHT,
        screen_width: CONSOLE_WIDTH * 8,
        screen_height: CONSOLE_HEIGHT * 8,
        window_title: "my roguelike".to_owned(),
        font_path: "terminal_8x8.png".to_owned(),
        vsync: true,
        fullscreen: false,
        show_cursor: true,
        resizable: false,
        intercept_close_request: false,
        max_fps: 12,
    };
    let mut app = App::new(options);

    app.set_engine(Box::new(MyRoguelike::new()));

    app.run();
}
